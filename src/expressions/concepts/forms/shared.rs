use super::*;

pub(crate) type QqqShared<T> = Actual<'static, T, BeShared>;

pub(crate) struct BeShared;
impl IsForm for BeShared {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;
}

impl IsHierarchicalForm for BeShared {
    type Leaf<'a, T: IsValueLeaf> = SharedSubRcRefCell<Value, T>;
}

impl IsDynCompatibleForm for BeShared {
    type DynLeaf<'a, T: 'static + ?Sized> = SharedSubRcRefCell<Value, T>;
}

impl IsDynMappableForm for BeShared {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_ref)
    }
}
