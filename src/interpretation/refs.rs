use super::*;

/// A flexible type which can either be a reference to a value of type `T`,
/// or an encapsulated reference from a [`Shared<T>`].
pub(crate) struct Ref<'a, T: 'static + ?Sized> {
    inner: RefInner<'a, T>,
}

pub(crate) type SpannedRef<'a, T> = Spanned<Ref<'a, T>>;

impl<'a, T: ?Sized> From<&'a T> for Ref<'a, T> {
    fn from(value: &'a T) -> Self {
        Self {
            inner: RefInner::Direct(value),
        }
    }
}

pub(crate) trait ToSpannedRef<'a> {
    type Target: ?Sized;
    fn spanned(self, source: impl HasSpanRange) -> SpannedRef<'a, Self::Target>;
}

impl<'a, T: ?Sized> ToSpannedRef<'a> for &'a T {
    type Target = T;
    fn spanned(self, source: impl HasSpanRange) -> SpannedRef<'a, Self::Target> {
        SpannedRef {
            value: self.into(),
            span_range: source.span_range(),
        }
    }
}

impl<'a, T: ?Sized> From<Shared<T>> for Ref<'a, T> {
    fn from(value: Shared<T>) -> Self {
        Self {
            inner: RefInner::Encapsulated(value.shared_cell),
        }
    }
}

impl<'a, T: ?Sized> From<Shared<T>> for SpannedRef<'a, T> {
    fn from(value: Shared<T>) -> Self {
        Self {
            value: Ref {
                inner: RefInner::Encapsulated(value.shared_cell),
            },
            span_range: value.span_range,
        }
    }
}

enum RefInner<'a, T: 'static + ?Sized> {
    Direct(&'a T),
    Encapsulated(SharedSubRcRefCell<ExpressionValue, T>),
}

impl<'a, T: 'static + ?Sized> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match &self.inner {
            RefInner::Direct(value) => value,
            RefInner::Encapsulated(shared) => shared,
        }
    }
}

/// A flexible type which can either be a mutable reference to a value of type `T`,
/// or an encapsulated reference from a [`Mutable<T>`].
pub(crate) struct RefMut<'a, T: 'static + ?Sized> {
    inner: RefMutInner<'a, T>,
}

/// A [`SpannedRefMut<T>`] is a more flexible [`Shared<T>`] which can also cheaply host a
/// `(&'a T, SpanRange)`.
pub(crate) type SpannedRefMut<'a, T> = Spanned<RefMut<'a, T>>;

impl<'a, T: ?Sized> From<&'a mut T> for RefMut<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Self {
            inner: RefMutInner::Direct(value),
        }
    }
}

#[allow(unused)]
pub(crate) trait ToSpannedRefMut<'a> {
    type Target: ?Sized;
    fn spanned(self, source: impl HasSpanRange) -> SpannedRefMut<'a, Self::Target>;
}

impl<'a, T: ?Sized + 'static> ToSpannedRefMut<'a> for &'a mut T {
    type Target = T;

    fn spanned(self, source: impl HasSpanRange) -> SpannedRefMut<'a, T> {
        SpannedRefMut {
            value: self.into(),
            span_range: source.span_range(),
        }
    }
}

impl<'a, T: ?Sized> From<Mutable<T>> for RefMut<'a, T> {
    fn from(value: Mutable<T>) -> Self {
        Self {
            inner: RefMutInner::Encapsulated(value.mut_cell),
        }
    }
}

impl<'a, T: ?Sized> From<Mutable<T>> for SpannedRefMut<'a, T> {
    fn from(value: Mutable<T>) -> Self {
        Self {
            value: RefMut {
                inner: RefMutInner::Encapsulated(value.mut_cell),
            },
            span_range: value.span_range,
        }
    }
}

enum RefMutInner<'a, T: 'static + ?Sized> {
    Direct(&'a mut T),
    Encapsulated(MutSubRcRefCell<ExpressionValue, T>),
}

impl<'a, T: 'static + ?Sized> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match &self.inner {
            RefMutInner::Direct(value) => value,
            RefMutInner::Encapsulated(shared) => shared,
        }
    }
}

impl<'a, T: 'static + ?Sized> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        match &mut self.inner {
            RefMutInner::Direct(value) => value,
            RefMutInner::Encapsulated(shared) => &mut *shared,
        }
    }
}
