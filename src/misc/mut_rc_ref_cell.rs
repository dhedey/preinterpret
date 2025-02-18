use crate::internal_prelude::*;
use std::cell::*;
use std::rc::Rc;

pub(crate) struct MutRcRefCell<T: 'static, U: 'static> {
    /// This is actually a reference to the contents of the RefCell
    /// but we store it using `unsafe` as `'static`, and use unsafe blocks
    /// to ensure it's dropped first.
    ///
    /// SAFETY: This *must* appear before `pointed_at` so that it's dropped first.
    ref_mut: RefMut<'static, U>,
    pointed_at: Rc<RefCell<T>>,
}

impl<T: 'static> MutRcRefCell<T, T> {
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

impl<T: 'static, U: 'static> MutRcRefCell<T, U> {
    pub(crate) fn try_map<V, E>(
        self,
        f: impl FnOnce(&mut U) -> Result<&mut V, E>,
    ) -> Result<MutRcRefCell<T, V>, E> {
        let mut error = None;
        let outcome = RefMut::filter_map(self.ref_mut, |inner| match f(inner) {
            Ok(value) => Some(value),
            Err(e) => {
                error = Some(e);
                None
            }
        });
        match outcome {
            Ok(ref_mut) => Ok(MutRcRefCell {
                ref_mut,
                pointed_at: self.pointed_at,
            }),
            Err(_) => Err(error.unwrap()),
        }
    }
}

impl<T: 'static, U: 'static> DerefMut for MutRcRefCell<T, U> {
    fn deref_mut(&mut self) -> &mut U {
        &mut self.ref_mut
    }
}

impl<T: 'static, U: 'static> Deref for MutRcRefCell<T, U> {
    type Target = U;
    fn deref(&self) -> &U {
        &self.ref_mut
    }
}
