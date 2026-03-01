use std::mem::transmute;

use super::*;

/// A flexible type which can either be a reference to a value of type `T`,
/// or an emplaced reference from a [`Shared<T>`].
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
        f: impl for<'m> FnOnce(&'m T, &mut AnyRefEmplacer<'_, 'a, 'm, T>) -> O,
    ) -> O {
        match self.inner {
            AnyRefInner::Direct(direct_ref) => {
                let raw_ptr = direct_ref as *const T;
                let mut emplacer = AnyRefEmplacer {
                    inner: AnyRefEmplacerState::Direct(DirectRefEmplacerState {
                        ptr: Some(raw_ptr),
                        original_lifetime: PhantomData,
                    }),
                };
                // SAFETY: raw_ptr was derived from a valid &'a T, and the original
                // reference was consumed by converting to *const T.
                f(unsafe { &*raw_ptr }, &mut emplacer)
            }
            AnyRefInner::Encapsulated(shared) => shared.emplace_map(|x, shared_emplacer| {
                let mut emplacer = AnyRefEmplacer {
                    inner: AnyRefEmplacerState::Encapsulated(shared_emplacer),
                };
                f(x, &mut emplacer)
            }),
        }
    }
}

pub(crate) struct AnyRefEmplacer<'x, 'a, 'm: 'x, T: 'static + ?Sized> {
    inner: AnyRefEmplacerState<'x, 'a, 'm, T>,
}

enum AnyRefEmplacerState<'x, 'a, 'm: 'x, T: 'static + ?Sized> {
    Direct(DirectRefEmplacerState<'a, T>),
    Encapsulated(&'x mut SharedEmplacer<'m, T>),
}

struct DirectRefEmplacerState<'a, T: 'static + ?Sized> {
    ptr: Option<*const T>,
    original_lifetime: PhantomData<&'a T>,
}

impl<'x, 'a, 'm, T: 'static + ?Sized> AnyRefEmplacer<'x, 'a, 'm, T> {
    pub(crate) fn revert(&mut self) -> AnyRef<'a, T> {
        match &mut self.inner {
            AnyRefEmplacerState::Direct(DirectRefEmplacerState { ptr, .. }) => {
                let ptr = ptr.take().expect("Emplacer already consumed");
                AnyRef {
                    // SAFETY: ptr was derived from a valid &'a T and the original
                    // reference was consumed when creating the emplacer.
                    inner: AnyRefInner::Direct(unsafe { &*ptr }),
                }
            }
            AnyRefEmplacerState::Encapsulated(emplacer) => AnyRef {
                inner: AnyRefInner::Encapsulated(emplacer.revert()),
            },
        }
    }

    /// Emplaces a mapped shared reference, consuming the emplacer's reference tracking.
    ///
    /// This is safe because all preconditions (correct PathExtension and valid reference
    /// derivation) are validated by the [`MappedRef`] constructor.
    pub(crate) fn emplace<V: 'static + ?Sized>(
        &mut self,
        mapped: MappedRef<'m, V>,
    ) -> AnyRef<'a, V> {
        match &mut self.inner {
            AnyRefEmplacerState::Direct(_) => {
                let (value, _, _) = mapped.into_parts();
                // SAFETY: In Direct case, 'm == 'a, so the lifetime of the value is valid for 'a
                let value = unsafe { transmute::<&'m V, &'a V>(value) };
                AnyRef {
                    inner: AnyRefInner::Direct(value),
                }
            }
            AnyRefEmplacerState::Encapsulated(emplacer) => AnyRef {
                inner: AnyRefInner::Encapsulated(emplacer.emplace(mapped)),
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

impl<'a, T: ?Sized> From<Shared<T>> for AnyRef<'a, T> {
    fn from(value: Shared<T>) -> Self {
        Self {
            inner: AnyRefInner::Encapsulated(value),
        }
    }
}

impl<T: ?Sized> ToSpannedRef<'static> for Shared<T> {
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
    Encapsulated(Shared<T>),
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
/// or an emplaced reference from a [`Mutable<T>`].
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
        f: impl for<'m> FnOnce(&'m mut T, &mut AnyMutEmplacer<'_, 'a, 'm, T>) -> O,
    ) -> O {
        match self.inner {
            AnyMutInner::Direct(direct_mut) => {
                // Convert to raw pointer to avoid &mut aliasing: the emplacer
                // stores the raw pointer, so no second &mut T exists.
                let raw_ptr = direct_mut as *mut T;
                let mut emplacer = AnyMutEmplacer {
                    inner: AnyMutEmplacerState::Direct(DirectMutEmplacerState {
                        ptr: Some(raw_ptr),
                        original_lifetime: PhantomData,
                    }),
                };
                // SAFETY: raw_ptr was derived from a valid &'a mut T, and the original
                // reference was consumed by converting to *mut T.
                f(unsafe { &mut *raw_ptr }, &mut emplacer)
            }
            AnyMutInner::Encapsulated(mutable) => mutable.emplace_map(|x, mut_emplacer| {
                let mut emplacer = AnyMutEmplacer {
                    inner: AnyMutEmplacerState::Encapsulated(mut_emplacer),
                };

                f(x, &mut emplacer)
            }),
        }
    }
}

