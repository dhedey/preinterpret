use super::*;

pub(crate) type QqqShared<T> = SharedSubRcRefCell<AnyValue, T>;

#[derive(Copy, Clone)]
pub(crate) struct BeShared;
impl IsForm for BeShared {}

impl IsHierarchicalForm for BeShared {
    type Leaf<'a, T: IsLeafType> = QqqShared<T::Leaf>;
}

impl IsDynCompatibleForm for BeShared {
    type DynLeaf<'a, D: 'static + ?Sized> = QqqShared<D>;
}

impl IsDynMappableForm for BeShared {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        leaf.map_optional(<T::Leaf>::map_ref)
    }
}

impl LeafAsRefForm for BeShared {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeShared {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        // value.expect_shared()
        todo!()
    }
}
