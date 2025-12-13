use crate::internal_prelude::*;
use std::cell::{BorrowError, BorrowMutError};

/// A mutable reference to a sub-value `U` inside a [`Rc<RefCell<T>>`].
/// Only one [`MutSubRcRefCell`] can exist at a time for a given [`Rc<RefCell<T>>`].
pub(crate) struct MutSubRcRefCell<T: 'static + ?Sized, U: 'static + ?Sized> {
    /// This is actually a reference to the contents of the RefCell
    /// but we store it using `unsafe` as `'static`, and use unsafe blocks
    /// to ensure it's dropped first.
    ///
    /// SAFETY: This *must* appear before `pointed_at` so that it's dropped first.
    ref_mut: RefMut<'static, U>,
    pointed_at: Rc<RefCell<T>>,
}

impl<T: 'static + ?Sized> MutSubRcRefCell<T, T> {
    pub(crate) fn new(pointed_at: Rc<RefCell<T>>) -> Result<Self, BorrowMutError> {
        let ref_mut = pointed_at.try_borrow_mut()?;
        Ok(Self {
            // SAFETY: We must ensure that this lifetime lives as long as the
            // reference to pointed_at (i.e. the RefCell).
            // This is guaranteed by the fact that the only time we drop the RefCell
            // is when we drop the MutRcRefCell, and we ensure that the RefMut is dropped first.
            ref_mut: unsafe {
                less_buggy_transmute::<std::cell::RefMut<'_, T>, std::cell::RefMut<'static, T>>(
                    ref_mut,
                )
            },
            pointed_at,
        })
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> MutSubRcRefCell<T, U> {
    pub(crate) fn into_shared(self) -> SharedSubRcRefCell<T, U> {
        let ptr = self.ref_mut.deref() as *const U;
        drop(self.ref_mut);
        // SAFETY:
        // - The pointer was previously a reference, so it is safe to deference it here
        //   (the pointer is pointing into the Rc<RefCell<...>> which hasn't moved)
        // - All our invariants for SharedSubRcRefCell / MutSubRcRefCell are maintained
        unsafe {
            // The unwrap cannot panic because we just held a mutable borrow, we're not in Sync land, so no-one else can have a borrow.
            SharedSubRcRefCell::new(self.pointed_at)
                .unwrap()
                .map(|_| &*ptr)
        }
    }

    /// SAFETY:
    /// * Must be paired with a call to `enable()` before any further use of the value.
    /// * Must not use the value while disabled.
    pub(crate) unsafe fn disable(&mut self) {
        // Ideally we'd just decrement the ref count, but RefCell doesn't expose that.
        // Instead, we duplicate it, so the old value gets dropped automatically,
        // decrementing the ref count.
        self.ref_mut = unsafe { core::ptr::read(&self.ref_mut) };
    }

    /// SAFETY:
    /// * Must only be used after a call to `disable()`.
    pub(crate) unsafe fn enable(&mut self) -> Result<(), BorrowMutError> {
        // Ideally we'd just increment the ref count, but RefCell doesn't expose that.
        // Instead, we re-borrow it mutably, which increments the ref count, then forget
        // the new borrow.
        match self.pointed_at.try_borrow_mut() {
            Ok(new_ref_mut) => {
                std::mem::forget(new_ref_mut);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn map<V: ?Sized>(self, f: impl FnOnce(&mut U) -> &mut V) -> MutSubRcRefCell<T, V> {
        MutSubRcRefCell {
            ref_mut: RefMut::map(self.ref_mut, f),
            pointed_at: self.pointed_at,
        }
    }

    pub(crate) fn try_map<V: ?Sized, E>(
        self,
        f: impl FnOnce(&mut U) -> Result<&mut V, E>,
    ) -> Result<MutSubRcRefCell<T, V>, E> {
        let mut error = None;
        let outcome = RefMut::filter_map(self.ref_mut, |inner| match f(inner) {
            Ok(value) => Some(value),
            Err(e) => {
                error = Some(e);
                None
            }
        });
        match outcome {
            Ok(ref_mut) => Ok(MutSubRcRefCell {
                ref_mut,
                pointed_at: self.pointed_at,
            }),
            Err(_) => Err(error.unwrap()),
        }
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> DerefMut for MutSubRcRefCell<T, U> {
    fn deref_mut(&mut self) -> &mut U {
        &mut self.ref_mut
    }
}

impl<T: 'static + ?Sized, U: 'static + ?Sized> Deref for MutSubRcRefCell<T, U> {
    type Target = U;
    fn deref(&self) -> &U {
        &self.ref_mut
    }
}

/// A shared (immutable) reference to a sub-value `U` inside a [`Rc<RefCell<T>>`].
/// Many [`SharedSubRcRefCell`] can exist at the same time for a given [`Rc<RefCell<T>>`],
/// but if any exist, then no [`MutSubRcRefCell`] can exist.
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
    pub(crate) fn new(pointed_at: Rc<RefCell<T>>) -> Result<Self, BorrowError> {
        let shared_ref = pointed_at.try_borrow()?;
        Ok(Self {
            // SAFETY: We must ensure that this lifetime lives as long as the
            // reference to pointed_at (i.e. the RefCell).
            // This is guaranteed by the fact that the only time we drop the RefCell
            // is when we drop the SharedSubRcRefCell, and we ensure that the Ref is dropped first.
            shared_ref: unsafe { less_buggy_transmute::<Ref<'_, T>, Ref<'static, T>>(shared_ref) },
            pointed_at,
        })
    }
}

impl<T: ?Sized, U: 'static + ?Sized> SharedSubRcRefCell<T, U> {
    pub(crate) fn clone(this: &SharedSubRcRefCell<T, U>) -> Self {
        Self {
            shared_ref: Ref::clone(&this.shared_ref),
            pointed_at: Rc::clone(&this.pointed_at),
        }
    }

    pub(crate) fn map<V: ?Sized>(self, f: impl FnOnce(&U) -> &V) -> SharedSubRcRefCell<T, V> {
        SharedSubRcRefCell {
            shared_ref: Ref::map(self.shared_ref, f),
            pointed_at: self.pointed_at,
        }
    }

    pub(crate) fn try_map<V: ?Sized, E>(
        self,
        f: impl FnOnce(&U) -> Result<&V, E>,
    ) -> Result<SharedSubRcRefCell<T, V>, E> {
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
            Err(_) => Err(error.unwrap()),
        }
    }

    /// SAFETY:
    /// * Must be paired with a call to `enable()` before any further use of the value.
    /// * Must not use the value while disabled.
    pub(crate) unsafe fn disable(&mut self) {
        // Ideally we'd just decrement the ref count, but RefCell doesn't expose that.
        // Instead, we duplicate it, so the old value gets dropped automatically,
        // decrementing the ref count.
        self.shared_ref = unsafe { core::ptr::read(&self.shared_ref) };
    }

    /// SAFETY:
    /// * Must only be used after a call to `disable()`.
    pub(crate) unsafe fn enable(&mut self) -> Result<(), BorrowError> {
        // Ideally we'd just increment the ref count, but RefCell doesn't expose that.
        // Instead, we re-borrow it mutably, which increments the ref count, then forget
        // the new borrow.
        match self.pointed_at.try_borrow() {
            Ok(new_ref_mut) => {
                std::mem::forget(new_ref_mut);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl<T: ?Sized, U: 'static + ?Sized> Deref for SharedSubRcRefCell<T, U> {
    type Target = U;

    fn deref(&self) -> &U {
        &self.shared_ref
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
