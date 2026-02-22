use super::*;

pub(crate) struct SharedReference<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T: ?Sized> Clone for SharedReference<T> {
    fn clone(&self) -> Self {
        SharedReference(self.0.clone())
    }
}

impl<T: ?Sized> SharedReference<T> {
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
    /// - The caller must ensure that the PathExtension is correct
    ///   (an overly-specific PathExtension may cause safety issues)
    pub(crate) unsafe fn map<V: ?Sized + 'static>(
        self,
        f: impl FnOnce(&T) -> &V,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> SharedReference<V> {
        self.emplace_map(move |input, emplacer| {
            emplacer.emplace(f(input), path_extension, new_span)
        })
    }

    /// SAFETY:
    /// - The caller must ensure that the PathExtension is correct
    ///   (an overly-specific PathExtension may cause safety issues)
    pub(crate) unsafe fn try_map<V: ?Sized + 'static, E>(
        self,
        f: impl FnOnce(&T) -> Result<&V, E>,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> Result<SharedReference<V>, (E, SharedReference<T>)> {
        self.emplace_map(|input, emplacer| match f(input) {
            Ok(output) => Ok(emplacer.emplace(output, path_extension, new_span)),
            Err(e) => Err((e, emplacer.revert())),
        })
    }
}

impl SharedReference<AnyValue> {
    /// Creates a new SharedReference from an owned value, wrapping it in a Referenceable.
    /// Uses a placeholder name and span for the Referenceable root.
    pub(crate) fn new_from_owned(value: AnyValue) -> Self {
        let referenceable = Referenceable::new(
            value,
            "<anonymous>".to_string(),
            SpanRange::new_single(Span::call_site()),
        );
        referenceable
            .new_inactive_shared()
            .activate()
            .expect("Freshly created referenceable must be borrowable as shared")
    }

    /// Clones the inner value. This is infallible because AnyValue always supports clone.
    pub(crate) fn infallible_clone(&self) -> AnyValue {
        (**self).clone()
    }
}

impl<T: ?Sized> SharedReference<T> {
    /// Disables this shared reference (bridge for old `disable()` API).
    /// Equivalent to `deactivate()` in the new naming.
    pub(crate) fn disable(self) -> InactiveSharedReference<T> {
        self.deactivate()
    }

    /// Safe map that uses a placeholder path extension.
    /// This is a bridge method for migration - callers should eventually switch to
    /// the unsafe `map()` with proper PathExtension.
    pub(crate) fn map_legacy<V: ?Sized + 'static>(
        self,
        value_map: impl FnOnce(&T) -> &V,
    ) -> SharedReference<V> {
        self.emplace_map(move |input, emplacer| {
            // SAFETY: We use Tightened(AnyType::type_kind()) as a conservative path extension
            // that says "we're still at the same depth, just changing our view".
            // This may be overly conservative but won't cause memory safety issues.
            unsafe {
                emplacer.emplace(
                    value_map(input),
                    PathExtension::Tightened(AnyType::type_kind()),
                    SpanRange::new_single(Span::call_site()),
                )
            }
        })
    }

    /// Safe try_map that uses a placeholder path extension.
    /// Bridge method for migration.
    pub(crate) fn try_map_legacy<V: ?Sized + 'static, E>(
        self,
        value_map: impl FnOnce(&T) -> Result<&V, E>,
    ) -> Result<SharedReference<V>, (E, SharedReference<T>)> {
        self.emplace_map(|input, emplacer| match value_map(input) {
            Ok(output) => {
                // SAFETY: Same conservative path extension as map_legacy
                Ok(unsafe {
                    emplacer.emplace(
                        output,
                        PathExtension::Tightened(AnyType::type_kind()),
                        SpanRange::new_single(Span::call_site()),
                    )
                })
            }
            Err(e) => Err((e, emplacer.revert())),
        })
    }

    /// Safe map_optional that uses a placeholder path extension.
    /// Bridge method for migration.
    pub(crate) fn map_optional_legacy<V: ?Sized + 'static>(
        self,
        value_map: impl FnOnce(&T) -> Option<&V>,
    ) -> Option<SharedReference<V>> {
        self.emplace_map(|input, emplacer| match value_map(input) {
            Some(output) => {
                // SAFETY: Same conservative path extension as map_legacy
                Some(unsafe {
                    emplacer.emplace(
                        output,
                        PathExtension::Tightened(AnyType::type_kind()),
                        SpanRange::new_single(Span::call_site()),
                    )
                })
            }
            None => {
                let _ = emplacer.revert(); // drop the reverted reference
                None
            }
        })
    }

    /// Bridge for the old `replace()` pattern.
    /// Uses the emplace_map internally with a legacy path extension.
    pub(crate) fn replace_legacy<O>(
        self,
        f: impl for<'e> FnOnce(&'e T, &mut SharedEmplacerV2<'e, T>) -> O,
    ) -> O {
        self.emplace_map(f)
    }
}

impl<T: ?Sized> InactiveSharedReference<T> {
    /// Re-enables this inactive shared reference (bridge for old `enable()` API).
    /// The span parameter is kept for API compatibility but activation errors
    /// now include span information from the reference itself.
    pub(crate) fn enable(self, _span: SpanRange) -> FunctionResult<SharedReference<T>> {
        self.activate()
    }
}

impl<T: ?Sized> Deref for SharedReference<T> {
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

impl<T: ?Sized> AsRef<T> for SharedReference<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

pub(crate) struct InactiveSharedReference<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T: ?Sized> Clone for InactiveSharedReference<T> {
    fn clone(&self) -> Self {
        InactiveSharedReference(self.0.clone())
    }
}

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
    /// - The caller must ensure that the PathExtension is correct
    ///   (an overly-specific PathExtension may cause safety issues)
    pub(crate) unsafe fn emplace<V: 'static + ?Sized>(
        &mut self,
        value: &'e V,
        path_extension: PathExtension,
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
    /// - The caller must ensure that the PathExtension is correct
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &V,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> SharedReference<V> {
        // SAFETY: The pointer is from a reference so non-null
        let pointer = unsafe { NonNull::new_unchecked(value as *const V as *mut V) };
        // SAFETY:
        // - The caller ensures that the reference is derived from the original content
        // - The caller ensures that the PathExtension is correct
        unsafe { SharedReference(self.0.emplace_unchecked(pointer, path_extension, new_span)) }
    }

    /// Legacy bridge: emplace_unchecked without PathExtension.
    /// Uses a conservative default path extension.
    ///
    /// SAFETY:
    /// - The caller must ensure that the value's lifetime is derived from the original content
    pub(crate) unsafe fn emplace_unchecked_legacy<V: 'static + ?Sized>(
        &mut self,
        value: &V,
    ) -> SharedReference<V> {
        unsafe {
            self.emplace_unchecked(
                value,
                PathExtension::Tightened(AnyType::type_kind()),
                SpanRange::new_single(Span::call_site()),
            )
        }
    }
}

/// Legacy type alias for backward compatibility
pub(crate) type SharedEmplacer<'e, T> = SharedEmplacerV2<'e, T>;
