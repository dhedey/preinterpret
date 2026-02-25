use super::*;

/// A flexible type which can either be a reference to a value of type `T`,
/// or an emplaced reference from a [`SharedReference<T>`].
pub(crate) struct AnyRef<'a, T: ?Sized + 'static> {
    inner: AnyRefInner<'a, T>,
}

impl<'a, T: ?Sized + 'static> AnyRef<'a, T> {
    /// Maps this `AnyRef` using a closure that returns a [`MappedRef`].
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedRef::new`] constructor.
    #[allow(unused)]
    pub(crate) fn map<S: ?Sized + 'static>(
        self,
        f: impl for<'r> FnOnce(&'r T) -> MappedRef<'r, S>,
    ) -> AnyRef<'a, S> {
        match self.inner {
            AnyRefInner::Direct(value) => {
                let (value, _path_extension, _span) = f(value).into_parts();
                AnyRef {
                    inner: AnyRefInner::Direct(value),
                }
            }
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.map(f)),
            },
        }
    }

    /// Maps this `AnyRef` using a closure that returns an optional [`MappedRef`].
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedRef::new`] constructor.
    #[allow(unused)]
    pub(crate) fn map_optional<S: ?Sized + 'static>(
        self,
        f: impl for<'r> FnOnce(&'r T) -> Option<MappedRef<'r, S>>,
    ) -> Option<AnyRef<'a, S>> {
        Some(match self.inner {
            AnyRefInner::Direct(value) => {
                let (value, _path_extension, _span) = f(value)?.into_parts();
                AnyRef {
                    inner: AnyRefInner::Direct(value),
                }
            }
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.emplace_map(|input, emplacer| match f(
                    input,
                ) {
                    Some(mapped) => Some(emplacer.emplace(mapped)),
                    None => {
                        let _ = emplacer.revert();
                        None
                    }
                })?),
            },
        })
    }

    pub(crate) fn emplace_map<O>(
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
    pub(crate) fn revert(&mut self) -> AnyRef<'a, T> {
        self.inner.take().expect("Emplacer already consumed")
    }

    /// Emplaces a mapped shared reference, consuming the emplacer's reference tracking.
    ///
    /// This is safe because all preconditions (correct PathExtension and valid reference
    /// derivation) are validated by the [`MappedRef`] constructor.
    pub(crate) fn emplace<V: 'static + ?Sized>(
        &mut self,
        mapped: MappedRef<'e, V>,
    ) -> AnyRef<'a, V> {
        let any_ref = self
            .inner
            .take()
            .expect("You can only emplace to create a new AnyRef value once");
        let (value, path_extension, span) = mapped.into_parts();
        match any_ref.inner {
            AnyRefInner::Direct(_) => AnyRef {
                // 'e: 'a is guaranteed by the struct constraint, so coercion is valid
                inner: AnyRefInner::Direct(value),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.emplace_map(|_, emplacer| {
                    // SAFETY: The value's lifetime ('e) matches the SharedEmplacer's lifetime
                    // since both are derived from the same underlying data
                    let remapped = unsafe { MappedRef::new_unchecked(value, path_extension, span) };
                    emplacer.emplace(remapped)
                })),
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
    /// Maps this `AnyMut` using a closure that returns a [`MappedMut`].
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedMut::new`] constructor.
    #[allow(unused)]
    pub(crate) fn map<S: ?Sized + 'static>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> MappedMut<'r, S>,
    ) -> AnyMut<'a, S> {
        match self.inner {
            AnyMutInner::Direct(value) => {
                let (value, _path_extension, _span) = f(value).into_parts();
                AnyMut {
                    inner: AnyMutInner::Direct(value),
                }
            }
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable.map(f)),
            },
        }
    }

    /// Maps this `AnyMut` using a closure that returns an optional [`MappedMut`].
    ///
    /// The unsafe PathExtension assertion is confined to the [`MappedMut::new`] constructor.
    #[allow(unused)]
    pub(crate) fn map_optional<S: ?Sized + 'static>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> Option<MappedMut<'r, S>>,
    ) -> Option<AnyMut<'a, S>> {
        Some(match self.inner {
            AnyMutInner::Direct(value) => {
                let (value, _path_extension, _span) = f(value)?.into_parts();
                AnyMut {
                    inner: AnyMutInner::Direct(value),
                }
            }
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable.emplace_map(
                    |input, emplacer| match f(input) {
                        Some(mapped) => Some(emplacer.emplace(mapped)),
                        None => {
                            let _ = emplacer.revert();
                            None
                        }
                    },
                )?),
            },
        })
    }

    pub(crate) fn emplace_map<O>(
        self,
        f: impl for<'e> FnOnce(&'e mut T, &mut AnyMutEmplacer<'a, 'e, T>) -> O,
    ) -> O {
        match self.inner {
            AnyMutInner::Direct(direct_mut) => {
                // Convert to raw pointer to avoid &mut aliasing: the emplacer
                // stores the raw pointer, so no second &mut T exists.
                let raw_ptr = direct_mut as *mut T;
                let mut emplacer = AnyMutEmplacer {
                    inner: Some(AnyMutEmplacerState::Direct(raw_ptr, PhantomData)),
                    encapsulation_lifetime: std::marker::PhantomData,
                };
                // SAFETY: raw_ptr was derived from a valid &'a mut T, and the original
                // reference was consumed by converting to *mut T.
                f(unsafe { &mut *raw_ptr }, &mut emplacer)
            }
            AnyMutInner::Encapsulated(mut mutable) => {
                // MutableReference internally stores NonNull<T> (a raw pointer),
                // not &mut T, so there is no aliasing issue.
                let raw_ptr = (&mut *mutable) as *mut T;
                let mut emplacer = AnyMutEmplacer {
                    inner: Some(AnyMutEmplacerState::Encapsulated(mutable)),
                    encapsulation_lifetime: std::marker::PhantomData,
                };
                // SAFETY: raw_ptr was derived from the MutableReference which
                // uses NonNull internally (not &mut), so no aliasing occurs.
                f(unsafe { &mut *raw_ptr }, &mut emplacer)
            }
        }
    }
}

pub(crate) struct AnyMutEmplacer<'a, 'e: 'a, T: 'static + ?Sized> {
    inner: Option<AnyMutEmplacerState<'a, T>>,
    encapsulation_lifetime: std::marker::PhantomData<&'e ()>,
}

/// Stores either a raw pointer (for Direct) or a MutableReference (for Encapsulated),
/// avoiding &mut aliasing that would occur if we stored the full AnyMut.
enum AnyMutEmplacerState<'a, T: 'static + ?Sized> {
    Direct(*mut T, PhantomData<&'a mut T>),
    Encapsulated(MutableReference<T>),
}

impl<'a, 'e: 'a, T: 'static + ?Sized> AnyMutEmplacer<'a, 'e, T> {
    pub(crate) fn revert(&mut self) -> AnyMut<'a, T> {
        let state = self.inner.take().expect("Emplacer already consumed");
        match state {
            AnyMutEmplacerState::Direct(ptr, _) => AnyMut {
                // SAFETY: ptr was derived from a valid &'a mut T and no other
                // &mut T currently exists (the one passed to the closure has ended).
                inner: AnyMutInner::Direct(unsafe { &mut *ptr }),
            },
            AnyMutEmplacerState::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable),
            },
        }
    }

    /// Emplaces a mapped mutable reference, consuming the emplacer's reference tracking.
    ///
    /// This is safe because all preconditions (correct PathExtension and valid reference
    /// derivation) are validated by the [`MappedMut`] constructor.
    pub(crate) fn emplace<V: 'static + ?Sized>(
        &mut self,
        mapped: MappedMut<'e, V>,
    ) -> AnyMut<'a, V> {
        let state = self
            .inner
            .take()
            .expect("You can only emplace to create a new AnyMut value once");
        let (value, path_extension, span) = mapped.into_parts();
        match state {
            AnyMutEmplacerState::Direct(_, _) => AnyMut {
                // 'e: 'a is guaranteed by the struct constraint, so coercion is valid
                inner: AnyMutInner::Direct(value),
            },
            AnyMutEmplacerState::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable.emplace_map(|_, emplacer| {
                    // SAFETY: The value's lifetime ('e) matches the MutableEmplacer's lifetime
                    // since both are derived from the same underlying data
                    let remapped = unsafe { MappedMut::new_unchecked(value, path_extension, span) };
                    emplacer.emplace(remapped)
                })),
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
