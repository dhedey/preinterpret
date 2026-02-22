use super::*;

#[derive(Clone)]
pub(crate) struct SharedReference<T>(pub(super) ReferenceCore<T>);

impl<T> SharedReference<T> {
    pub(crate) fn deactivate(self) -> InactiveSharedReference<T> {
        self.0.deactivate_reference();
        InactiveSharedReference(self.0)
    }
}

impl<T> Deref for SharedReference<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: See the rustdoc on dynamic_references/mod.rs for full details.
        // To summarize:
        // - Provided by the safety invariants of `ReferenceCore` and `ReferenceableCore`.
        // - The pointer is guaranteed to be valid and properly aligned for the duration of the reference's lifetime.
        // - The reference kind checks ensure that it is not mutably aliased while active.
        unsafe { &*self.0.pointer }
    }
}
