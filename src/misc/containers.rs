use super::*;

/// Similar to Cow, but it doesn't require the ref
/// to be convertible into Owned
pub(crate) enum OwnedOrRef<'a, T> {
    Owned(T),
    Ref(&'a T),
}

impl<T> Deref for OwnedOrRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            OwnedOrRef::Owned(value) => value,
            OwnedOrRef::Ref(value) => *value,
        }
    }
}