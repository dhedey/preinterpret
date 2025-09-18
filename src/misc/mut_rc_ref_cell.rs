#![allow(unused)] // TODO: Remove when places are properly late-bound
use crate::internal_prelude::*;
use std::cell::*;
use std::rc::Rc;

/// A mutable reference to a sub-value `U` inside a [`Rc<RefCell<T>>`].
/// Only one [`MutSubRcRefCell`] can exist at a time for a given [`Rc<RefCell<T>>`].
pub(crate) struct MutSubRcRefCell<T: 'static, U: 'static> {
    /// This is actually a reference to the contents of the RefCell
    /// but we store it using `unsafe` as `'static`, and use unsafe blocks
    /// to ensure it's dropped first.
    ///
    /// SAFETY: This *must* appear before `pointed_at` so that it's dropped first.
    ref_mut: RefMut<'static, U>,
    pointed_at: Rc<RefCell<T>>,
}

impl<T: 'static> MutSubRcRefCell<T, T> {
    pub(crate) fn new(pointed_at: Rc<RefCell<T>>) -> Result<Self, BorrowMutError> {
        let ref_mut = pointed_at.try_borrow_mut()?;
        Ok(Self {
            // SAFETY: We must ensure that this lifetime lives as long as the
            // reference to pointed_at (i.e. the RefCell).
            // This is guaranteed by the fact that the only time we drop the RefCell
            // is when we drop the MutRcRefCell, and we ensure that the RefMut is dropped first.
            ref_mut: unsafe { std::mem::transmute::<RefMut<'_, T>, RefMut<'static, T>>(ref_mut) },
            pointed_at,
        })
    }
}

impl<T: 'static, U: 'static> MutSubRcRefCell<T, U> {
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

    pub(crate) fn map<V>(self, f: impl FnOnce(&mut U) -> &mut V) -> MutSubRcRefCell<T, V> {
        MutSubRcRefCell {
            ref_mut: RefMut::map(self.ref_mut, f),
            pointed_at: self.pointed_at,
        }
    }

    pub(crate) fn try_map<V, E>(
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

impl<T: 'static, U: 'static> DerefMut for MutSubRcRefCell<T, U> {
    fn deref_mut(&mut self) -> &mut U {
        &mut self.ref_mut
    }
}

impl<T: 'static, U: 'static> Deref for MutSubRcRefCell<T, U> {
    type Target = U;
    fn deref(&self) -> &U {
        &self.ref_mut
    }
}

/// A shared (immutable) reference to a sub-value `U` inside a [`Rc<RefCell<T>>`].
/// Many [`SharedSubRcRefCell`] can exist at the same time for a given [`Rc<RefCell<T>>`],
/// but if any exist, then no [`MutSubRcRefCell`] can exist.
pub(crate) struct SharedSubRcRefCell<T: 'static, U: 'static> {
    /// This is actually a reference to the contents of the RefCell
    /// but we store it using `unsafe` as `'static`, and use unsafe blocks
    /// to ensure it's dropped first.
    ///
    /// SAFETY: This *must* appear before `pointed_at` so that it's dropped first.
    shared_ref: Ref<'static, U>,
    pointed_at: Rc<RefCell<T>>,
}

impl<T: 'static> SharedSubRcRefCell<T, T> {
    pub(crate) fn new(pointed_at: Rc<RefCell<T>>) -> Result<Self, BorrowError> {
        let shared_ref = pointed_at.try_borrow()?;
        Ok(Self {
            // SAFETY: We must ensure that this lifetime lives as long as the
            // reference to pointed_at (i.e. the RefCell).
            // This is guaranteed by the fact that the only time we drop the RefCell
            // is when we drop the SharedSubRcRefCell, and we ensure that the Ref is dropped first.
            shared_ref: unsafe { std::mem::transmute::<Ref<'_, T>, Ref<'static, T>>(shared_ref) },
            pointed_at,
        })
    }
}

impl<T: 'static, U: 'static> SharedSubRcRefCell<T, U> {
    pub(crate) fn map<V>(self, f: impl FnOnce(&U) -> &V) -> SharedSubRcRefCell<T, V> {
        SharedSubRcRefCell {
            shared_ref: Ref::map(self.shared_ref, f),
            pointed_at: self.pointed_at,
        }
    }

    pub(crate) fn try_map<V, E>(
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
}

impl<T: 'static, U: 'static> Deref for SharedSubRcRefCell<T, U> {
    type Target = U;
    fn deref(&self) -> &U {
        &self.shared_ref
    }
}
