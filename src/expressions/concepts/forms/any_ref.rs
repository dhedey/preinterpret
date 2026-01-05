use super::*;

type QqqAnyRef<'a, T> = Actual<'a, T, BeAnyRef>;

#[derive(Copy, Clone)]
pub(crate) struct BeAnyRef;
impl IsForm for BeAnyRef {}

impl IsHierarchicalForm for BeAnyRef {
    type Leaf<'a, T: IsValueLeaf> = crate::internal_prelude::AnyRef<'a, T>;
}

impl IsDynCompatibleForm for BeAnyRef {
    type DynLeaf<'a, T: 'static + ?Sized> = crate::internal_prelude::AnyRef<'a, T>;
}

impl IsDynMappableForm for BeAnyRef {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_ref)
    }
}

impl LeafAsRefForm for BeAnyRef {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

impl MapFromArgument for BeAnyRef {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        todo!()
        // value.expect_shared().as_any_ref()
    }
}
