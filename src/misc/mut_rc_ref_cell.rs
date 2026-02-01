use crate::internal_prelude::*;
use std::cell::{BorrowError, BorrowMutError};

/// A mutable reference to a sub-value `U` inside a [`Rc<RefCell<T>>`].
/// Only one [`MutableSubRcRefCell`] can exist at a time for a given [`Rc<RefCell<T>>`].
pub(crate) struct MutableSubRcRefCell<T: 'static + ?Sized, U: 'static + ?Sized> {
    /// This is actually a reference to the contents of the RefCell
    /// but we store it using `unsafe` as `'static`, and use unsafe blocks
    /// to ensure it's dropped first.
    ///
    /// SAFETY: This *must* appear before `pointed_at` so that it's dropped first.
    ref_mut: RefMut<'static, U>,
    pointed_at: Rc<RefCell<T>>,
}

impl<T: 'static + ?Sized> MutableSubRcRefCell<T, T> {
    pub(crate) fn new(pointed_at: Rc<RefCell<T>>) -> Result<Self, Rc<RefCell<T>>> {
        let ref_mut = match pointed_at.try_borrow_mut() {
            Ok(ref_mut) => {
                // SAFETY: We must ensure that this lifetime lives as long as the
                // reference to pointed_at (i.e. the RefCell).
                // This is guaranteed by the fact that the only time we drop the RefCell
                // is when we drop the MutRcRefCell, and we ensure that the RefMut is dropped first.
                unsafe {
                    Some(less_buggy_transmute::<
                        std::cell::RefMut<'_, T>,
                        std::cell::RefMut<'static, T>,
                    >(ref_mut))
                }
            }
            Err(_) => None,
        };
        match ref_mut {
            Some(ref_mut) => Ok(Self {
                ref_mut,
                pointed_at,
            }),
            None => Err(pointed_at),
        }
    }

    pub(crate) fn new_from_owned(owned: T) -> Self
    where
        T: Sized,
    {
        let rc = Rc::new(RefCell::new(owned));
        Self::new(rc).unwrap_or_else(|_| unreachable!("New refcell must be mut borrowable"))
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> MutableSubRcRefCell<T, U> {
    pub(crate) fn into_shared(self) -> SharedSubRcRefCell<T, U> {
        let ptr = self.ref_mut.deref() as *const U;
        drop(self.ref_mut);
        // SAFETY:
        // - The pointer was previously a reference, so it is safe to deference it here
        //   (the pointer is pointing into the Rc<RefCell<...>> which hasn't moved)
        // - All our invariants for SharedSubRcRefCell / MutableSubRcRefCell are maintained
        unsafe {
            // The unwrap cannot panic because we just held a mutable borrow, we're not in Sync land, so no-one else can have a borrow.
            SharedSubRcRefCell::new(self.pointed_at)
                .unwrap_or_else(|_| {
                    unreachable!("Must be able to create shared after holding mutable")
                })
                .map(|_| &*ptr)
        }
    }

    /// Disables this mutable reference, releasing the borrow on the RefCell.
    /// Returns a `DisabledMutableSubRcRefCell` which can be cloned and later re-enabled.
    pub(crate) fn disable(self) -> DisabledMutableSubRcRefCell<T, U> {
        let sub_ptr = self.ref_mut.deref() as *const U as *mut U;
        // Drop the RefMut to release the borrow
        drop(self.ref_mut);
        DisabledMutableSubRcRefCell {
            pointed_at: self.pointed_at,
            sub_ptr,
        }
    }

    pub(crate) fn map<V: ?Sized>(
        self,
        f: impl FnOnce(&mut U) -> &mut V,
    ) -> MutableSubRcRefCell<T, V> {
        MutableSubRcRefCell {
            ref_mut: RefMut::map(self.ref_mut, f),
            pointed_at: self.pointed_at,
        }
    }

    pub(crate) fn map_optional<V: ?Sized>(
        self,
        f: impl FnOnce(&mut U) -> Option<&mut V>,
    ) -> Option<MutableSubRcRefCell<T, V>> {
        Some(MutableSubRcRefCell {
            ref_mut: RefMut::filter_map(self.ref_mut, f).ok()?,
            pointed_at: self.pointed_at,
        })
    }

    pub(crate) fn try_map<V: ?Sized, E>(
        self,
        f: impl FnOnce(&mut U) -> Result<&mut V, E>,
    ) -> Result<MutableSubRcRefCell<T, V>, (E, MutableSubRcRefCell<T, U>)> {
        let mut error = None;
        let outcome = RefMut::filter_map(self.ref_mut, |inner| match f(inner) {
            Ok(value) => Some(value),
            Err(e) => {
                error = Some(e);
                None
            }
        });
        match outcome {
            Ok(ref_mut) => Ok(MutableSubRcRefCell {
                ref_mut,
                pointed_at: self.pointed_at,
            }),
            Err(original_ref_mut) => Err((
                error.unwrap(),
                MutableSubRcRefCell {
                    ref_mut: original_ref_mut,
                    pointed_at: self.pointed_at,
                },
            )),
        }
    }

    pub(crate) fn replace<O>(
        mut self,
        f: impl for<'a> FnOnce(&'a mut U, &mut MutableSubEmplacer<'a, T, U>) -> O,
    ) -> O {
        let ref_mut = self.ref_mut.deref_mut() as *mut U;
        let mut emplacer = MutableSubEmplacer {
            inner: Some(self),
            encapsulation_lifetime: std::marker::PhantomData,
        };
        f(
            // SAFETY: We are cloning a mutable reference here, but it is safe because:
            // - What it's pointing at still lives, as RefMut still lives inside emplacer.inner
            // - No other "mutable reference" is created from the RefMut except at encapsulation time
            unsafe { &mut *ref_mut },
            &mut emplacer,
        )
    }
}

#[allow(unused)]
pub(crate) type MutableEmplacer<'e, U> = MutableSubEmplacer<'e, AnyValue, U>;

pub(crate) struct MutableSubEmplacer<'e, T: 'static + ?Sized, U: 'static + ?Sized> {
    inner: Option<MutableSubRcRefCell<T, U>>,
    encapsulation_lifetime: std::marker::PhantomData<&'e ()>,
}

