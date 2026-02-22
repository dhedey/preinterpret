use super::*;

#[derive(Clone)]
pub(crate) struct SharedReference<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T> SharedReference<T> {
    pub(crate) fn deactivate(self) -> InactiveSharedReference<T> {
        self.0.core.data_mut().deactivate_reference(self.0.id);
        InactiveSharedReference(self.0)
    }

    /// A powerful map method which lets you place the resultant mapped reference inside
    /// a structure arbitrarily.
    pub(crate) fn emplace_map<O>(
        self,
        f: impl for<'e> FnOnce(&'e T, &mut SharedEmplacerV2<'e, T>) -> O,
    ) -> O {
        // SAFETY: The validity + safety invariants are upheld by `ReferenceableCore`
        // ... assuming this id is marked as a shared reference for the duration.
        // The emplacer ensures that this is upheld whilst it is alive; via either:
        // - Delegating to the created SharedReference if it is emplaced
        // - Surviving until Drop at the end of this method if it is not emplaced
        let copied_ref = unsafe { self.0.pointer.as_ref() };
        let mut emplacer = SharedEmplacerV2(self.0.into_emplacer());
        f(copied_ref, &mut emplacer)
    }

    /// SAFETY:
    /// - The caller must ensure that the ReferencePathExtension is correct
    ///   (an overly-specific ReferencePathExtension may cause safety issues)
    pub(crate) unsafe fn map<V: ?Sized + 'static>(
        self,
        f: impl FnOnce(&T) -> &V,
        path_extension: ReferencePathExtension,
        new_span: SpanRange,
    ) -> SharedReference<V> {
        self.emplace_map(move |input, emplacer| {
            emplacer.emplace(f(input), path_extension, new_span)
        })
    }

    /// SAFETY:
    /// - The caller must ensure that the ReferencePathExtension is correct
    ///   (an overly-specific ReferencePathExtension may cause safety issues)
    pub(crate) unsafe fn try_map<V: ?Sized + 'static, E>(
        self,
        f: impl FnOnce(&T) -> Result<&V, E>,
        path_extension: ReferencePathExtension,
        new_span: SpanRange,
    ) -> Result<SharedReference<V>, (E, SharedReference<T>)> {
        self.emplace_map(|input, emplacer| match f(input) {
            Ok(output) => Ok(emplacer.emplace(output, path_extension, new_span)),
            Err(e) => Err((e, emplacer.revert())),
        })
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
        unsafe { self.0.pointer.as_ref() }
    }
}

#[derive(Clone)]
pub(crate) struct InactiveSharedReference<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T: ?Sized> InactiveSharedReference<T> {
    pub(crate) fn activate(self) -> FunctionResult<SharedReference<T>> {
        self.0
            .core
            .data_mut()
            .activate_shared_reference(self.0.id)?;
        Ok(SharedReference(self.0))
    }
}

pub(crate) struct SharedEmplacerV2<'e, T: ?Sized>(EmplacerCore<'e, T>);

impl<'e, T: ?Sized> SharedEmplacerV2<'e, T> {
    pub(crate) fn revert(&mut self) -> SharedReference<T> {
        SharedReference(self.0.revert())
    }

    /// SAFETY:
    /// - The caller must ensure that the ReferencePathExtension is correct
    ///   (an overly-specific ReferencePathExtension may cause safety issues)
    pub(crate) unsafe fn emplace<V: 'static + ?Sized>(
        &mut self,
        value: &'e V,
        path_extension: ReferencePathExtension,
        new_span: SpanRange,
    ) -> SharedReference<V> {
        unsafe {
            // SAFETY: The lifetime 'e is equal to the &'e content argument in replace
            // So this guarantees that the returned reference is valid as long as the SharedSubRcRefCell exists
            self.emplace_unchecked(value, path_extension, new_span)
        }
    }

    /// SAFETY:
    /// - The caller must ensure that the value's lifetime is derived from the original content
    /// - The caller must ensure that the ReferencePathExtension is correct
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &V,
        path_extension: ReferencePathExtension,
        new_span: SpanRange,
    ) -> SharedReference<V> {
        // SAFETY: The pointer is from a reference so non-null
        let pointer = unsafe { NonNull::new_unchecked(value as *const V as *mut V) };
        // SAFETY:
        // - The caller ensures that the reference is derived from the original content
        // - The caller ensures that the ReferencePathExtension is correct
        unsafe { SharedReference(self.0.emplace_unchecked(pointer, path_extension, new_span)) }
    }
}