pub(crate) struct AnyMutEmplacer<'x, 'a, 'm: 'x, T: 'static + ?Sized> {
    inner: AnyMutEmplacerState<'x, 'a, 'm, T>,
}

enum AnyMutEmplacerState<'x, 'a, 'm: 'x, T: 'static + ?Sized> {
    Direct(DirectMutEmplacerState<'a, T>),
    Encapsulated(&'x mut MutableEmplacer<'m, T>),
}

struct DirectMutEmplacerState<'a, T: 'static + ?Sized> {
    ptr: Option<*mut T>,
    original_lifetime: PhantomData<&'a mut T>,
}

impl<'x, 'a, 'm, T: 'static + ?Sized> AnyMutEmplacer<'x, 'a, 'm, T> {
    pub(crate) fn revert(&mut self) -> AnyMut<'a, T> {
        match &mut self.inner {
            AnyMutEmplacerState::Direct(DirectMutEmplacerState { ptr, .. }) => {
                let ptr = ptr.take().expect("Emplacer already consumed");
                AnyMut {
                    // SAFETY: ptr was derived from a valid &'a mut T and no other
                    // &mut T currently exists (the one passed to the closure has ended).
                    inner: AnyMutInner::Direct(unsafe { &mut *ptr }),
                }
            }
            AnyMutEmplacerState::Encapsulated(emplacer) => AnyMut {
                inner: AnyMutInner::Encapsulated(emplacer.revert()),
            },
        }
    }

    /// Emplaces a mapped mutable reference, consuming the emplacer's reference tracking.
    ///
    /// This is safe because all preconditions (correct PathExtension and valid reference
    /// derivation) are validated by the [`MappedMut`] constructor.
    pub(crate) fn emplace<V: 'static + ?Sized>(
        &mut self,
        mapped: MappedMut<'m, V>,
    ) -> AnyMut<'a, V> {
        match &mut self.inner {
            AnyMutEmplacerState::Direct(_) => {
                let (value, _, _) = mapped.into_parts();
                // SAFETY: In Direct case, 'm == 'a, so the lifetime of the value is valid for 'a
                let value = unsafe { transmute::<&'m mut V, &'a mut V>(value) };
                AnyMut {
                    inner: AnyMutInner::Direct(value),
                }
            }
            AnyMutEmplacerState::Encapsulated(emplacer) => AnyMut {
                inner: AnyMutInner::Encapsulated(emplacer.emplace(mapped)),
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

impl<'a, T: ?Sized> From<Mutable<T>> for AnyMut<'a, T> {
    fn from(value: Mutable<T>) -> Self {
        Self {
            inner: AnyMutInner::Encapsulated(value),
        }
    }
}

enum AnyMutInner<'a, T: 'static + ?Sized> {
    Direct(&'a mut T),
    Encapsulated(Mutable<T>),
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
