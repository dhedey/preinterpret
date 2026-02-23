use std::mem::transmute;

use super::*;

/// A flexible type which can either be a reference to a value of type `T`,
/// or an emplaced reference from a [`SharedReference<T>`].
pub(crate) struct AnyRef<'a, T: ?Sized + 'static> {
    inner: AnyRefInner<'a, T>,
}

impl<'a, T: ?Sized + 'static> AnyRef<'a, T> {
    /// SAFETY: The caller must ensure the PathExtension is correct.
    #[allow(unused)]
    pub(crate) unsafe fn map<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r T) -> &'r S,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> AnyRef<'a, S> {
        match self.inner {
            AnyRefInner::Direct(value) => AnyRef {
                inner: AnyRefInner::Direct(f(value)),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(unsafe {
                    shared.map(|x| f(x), path_extension, new_span)
                }),
            },
        }
    }

    /// SAFETY: The caller must ensure the PathExtension is correct.
    #[allow(unused)]
    pub(crate) unsafe fn map_optional<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r T) -> Option<&'r S>,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> Option<AnyRef<'a, S>> {
        Some(match self.inner {
            AnyRefInner::Direct(value) => AnyRef {
                inner: AnyRefInner::Direct(f(value)?),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.emplace_map(|input, emplacer| match f(
                    input,
                ) {
                    Some(output) => {
                        Some(unsafe { emplacer.emplace(output, path_extension, new_span) })
                    }
                    None => {
                        let _ = emplacer.revert();
                        None
                    }
                })?),
            },
        })
    }

    pub(crate) fn replace<O>(
        self,
        f: impl for<'e> FnOnce(&'e T, &mut AnyRefEmplacer<'a, 'e, T>) -> O,
    ) -> O {
        let copied_ref = self.deref() as *const T;
        let mut emplacer = AnyRefEmplacer {
            inner: Some(self),
            encapsulation_lifetime: std::marker::PhantomData,
        };
        f(
            // SAFETY: The underlying reference is valid for the lifetime of self
            // So we can copy it fine
            unsafe { &*copied_ref },
            &mut emplacer,
        )
    }
}

pub(crate) struct AnyRefEmplacer<'a, 'e: 'a, T: 'static + ?Sized> {
    inner: Option<AnyRef<'a, T>>,
    encapsulation_lifetime: std::marker::PhantomData<&'e ()>,
}

impl<'a, 'e: 'a, T: 'static + ?Sized> AnyRefEmplacer<'a, 'e, T> {
    /// Returns the current span of the underlying reference (if encapsulated),
    /// or a placeholder span (if direct).
    pub(crate) fn current_span(&self) -> SpanRange {
        match &self
            .inner
            .as_ref()
            .expect("Emplacer already consumed")
            .inner
        {
            AnyRefInner::Direct(_) => SpanRange::new_single(Span::call_site()),
            AnyRefInner::Encapsulated(shared) => shared.current_span(),
        }
    }

    /// SAFETY: The caller must ensure the PathExtension is correct.
    pub(crate) unsafe fn emplace<V: 'static + ?Sized>(
        &mut self,
        value: &'e V,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> AnyRef<'a, V> {
        unsafe {
            // SAFETY: The lifetime 'e is equal to the &'e content argument in replace
            // So this guarantees that the returned reference is valid as long as the AnyRef exists
            self.emplace_unchecked(value, path_extension, new_span)
        }
    }

    /// SAFETY:
    /// - The caller must ensure that the value's lifetime is derived from the original content
    /// - The caller must ensure the PathExtension is correct
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &V,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> AnyRef<'a, V> {
        let any_ref = self
            .inner
            .take()
            .expect("You can only emplace to create a new AnyRef value once");
        match any_ref.inner {
            AnyRefInner::Direct(_) => AnyRef {
                // SAFETY: As defined in the rustdoc above
                inner: AnyRefInner::Direct(unsafe { transmute::<&V, &'static V>(value) }),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(unsafe {
                    shared.map(
                        |_| transmute::<&V, &'static V>(value),
                        path_extension,
                        new_span,
                    )
                }),
            },
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for AnyRef<'a, T> {
    fn from(value: &'a T) -> Self {
        Self {
            inner: AnyRefInner::Direct(value),
        }
    }
}

pub(crate) trait ToSpannedRef<'a> {
    type Target: ?Sized;
    fn into_ref(self) -> AnyRef<'a, Self::Target>;
    fn into_spanned_ref(self, source: impl HasSpanRange) -> Spanned<AnyRef<'a, Self::Target>>;
}

impl<'a, T: ?Sized> ToSpannedRef<'a> for &'a T {
    type Target = T;
    fn into_ref(self) -> AnyRef<'a, Self::Target> {
        self.into()
    }
    fn into_spanned_ref(self, source: impl HasSpanRange) -> Spanned<AnyRef<'a, Self::Target>> {
        self.into_ref().spanned(source)
    }
}

impl<'a, T: ?Sized> From<SharedReference<T>> for AnyRef<'a, T> {
    fn from(value: SharedReference<T>) -> Self {
        Self {
            inner: AnyRefInner::Encapsulated(value),
        }
    }
}

impl<T: ?Sized> ToSpannedRef<'static> for SharedReference<T> {
    type Target = T;
    fn into_ref(self) -> AnyRef<'static, Self::Target> {
        self.into()
    }
    fn into_spanned_ref(self, source: impl HasSpanRange) -> Spanned<AnyRef<'static, Self::Target>> {
        AnyRef::from(self).spanned(source)
    }
}

enum AnyRefInner<'a, T: 'static + ?Sized> {
    Direct(&'a T),
    Encapsulated(SharedReference<T>),
}

impl<'a, T: 'static + ?Sized> Deref for AnyRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match &self.inner {
            AnyRefInner::Direct(value) => value,
            AnyRefInner::Encapsulated(shared) => shared,
        }
    }
}

