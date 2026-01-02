use super::*;

type QqqAnyRef<'a, T> = Actual<'a, T, BeAnyRef>;

pub(crate) struct BeAnyRef;
impl IsForm for BeAnyRef {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;
}

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

impl MapFromArgument for BeAnyRef {
    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        todo!()
    }
}
