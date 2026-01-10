use super::*;

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
pub(crate) type QqqLateBound<T> = Actual<'static, T, BeLateBound>;

#[derive(Copy, Clone)]
pub(crate) struct BeLateBound;
impl IsForm for BeLateBound {}

impl IsHierarchicalForm for BeLateBound {
    type Leaf<'a, T: IsValueLeaf> = LateBoundContent<T, T>;
    type LeafLifetimeCapture = UNSAFE_DECLARTION_LeafDoesNotCaptureLifetime;
}

impl IsDynCompatibleForm for BeLateBound {
    type DynLeaf<'a, T: 'static + ?Sized> = LateBoundContent<T, Box<T>>;
}

impl IsDynMappableForm for BeLateBound {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        _leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        // TODO: Add back once we add a map to LateBoundContent
        todo!()
    }
}

pub(crate) enum LateBoundContent<T: 'static + ?Sized, O: 'static> {
    /// An owned value that can be converted to any ownership type
    Owned(LateBoundOwned<O>),
    /// A copy-on-write value that can be converted to an owned value
    CopyOnWrite(CopyOnWriteContent<T, O>),
    /// A mutable reference
    Mutable(MutableSubRcRefCell<AnyValue, T>),
    /// A shared reference where mutable access failed for a specific reason
    Shared(LateBoundShared<T>),
}

pub(crate) struct LateBoundOwned<O: 'static> {
    pub(crate) owned: O,
    pub(crate) is_from_last_use: bool,
}

/// A shared value where mutable access failed for a specific reason
pub(crate) struct LateBoundShared<T: 'static + ?Sized> {
    pub(crate) shared: SharedSubRcRefCell<AnyValue, T>,
    pub(crate) reason_not_mutable: syn::Error,
}
