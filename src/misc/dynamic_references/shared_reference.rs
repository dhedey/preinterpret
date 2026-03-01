use std::mem::transmute;

use super::*;

/// Represents an active `&T` reference, to something derived from a Referenceable root.
pub(crate) struct Shared<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<T: ?Sized> Shared<T> {
    pub(crate) fn deactivate(self) -> InactiveShared<T> {
        self.0.core.data_mut().deactivate_reference(self.0.id);
        InactiveShared(self.0)
    }

    /// A powerful map method which lets you place the resultant mapped reference inside
    /// a structure arbitrarily.
    pub(crate) fn emplace_map<O>(
        self,
        f: impl for<'a> FnOnce(&'a T, &mut SharedEmplacer<'a, T>) -> O,
    ) -> O {
        // SAFETY: The validity + safety invariants are upheld by `ReferenceableCore`
        // ... assuming this id is marked as a shared reference for the duration.
        // The emplacer ensures that this is upheld whilst it is alive; via either:
        // - Delegating to the created SharedReference if it is emplaced
        // - Surviving until Drop at the end of this method if it is not emplaced
        let copied_ref = unsafe { self.0.pointer.as_ref() };
        let mut emplacer = SharedEmplacer(self.0.into_emplacer());
        f(copied_ref, &mut emplacer)
    }

    /// Maps this shared reference using a closure that returns a [`MappedRef`].
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedRef::new`] constructor,
    /// making this method itself safe.
    pub(crate) fn map<V: ?Sized + 'static>(
        self,
        f: impl for<'a> FnOnce(&'a T) -> MappedRef<'a, V>,
    ) -> Shared<V> {
        self.emplace_map(move |input, emplacer| emplacer.emplace(f(input)))
    }

    /// Fallible version of [`map`](Self::map) that returns the original reference on error.
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedRef::new`] constructor.
    pub(crate) fn try_map<V: ?Sized + 'static, E>(
        self,
        f: impl for<'a> FnOnce(&'a T) -> Result<MappedRef<'a, V>, E>,
    ) -> Result<Shared<V>, (E, Shared<T>)> {
        self.emplace_map(|input, emplacer| match f(input) {
            Ok(mapped) => Ok(emplacer.emplace(mapped)),
            Err(e) => Err((e, emplacer.revert())),
        })
    }
}

impl Shared<AnyValue> {
    /// Creates a new SharedReference from an owned value, wrapping it in a Referenceable.
    /// Uses a placeholder name and span for the Referenceable root.
    pub(crate) fn new_from_owned(
        value: AnyValue,
        root_name: Option<String>,
        span_range: SpanRange,
    ) -> Self {
        let referenceable = Referenceable::new(value, root_name, span_range);
        referenceable
            .new_active_shared(span_range)
            .expect("Freshly created referenceable must be borrowable as shared")
    }

    /// Clones the inner value. This is infallible because AnyValue always supports clone.
    pub(crate) fn infallible_clone(&self) -> AnyValue {
        (**self).clone()
    }
}

impl<T: ?Sized> Deref for Shared<T> {
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

impl<T: ?Sized> AsRef<T> for Shared<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

/// Represents an inactive but *valid* shared reference.
///
/// It can't currently deref into a `&T`, but can after being activated, when
/// the aliasing rules get checked and enforced.
pub(crate) struct InactiveShared<T: ?Sized>(pub(super) ReferenceCore<T>);

impl<T: ?Sized> Clone for InactiveShared<T> {
    fn clone(&self) -> Self {
        InactiveShared(self.0.clone())
    }
}

impl<T: ?Sized> InactiveShared<T> {
    pub(crate) fn activate(self, span: SpanRange) -> FunctionResult<Shared<T>> {
        self.0
            .core
            .data_mut()
            .activate_shared_reference(self.0.id, span)?;
        Ok(Shared(self.0))
    }
}

/// A mapped shared reference bundled with its [`PathExtension`] and span.
///
/// Constructing this is unsafe because the caller must ensure the
/// [`PathExtension`] correctly describes the relationship between the
/// source and mapped reference.
pub(crate) struct MappedRef<'a, V: ?Sized> {
    value: &'a V,
    path_extension: PathExtension,
    span: Option<SpanRange>,
}

impl<'a, V: ?Sized> MappedRef<'a, V> {
    /// SAFETY: The caller must ensure that the PathExtension correctly describes
    /// the navigation from the source reference to this mapped reference.
    /// An overly-specific PathExtension may cause safety issues.
    pub(crate) unsafe fn new(value: &'a V, path_extension: PathExtension, span: SpanRange) -> Self {
        Self {
            value,
            path_extension,
            span: Some(span),
        }
    }

    /// SAFETY: In addition to the safety requirements of [`new`](Self::new), the caller
    /// must ensure that the reference lifetime is valid for the target lifetime `'a`.
    /// This is needed when the compiler cannot prove the lifetime relationship
    /// (e.g. in LeafMapper implementations where `'l` and `'e` cannot be unified).
    pub(crate) unsafe fn new_unchecked<'any>(
        value: &'any V,
        path_extension: PathExtension,
        span: Option<SpanRange>,
    ) -> Self {
        Self {
            value: unsafe { transmute::<&'any V, &'a V>(value) },
            path_extension,
            span,
        }
    }

    pub(crate) fn into_parts(self) -> (&'a V, PathExtension, Option<SpanRange>) {
        (self.value, self.path_extension, self.span)
    }
}

pub(crate) struct SharedEmplacer<'a, T: ?Sized>(EmplacerCore<'a, T>);

impl<'a, T: ?Sized> SharedEmplacer<'a, T> {
    pub(crate) fn revert(&mut self) -> Shared<T> {
        Shared(self.0.revert())
    }

    /// Emplaces a mapped shared reference, consuming the emplacer's reference tracking.
    ///
    /// This is safe because all preconditions (correct PathExtension and valid reference
    /// derivation) are validated by the [`MappedRef`] constructor.
    pub(crate) fn emplace<'e: 'a, V: 'static + ?Sized>(
        &mut self,
        mapped: MappedRef<'e, V>,
    ) -> Shared<V> {
        let (value, path_extension, span) = mapped.into_parts();
        // SAFETY: The pointer is from a valid reference (guaranteed by MappedRef constructor),
        // and the PathExtension was validated by the MappedRef constructor.
        let pointer = unsafe { NonNull::new_unchecked(value as *const V as *mut V) };
        unsafe { Shared(self.0.emplace_unchecked(pointer, path_extension, span)) }
    }
}