/// A flexible type which can either be a mutable reference to a value of type `T`,
/// or an emplaced reference from a [`MutableReference<T>`].
pub(crate) struct AnyMut<'a, T: 'static + ?Sized> {
    inner: AnyMutInner<'a, T>,
}

impl<'a, T: ?Sized> From<&'a mut T> for AnyMut<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Self {
            inner: AnyMutInner::Direct(value),
        }
    }
}

impl<'a, T: ?Sized + 'static> AnyMut<'a, T> {
    /// SAFETY: The caller must ensure the PathExtension is correct.
    #[allow(unused)]
    pub(crate) unsafe fn map<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> &'r mut S,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> AnyMut<'a, S> {
        match self.inner {
            AnyMutInner::Direct(value) => AnyMut {
                inner: AnyMutInner::Direct(f(value)),
            },
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(unsafe {
                    mutable.map(|x| f(x), path_extension, new_span)
                }),
            },
        }
    }

    /// SAFETY: The caller must ensure the PathExtension is correct.
    #[allow(unused)]
    pub(crate) unsafe fn map_optional<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> Option<&'r mut S>,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> Option<AnyMut<'a, S>> {
        Some(match self.inner {
            AnyMutInner::Direct(value) => AnyMut {
                inner: AnyMutInner::Direct(f(value)?),
            },
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable.emplace_map(
                    |input, emplacer| match f(input) {
                        Some(output) => {
                            Some(unsafe { emplacer.emplace(output, path_extension, new_span) })
                        }
                        None => {
                            let _ = emplacer.revert();
                            None
                        }
                    },
                )?),
            },
        })
    }

    pub(crate) fn replace<O>(
        mut self,
        f: impl for<'e> FnOnce(&'e mut T, &mut AnyMutEmplacer<'a, 'e, T>) -> O,
    ) -> O {
        let copied_mut = self.deref_mut() as *mut T;
        let mut emplacer = AnyMutEmplacer {
            inner: Some(self),
            encapsulation_lifetime: std::marker::PhantomData,
        };
        f(
            // SAFETY: We are cloning a mutable reference here, but it is safe because:
            // - What it's pointing at still lives, inside emplacer.inner
            // - No other "mutable reference" is created except at encapsulation time
            unsafe { &mut *copied_mut },
            &mut emplacer,
        )
    }
}

pub(crate) struct AnyMutEmplacer<'a, 'e: 'a, T: 'static + ?Sized> {
    inner: Option<AnyMut<'a, T>>,
    encapsulation_lifetime: std::marker::PhantomData<&'e ()>,
}

impl<'a, 'e: 'a, T: 'static + ?Sized> AnyMutEmplacer<'a, 'e, T> {
    /// Returns the current span of the underlying reference (if encapsulated),
    /// or a placeholder span (if direct).
    pub(crate) fn current_span(&self) -> SpanRange {
        match &self
            .inner
            .as_ref()
            .expect("Emplacer already consumed")
            .inner
        {
            AnyMutInner::Direct(_) => SpanRange::new_single(Span::call_site()),
            AnyMutInner::Encapsulated(mutable) => mutable.current_span(),
        }
    }

    /// SAFETY: The caller must ensure the PathExtension is correct.
    pub(crate) unsafe fn emplace<V: 'static + ?Sized>(
        &mut self,
        value: &'e mut V,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> AnyMut<'a, V> {
        unsafe {
            // SAFETY: The lifetime 'e is equal to the &'e content argument in replace
            // So this guarantees that the returned reference is valid as long as the AnyMut exists
            self.emplace_unchecked(value, path_extension, new_span)
        }
    }

    /// SAFETY:
    /// - The caller must ensure that the value's lifetime is derived from the original content
    /// - The caller must ensure the PathExtension is correct
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &mut V,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) -> AnyMut<'a, V> {
        let any_mut = self
            .inner
            .take()
            .expect("You can only emplace to create a new AnyMut value once");
        match any_mut.inner {
            AnyMutInner::Direct(_) => AnyMut {
                // SAFETY: As defined in the rustdoc above
                inner: AnyMutInner::Direct(unsafe { transmute::<&mut V, &'static mut V>(value) }),
            },
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(unsafe {
                    mutable.map(
                        |_| transmute::<&mut V, &'static mut V>(value),
                        path_extension,
                        new_span,
                    )
                }),
            },
        }
    }
}

#[allow(unused)]
pub(crate) trait IntoAnyMut<'a> {
    type Target: ?Sized;
    fn into_any_mut(self) -> AnyMut<'a, Self::Target>;
}

impl<'a, T: ?Sized + 'static> IntoAnyMut<'a> for &'a mut T {
    type Target = T;

    fn into_any_mut(self) -> AnyMut<'a, Self::Target> {
        self.into()
    }
}

impl<'a, T: ?Sized> From<MutableReference<T>> for AnyMut<'a, T> {
    fn from(value: MutableReference<T>) -> Self {
        Self {
            inner: AnyMutInner::Encapsulated(value),
        }
    }
}

enum AnyMutInner<'a, T: 'static + ?Sized> {
    Direct(&'a mut T),
    Encapsulated(MutableReference<T>),
}

impl<'a, T: 'static + ?Sized> Deref for AnyMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match &self.inner {
            AnyMutInner::Direct(value) => value,
            AnyMutInner::Encapsulated(shared) => shared,
        }
    }
}

impl<'a, T: 'static + ?Sized> DerefMut for AnyMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        match &mut self.inner {
            AnyMutInner::Direct(value) => value,
            AnyMutInner::Encapsulated(shared) => &mut *shared,
        }
    }
}
