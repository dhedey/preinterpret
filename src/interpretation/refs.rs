use std::mem::transmute;

use super::*;

/// A flexible type which can either be a reference to a value of type `T`,
/// or an emplaced reference from a [`SharedReference<T>`].
pub(crate) struct AnyRef<'a, T: ?Sized + 'static> {
    inner: AnyRefInner<'a, T>,
}

impl<'a, T: ?Sized + 'static> AnyRef<'a, T> {
    #[allow(unused)]
    pub(crate) fn map<S: ?Sized>(self, f: impl for<'r> FnOnce(&'r T) -> &'r S) -> AnyRef<'a, S> {
        match self.inner {
            AnyRefInner::Direct(value) => AnyRef {
                inner: AnyRefInner::Direct(f(value)),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.map_legacy(|x| f(x))),
            },
        }
    }

    #[allow(unused)]
    pub(crate) fn map_optional<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r T) -> Option<&'r S>,
    ) -> Option<AnyRef<'a, S>> {
        Some(match self.inner {
            AnyRefInner::Direct(value) => AnyRef {
                inner: AnyRefInner::Direct(f(value)?),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.map_optional_legacy(f)?),
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
    pub(crate) fn emplace<V: 'static + ?Sized>(&mut self, value: &'e V) -> AnyRef<'a, V> {
        unsafe {
            // SAFETY: The lifetime 'e is equal to the &'e content argument in replace
            // So this guarantees that the returned reference is valid as long as the AnyRef exists
            self.emplace_unchecked(value)
        }
    }

    // SAFETY:
    // * The caller must ensure that the value's lifetime is derived from the original content
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &V,
    ) -> AnyRef<'a, V> {
        self.inner
            .take()
            .expect("You can only emplace to create a new AnyRef value once")
            .map(|_|
                // SAFETY: As defined in the rustdoc above
                unsafe { transmute::<&V, &'static V>(value) })
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
    #[allow(unused)]
    pub(crate) fn map<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> &'r mut S,
    ) -> AnyMut<'a, S> {
        match self.inner {
            AnyMutInner::Direct(value) => AnyMut {
                inner: AnyMutInner::Direct(f(value)),
            },
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable.map_legacy(|x| f(x))),
            },
        }
    }

    #[allow(unused)]
    pub(crate) fn map_optional<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> Option<&'r mut S>,
    ) -> Option<AnyMut<'a, S>> {
        Some(match self.inner {
            AnyMutInner::Direct(value) => AnyMut {
                inner: AnyMutInner::Direct(f(value)?),
            },
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable.map_optional_legacy(f)?),
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
    pub(crate) fn emplace<V: 'static + ?Sized>(&mut self, value: &'e mut V) -> AnyMut<'a, V> {
        unsafe {
            // SAFETY: The lifetime 'e is equal to the &'e content argument in replace
            // So this guarantees that the returned reference is valid as long as the AnyMut exists
            self.emplace_unchecked(value)
        }
    }

    // SAFETY:
    // * The caller must ensure that the value's lifetime is derived from the original content
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        value: &mut V,
    ) -> AnyMut<'a, V> {
        self.inner
            .take()
            .expect("You can only emplace to create a new AnyMut value once")
            .map(|_|
                // SAFETY: As defined in the rustdoc above
                unsafe { transmute::<&mut V, &'static mut V>(value) })
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
