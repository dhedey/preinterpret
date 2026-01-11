use super::*;

type QqqAnyRef<'a, T> = Content<'a, T, BeAnyRef>;

#[derive(Copy, Clone)]
pub(crate) struct BeAnyRef;
impl IsForm for BeAnyRef {}

impl IsHierarchicalForm for BeAnyRef {
    type Leaf<'a, T: IsLeafType> = crate::internal_prelude::AnyRef<'a, T::Leaf>;
}

impl IsDynCompatibleForm for BeAnyRef {
    type DynLeaf<'a, D: 'static + ?Sized> = crate::internal_prelude::AnyRef<'a, D>;
}

impl IsDynMappableForm for BeAnyRef {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        leaf.map_optional(<T::Leaf>::map_ref)
    }
}

impl LeafAsRefForm for BeAnyRef {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeAnyRef {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        todo!()
        // value.expect_shared().as_any_ref()
    }
}