impl<'e, T: 'static + ?Sized, U: 'static + ?Sized> MutableSubEmplacer<'e, T, U> {
    pub(crate) fn emplace<V: 'static + ?Sized>(
        &mut self,
        value: &'e mut V,
    ) -> MutableSubRcRefCell<T, V> {
        unsafe {
            // SAFETY: The lifetime 'e is equal to the &'e content argument in replace
            // So this guarantees that the returned reference is valid as long as the MutableSubRcRefCell exists
            self.emplace_unchecked(value)
        }
    }

    // SAFETY:
    // * The caller must ensure that the value's lifetime is derived from the original content
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &mut V,
    ) -> MutableSubRcRefCell<T, V> {
        self.inner
            .take()
            .expect("You can only emplace to create a new Mutable value once")
            .map(|_|
                // SAFETY: As defined in the rustdoc above
                unsafe { less_buggy_transmute::<&mut V, &'static mut V>(value) })
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> DerefMut for MutableSubRcRefCell<T, U> {
    fn deref_mut(&mut self) -> &mut U {
        &mut self.ref_mut
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> Deref for MutableSubRcRefCell<T, U> {
    type Target = U;
    fn deref(&self) -> &U {
        &self.ref_mut
    }
}

/// A disabled mutable reference that can be safely cloned and dropped.
///
/// This type holds just the `Rc` and a raw pointer to the sub-value,
/// without an active borrow on the RefCell. This means:
/// - Dropping is safe (no borrow count to decrement)
/// - Cloning is safe (just clones the Rc and copies the pointer)
/// - The value cannot be accessed until re-enabled
pub(crate) struct DisabledMutableSubRcRefCell<T: 'static + ?Sized, U: 'static + ?Sized> {
    pointed_at: Rc<RefCell<T>>,
    sub_ptr: *mut U,
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> Clone for DisabledMutableSubRcRefCell<T, U> {
    fn clone(&self) -> Self {
        Self {
            pointed_at: Rc::clone(&self.pointed_at),
            sub_ptr: self.sub_ptr,
        }
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> DisabledMutableSubRcRefCell<T, U> {
    /// Re-enables this disabled mutable reference by re-acquiring the borrow.
    ///
    /// Returns an error if the RefCell is currently borrowed.
    pub(crate) fn enable(self) -> Result<MutableSubRcRefCell<T, U>, BorrowMutError> {
        let ref_mut = self.pointed_at.try_borrow_mut()?;
        // SAFETY:
        // - sub_ptr was derived from a valid &mut U inside the RefCell
        // - The Rc is still alive, so the RefCell contents haven't moved
        // - We just acquired a mutable borrow, so we have exclusive access
        let ref_mut = RefMut::map(ref_mut, |_| unsafe { &mut *self.sub_ptr });
        Ok(MutableSubRcRefCell {
            ref_mut: unsafe { less_buggy_transmute::<RefMut<'_, U>, RefMut<'static, U>>(ref_mut) },
            pointed_at: self.pointed_at,
        })
    }

    pub(crate) fn into_shared(self) -> DisabledSharedSubRcRefCell<T, U> {
        DisabledSharedSubRcRefCell {
            pointed_at: self.pointed_at,
            sub_ptr: self.sub_ptr as *const U,
        }
    }
}

/// A shared (immutable) reference to a sub-value `U` inside a [`Rc<RefCell<T>>`].
/// Many [`SharedSubRcRefCell`] can exist at the same time for a given [`Rc<RefCell<T>>`],
/// but if any exist, then no [`MutableSubRcRefCell`] can exist.
pub(crate) struct SharedSubRcRefCell<T: ?Sized, U: 'static + ?Sized> {
    /// This is actually a reference to the contents of the RefCell
    /// but we store it using `unsafe` as `'static`, and use unsafe blocks
    /// to ensure it's dropped first.
    ///
    /// SAFETY: This *must* appear before `pointed_at` so that it's dropped first.
    shared_ref: Ref<'static, U>,
    pointed_at: Rc<RefCell<T>>,
}

impl<T: 'static + ?Sized> SharedSubRcRefCell<T, T> {
    pub(crate) fn new(pointed_at: Referenceable<T>) -> Result<Self, Referenceable<T>> {
        let shared_ref = match pointed_at.try_borrow() {
            Ok(shared_ref) => {
                // SAFETY: We must ensure that this lifetime lives as long as the
                // reference to pointed_at (i.e. the RefCell).
                // This is guaranteed by the fact that the only time we drop the RefCell
                // is when we drop the SharedSubRcRefCell, and we ensure that the Ref is dropped first.
                unsafe {
                    Some(less_buggy_transmute::<Ref<'_, T>, Ref<'static, T>>(
                        shared_ref,
                    ))
                }
            }
            Err(_) => None,
        };
        match shared_ref {
            Some(shared_ref) => Ok(Self {
                shared_ref,
                pointed_at,
            }),
            None => Err(pointed_at),
        }
    }

    pub(crate) fn new_from_owned(owned: T) -> Self
    where
        T: Sized,
    {
        let rc = Rc::new(RefCell::new(owned));
        Self::new(rc).unwrap_or_else(|_| unreachable!("New refcell must be borrowable"))
    }
}

impl<T: ?Sized, U: 'static + ?Sized> SharedSubRcRefCell<T, U> {
    pub(crate) fn clone(this: &SharedSubRcRefCell<T, U>) -> Self {
        Self {
            shared_ref: Ref::clone(&this.shared_ref),
            pointed_at: Rc::clone(&this.pointed_at),
        }
    }

    pub(crate) fn map<V: ?Sized>(
        self,
        f: impl for<'a> FnOnce(&'a U) -> &'a V,
    ) -> SharedSubRcRefCell<T, V> {
        SharedSubRcRefCell {
            shared_ref: Ref::map(self.shared_ref, f),
            pointed_at: self.pointed_at,
        }
    }

    pub(crate) fn map_optional<V: ?Sized>(
        self,
        f: impl FnOnce(&U) -> Option<&V>,
    ) -> Option<SharedSubRcRefCell<T, V>> {
        Some(SharedSubRcRefCell {
            shared_ref: Ref::filter_map(self.shared_ref, f).ok()?,
            pointed_at: self.pointed_at,
        })
    }

    pub(crate) fn try_map<V: ?Sized, E>(
        self,
        f: impl FnOnce(&U) -> Result<&V, E>,
    ) -> Result<SharedSubRcRefCell<T, V>, (E, SharedSubRcRefCell<T, U>)> {
        let mut error = None;
        let outcome = Ref::filter_map(self.shared_ref, |inner| match f(inner) {
            Ok(value) => Some(value),
            Err(e) => {
                error = Some(e);
                None
            }
        });
        match outcome {
            Ok(shared_ref) => Ok(SharedSubRcRefCell {
                shared_ref,
                pointed_at: self.pointed_at,
            }),
            Err(original_shared_ref) => Err((
                error.unwrap(),
                SharedSubRcRefCell {
                    shared_ref: original_shared_ref,
                    pointed_at: self.pointed_at,
                },
            )),
        }
    }

    pub(crate) fn replace<O>(
        self,
        f: impl for<'e> FnOnce(&'e U, &mut SharedSubEmplacer<'e, T, U>) -> O,
    ) -> O {
        let copied_ref = Ref::clone(&self.shared_ref);
        let mut emplacer = SharedSubEmplacer {
            inner: Some(self),
            encapsulation_lifetime: std::marker::PhantomData,
        };
        f(&*copied_ref, &mut emplacer)
    }

    /// Disables this shared reference, releasing the borrow on the RefCell.
    /// Returns a `DisabledSharedSubRcRefCell` which can be cloned and later re-enabled.
    pub(crate) fn disable(self) -> DisabledSharedSubRcRefCell<T, U> {
        let sub_ptr = self.shared_ref.deref() as *const U;
        // Drop the Ref to release the borrow
        drop(self.shared_ref);
        DisabledSharedSubRcRefCell {
            pointed_at: self.pointed_at,
            sub_ptr,
        }
    }
}

pub(crate) type SharedEmplacer<'e, U> = SharedSubEmplacer<'e, AnyValue, U>;

pub(crate) struct SharedSubEmplacer<'e, T: ?Sized, U: 'static + ?Sized> {
    inner: Option<SharedSubRcRefCell<T, U>>,
    encapsulation_lifetime: std::marker::PhantomData<&'e ()>,
}

impl<'e, T: 'static + ?Sized, U: 'static + ?Sized> SharedSubEmplacer<'e, T, U> {
    pub(crate) fn emplace<V: 'static + ?Sized>(
        &mut self,
        value: &'e V,
    ) -> SharedSubRcRefCell<T, V> {
        unsafe {
            // SAFETY: The lifetime 'e is equal to the &'e content argument in replace
            // So this guarantees that the returned reference is valid as long as the SharedSubRcRefCell exists
            self.emplace_unchecked(value)
        }
    }

    // SAFETY:
    // * The caller must ensure that the value's lifetime is derived from the original content
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &V,
    ) -> SharedSubRcRefCell<T, V> {
        self.inner
            .take()
            .expect("You can only emplace to create a new shared value once")
            .map(|_|
                // SAFETY: As defined in the rustdoc above
                unsafe { less_buggy_transmute::<&V, &'static V>(value) })
    }
}

impl<T: ?Sized, U: 'static + ?Sized> Deref for SharedSubRcRefCell<T, U> {
    type Target = U;

    fn deref(&self) -> &U {
        &self.shared_ref
    }
}

/// A disabled shared reference that can be safely cloned and dropped.
///
/// This type holds just the `Rc` and a raw pointer to the sub-value,
/// without an active borrow on the RefCell. This means:
/// - Dropping is safe (no borrow count to decrement)
/// - Cloning is safe (just clones the Rc and copies the pointer)
/// - The value cannot be accessed until re-enabled
pub(crate) struct DisabledSharedSubRcRefCell<T: 'static + ?Sized, U: 'static + ?Sized> {
    pointed_at: Rc<RefCell<T>>,
    sub_ptr: *const U,
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> Clone for DisabledSharedSubRcRefCell<T, U> {
    fn clone(&self) -> Self {
        Self {
            pointed_at: Rc::clone(&self.pointed_at),
            sub_ptr: self.sub_ptr,
        }
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> DisabledSharedSubRcRefCell<T, U> {
    /// Re-enables this disabled shared reference by re-acquiring the borrow.
    ///
    /// Returns an error if the RefCell is currently mutably borrowed.
    pub(crate) fn enable(self) -> Result<SharedSubRcRefCell<T, U>, BorrowError> {
        let shared_ref = self.pointed_at.try_borrow()?;
        // SAFETY:
        // - sub_ptr was derived from a valid &U inside the RefCell
        // - The Rc is still alive, so the RefCell contents haven't moved
        // - We just acquired a shared borrow, so the data is valid
        let shared_ref = Ref::map(shared_ref, |_| unsafe { &*self.sub_ptr });
        Ok(SharedSubRcRefCell {
            shared_ref: unsafe { less_buggy_transmute::<Ref<'_, U>, Ref<'static, U>>(shared_ref) },
            pointed_at: self.pointed_at,
        })
    }
}

/// SAFETY: The user must ensure that the two types are transmutable,
/// and in particular are the same size
unsafe fn less_buggy_transmute<T, U>(t: T) -> U {
    // std::mem::transmute::<Ref<'_, T>, Ref<'static, T>> for T: ?Sized
    // Is fine on latest Rust, but on MSRV only incorrectly flags:
    // > error[E0512]: cannot transmute between types of different sizes, or dependently-sized types
    // Likely on old versions of Rust, it assumes that Ref is therefore ?Sized (it's not).
    // To workaround this, we use a recommendation from https://users.rust-lang.org/t/transmute-doesnt-work-on-generic-types/87272
    // using transmute_copy and manual forgetting
    use std::mem::ManuallyDrop;
    debug_assert!(std::mem::size_of::<T>() == std::mem::size_of::<U>());
    std::mem::transmute_copy::<ManuallyDrop<T>, U>(&ManuallyDrop::new(t))
}
