use super::*;

pub(crate) struct MutableReference<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T: ?Sized> MutableReference<T> {
    pub(crate) fn deactivate(self) -> InactiveMutableReference<T> {
        self.0.core.data_mut().deactivate_reference(self.0.id);
        InactiveMutableReference(self.0)
    }

    /// A powerful map method which lets you place the resultant mapped reference inside
    /// a structure arbitrarily.
    pub(crate) fn emplace_map<O>(
        mut self,
        f: impl for<'e> FnOnce(&'e mut T, &mut MutableEmplacer<'e, T>) -> O,
    ) -> O {
        // SAFETY: The validity + safety invariants are upheld by `ReferenceableCore`
        // ... assuming this id is marked as a mutable reference for the duration.
        // The emplacer ensures that this is upheld whilst it is alive; via either:
        // - Delegating to the created MutableReference if it is emplaced
        // - Surviving until Drop at the end of this method if it is not emplaced
        let copied_mut = unsafe { self.0.pointer.as_mut() };
        let mut emplacer = MutableEmplacer(self.0.into_emplacer());
        f(copied_mut, &mut emplacer)
    }

    /// Maps this mutable reference using a closure that returns a [`MappedMut`].
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedMut::new`] constructor,
    /// making this method itself safe.
    pub(crate) fn map<V: ?Sized + 'static>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> MappedMut<'r, V>,
    ) -> MutableReference<V> {
        self.emplace_map(move |input, emplacer| {
            let (value, path_extension, span) = f(input).into_parts();
            // SAFETY: MappedMut constructor already validated the PathExtension.
            // The lifetime is checked by emplace (value: &'e mut V).
            unsafe { emplacer.emplace(value, path_extension, Some(span)) }
        })
    }

    /// Fallible version of [`map`](Self::map) that returns the original reference on error.
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedMut::new`] constructor.
    pub(crate) fn try_map<V: ?Sized + 'static, E>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> Result<MappedMut<'r, V>, E>,
    ) -> Result<MutableReference<V>, (E, MutableReference<T>)> {
        self.emplace_map(|input, emplacer| match f(input) {
            Ok(mapped) => {
                let (value, path_extension, span) = mapped.into_parts();
                // SAFETY: MappedMut constructor already validated the PathExtension.
                // The lifetime is checked by emplace (value: &'e mut V).
                Ok(unsafe { emplacer.emplace(value, path_extension, Some(span)) })
            }
            Err(e) => Err((e, emplacer.revert())),
        })
    }
}

impl MutableReference<AnyValue> {
    /// Creates a new MutableReference from an owned value, wrapping it in a Referenceable.
    pub(crate) fn new_from_owned(
        value: AnyValue,
        root_name: Option<String>,
        span_range: SpanRange,
    ) -> Self {
        let referenceable = Referenceable::new(value, root_name, span_range);
        referenceable
            .new_inactive_mutable()
            .activate()
            .expect("Freshly created referenceable must be borrowable as mutable")
    }
}

impl<T: ?Sized> MutableReference<T> {
    /// Disables this mutable reference (bridge for old `disable()` API).
    /// Equivalent to `deactivate()` in the new naming.
    pub(crate) fn disable(self) -> InactiveMutableReference<T> {
        self.deactivate()
    }

    /// Converts this mutable reference into a shared reference.
    /// Bridge for old `into_shared()` API.
    pub(crate) fn into_shared(self) -> SharedReference<T> {
        let inactive = self.deactivate();
        let inactive_shared = inactive.into_shared();
        inactive_shared
            .activate()
            .expect("Converting mutable to shared should always succeed since we just released the mutable borrow")
    }
}

impl<T: ?Sized> InactiveMutableReference<T> {
    /// Re-enables this inactive mutable reference (bridge for old `enable()` API).
    pub(crate) fn enable(self, _span: SpanRange) -> FunctionResult<MutableReference<T>> {
        self.activate()
    }
}

impl<T: ?Sized> Deref for MutableReference<T> {
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

impl<T: ?Sized> AsRef<T> for MutableReference<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> AsMut<T> for MutableReference<T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized> DerefMut for MutableReference<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: See the rustdoc on dynamic_references/mod.rs for full details.
        // To summarize:
        // - Provided by the safety invariants of `ReferenceCore` and `ReferenceableCore`.
        // - The pointer is guaranteed to be valid and properly aligned for the duration of the reference's lifetime.
        // - The reference kind checks ensure that it is not aliased while active.
        unsafe { self.0.pointer.as_mut() }
    }
}

pub(crate) struct InactiveMutableReference<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T: ?Sized> Clone for InactiveMutableReference<T> {
    fn clone(&self) -> Self {
        InactiveMutableReference(self.0.clone())
    }
}

impl<T: ?Sized> InactiveMutableReference<T> {
    pub(crate) fn activate(self) -> FunctionResult<MutableReference<T>> {
        self.0
            .core
            .data_mut()
            .activate_mutable_reference(self.0.id)?;
        Ok(MutableReference(self.0))
    }

    pub(crate) fn into_shared(self) -> InactiveSharedReference<T> {
        // As an inactive reference, we are free to map between them
        // ... we could even enable the other way around, but that'd likely allow breaking
        // application invariants which we want to respect.
        self.0.core.data_mut().make_shared(self.0.id);
        InactiveSharedReference(self.0)
    }
}

pub(crate) struct MutableEmplacer<'e, T: ?Sized>(EmplacerCore<'e, T>);

impl<'e, T: ?Sized> MutableEmplacer<'e, T> {
    pub(crate) fn revert(&mut self) -> MutableReference<T> {
        MutableReference(self.0.revert())
    }

    /// SAFETY: The caller must ensure that the PathExtension is correct.
    ///
    /// The lifetime `'e` is checked by the compiler, ensuring the value is derived
    /// from the original content.
    pub(crate) unsafe fn emplace<V: 'static + ?Sized>(
        &mut self,
        value: &'e mut V,
        path_extension: PathExtension,
        new_span: Option<SpanRange>,
    ) -> MutableReference<V> {
        unsafe { self.emplace_unchecked(value, path_extension, new_span) }
    }

    /// SAFETY:
    /// - The caller must ensure that the value's lifetime is derived from the original content
    /// - The caller must ensure that the PathExtension is correct
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &mut V,
        path_extension: PathExtension,
        new_span: Option<SpanRange>,
    ) -> MutableReference<V> {
        // SAFETY: The pointer is from a reference so non-null
        let pointer = unsafe { NonNull::new_unchecked(value as *mut V) };
        // SAFETY:
        // - The caller ensures that the reference is derived from the original content
        // - The caller ensures that the PathExtension is correct
        unsafe { MutableReference(self.0.emplace_unchecked(pointer, path_extension, new_span)) }
    }
}
