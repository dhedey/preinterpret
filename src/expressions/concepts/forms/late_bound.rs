use super::*;

pub(crate) enum QqqLateBound<T: 'static> {
    /// An owned value that can be converted to any ownership type
    Owned(QqqLateBoundOwned<T>),
    /// A copy-on-write value that can be converted to an owned value
    CopyOnWrite(QqqCopyOnWrite<T>),
    /// A mutable reference
    Mutable(QqqMutable<T>),
    /// A shared reference where mutable access failed for a specific reason
    Shared(QqqLateBoundShared<T>),
}

impl<L: IsValueLeaf> IsValueContent for QqqLateBound<L> {
    type Type = L::Type;
    type Form = BeLateBound;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqLateBound<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqLateBound<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

/// Universal value type that can resolve to any concrete ownership type.
///
/// Sometimes, a value can be accessed, but we don't yet know *how* we need to access it.
/// In this case, we attempt to load it with the most powerful access we can have, and convert it later.
///
/// ## Example of requirement
/// For example, if we have `x[a].method()`, we first need to resolve the type of `x[a]` to know
/// whether the method needs `x[a]` to be a shared reference, mutable reference or an owned value.
///
/// So instead, we take the most powerful access we can have for `x[a]`, and convert it later.
#[derive(Copy, Clone)]
pub(crate) struct BeLateBound;
impl IsForm for BeLateBound {}

impl IsHierarchicalForm for BeLateBound {
    type Leaf<'a, T: IsLeafType> = QqqLateBound<T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

pub(crate) struct QqqLateBoundOwned<O: 'static> {
    pub(crate) owned: O,
    pub(crate) is_from_last_use: bool,
}

/// A shared value where mutable access failed for a specific reason
pub(crate) struct QqqLateBoundShared<T: 'static> {
    pub(crate) shared: QqqShared<T>,
    pub(crate) reason_not_mutable: syn::Error,
}
