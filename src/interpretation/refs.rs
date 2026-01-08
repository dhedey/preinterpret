use super::*;

/// A flexible type which can either be a reference to a value of type `T`,
/// or an encapsulated reference from a [`Shared<T>`].
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
                inner: AnyRefInner::Encapsulated(shared.map(|x| f(x))),
            },
        }
    }

    pub(crate) fn map_optional<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r T) -> Option<&'r S>,
    ) -> Option<AnyRef<'a, S>> {
        Some(match self.inner {
            AnyRefInner::Direct(value) => AnyRef {
                inner: AnyRefInner::Direct(f(value)?),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.map_optional(f)?),
            },
        })
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
            inner: AnyRefInner::Encapsulated(value.0),
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
    Encapsulated(SharedSubRcRefCell<AnyValue, T>),
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
/// or an encapsulated reference from a [`Mutable<T>`].
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
                inner: AnyMutInner::Encapsulated(mutable.map(|x| f(x))),
            },
        }
    }

    pub(crate) fn map_optional<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r mut T) -> Option<&'r mut S>,
    ) -> Option<AnyMut<'a, S>> {
        Some(match self.inner {
            AnyMutInner::Direct(value) => AnyMut {
                inner: AnyMutInner::Direct(f(value)?),
            },
            AnyMutInner::Encapsulated(mutable) => AnyMut {
                inner: AnyMutInner::Encapsulated(mutable.map_optional(f)?),
            },
        })
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
            inner: AnyMutInner::Encapsulated(value.0),
        }
    }
}

enum AnyMutInner<'a, T: 'static + ?Sized> {
    Direct(&'a mut T),
    Encapsulated(MutableSubRcRefCell<AnyValue, T>),
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
