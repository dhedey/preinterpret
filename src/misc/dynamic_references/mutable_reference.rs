use super::*;

pub(crate) struct MutableReference<T>(pub(super) ReferenceCore<T>);

impl<T> MutableReference<T> {
    pub(crate) fn deactivate(self) -> InactiveMutableReference<T> {
        self.0.deactivate_reference();
        InactiveMutableReference(self.0)
    }
}

impl<T> Deref for MutableReference<T> {
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

impl<T> DerefMut for MutableReference<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: See the rustdoc on dynamic_references/mod.rs for full details.
        // To summarize:
        // - Provided by the safety invariants of `ReferenceCore` and `ReferenceableCore`.
        // - The pointer is guaranteed to be valid and properly aligned for the duration of the reference's lifetime.
        // - The reference kind checks ensure that it is not aliased while active.
        unsafe { &mut *self.0.pointer }
    }
}