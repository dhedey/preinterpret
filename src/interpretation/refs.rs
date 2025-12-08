use super::*;

/// A flexible type which can either be a reference to a value of type `T`,
/// or an encapsulated reference from a [`Shared<T>`].
pub(crate) struct AnyRef<'a, T: ?Sized + 'static> {
    inner: AnyRefInner<'a, T>,
}

impl<'a, T: ?Sized + 'static> AnyRef<'a, T> {
    pub(crate) fn map_optional<S: ?Sized>(
        self,
        f: impl for<'r> FnOnce(&'r T) -> Option<&'r S>,
    ) -> Option<AnyRef<'a, S>> {
        Some(match self.inner {
            AnyRefInner::Direct(value) => AnyRef {
                inner: AnyRefInner::Direct(f(value)?),
            },
            AnyRefInner::Encapsulated(shared) => AnyRef {
                inner: AnyRefInner::Encapsulated(shared.try_map(|x| f(x).ok_or(())).ok()?),
            },
        })
    }
}

pub(crate) type SpannedAnyRef<'a, T> = Spanned<AnyRef<'a, T>>;

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
    fn into_spanned_ref(self, source: impl HasSpanRange) -> SpannedAnyRef<'a, Self::Target>;
}

impl<'a, T: ?Sized> ToSpannedRef<'a> for &'a T {
    type Target = T;
    fn into_ref(self) -> AnyRef<'a, Self::Target> {
        self.into()
    }
    fn into_spanned_ref(self, source: impl HasSpanRange) -> SpannedAnyRef<'a, Self::Target> {
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

impl<'a, T: ?Sized> From<Shared<T>> for SpannedAnyRef<'a, T> {
    fn from(value: Shared<T>) -> Self {
        // TODO: Track proper span through shared value
        let span_range = Span::call_site().span_range();
        Spanned(
            AnyRef {
                inner: AnyRefInner::Encapsulated(value.0),
            },
            span_range,
        )
    }
}

enum AnyRefInner<'a, T: 'static + ?Sized> {
    Direct(&'a T),
    Encapsulated(SharedSubRcRefCell<Value, T>),
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
pub(crate) struct AnyRefMut<'a, T: 'static + ?Sized> {
    inner: AnyRefMutInner<'a, T>,
}

/// A [`SpannedRefMut<T>`] is a more flexible [`Shared<T>`] which can also cheaply host a
/// `(&'a T, SpanRange)`.
pub(crate) type SpannedAnyRefMut<'a, T> = Spanned<AnyRefMut<'a, T>>;

impl<'a, T: ?Sized> From<&'a mut T> for AnyRefMut<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Self {
            inner: AnyRefMutInner::Direct(value),
        }
    }
}

#[allow(unused)]
pub(crate) trait IntoRefMut<'a> {
    type Target: ?Sized;
    fn into_ref_mut(self) -> AnyRefMut<'a, Self::Target>;
    fn into_spanned_ref_mut(self, source: impl HasSpanRange) -> SpannedAnyRefMut<'a, Self::Target>;
}

impl<'a, T: ?Sized + 'static> IntoRefMut<'a> for &'a mut T {
    type Target = T;

    fn into_spanned_ref_mut(self, source: impl HasSpanRange) -> SpannedAnyRefMut<'a, T> {
        self.into_ref_mut().spanned(source)
    }

    fn into_ref_mut(self) -> AnyRefMut<'a, Self::Target> {
        self.into()
    }
}

impl<'a, T: ?Sized> From<Mutable<T>> for AnyRefMut<'a, T> {
    fn from(value: Mutable<T>) -> Self {
        Self {
            inner: AnyRefMutInner::Encapsulated(value.0),
        }
    }
}

impl<'a, T: ?Sized> From<Mutable<T>> for SpannedAnyRefMut<'a, T> {
    fn from(value: Mutable<T>) -> Self {
        // TODO: Track proper span through mutable value
        let span_range = Span::call_site().span_range();
        Spanned(
            AnyRefMut {
                inner: AnyRefMutInner::Encapsulated(value.0),
            },
            span_range,
        )
    }
}

enum AnyRefMutInner<'a, T: 'static + ?Sized> {
    Direct(&'a mut T),
    Encapsulated(MutSubRcRefCell<Value, T>),
}

impl<'a, T: 'static + ?Sized> Deref for AnyRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match &self.inner {
            AnyRefMutInner::Direct(value) => value,
            AnyRefMutInner::Encapsulated(shared) => shared,
        }
    }
}

impl<'a, T: 'static + ?Sized> DerefMut for AnyRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        match &mut self.inner {
            AnyRefMutInner::Direct(value) => value,
            AnyRefMutInner::Encapsulated(shared) => &mut *shared,
        }
    }
}
