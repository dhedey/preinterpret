use super::*;

pub(crate) type QqqShared<T> = SharedSubRcRefCell<AnyValue, T>;

impl<L: IsValueLeaf> IsValueContent for QqqShared<L> {
    type Type = L::Type;
    type Form = BeShared;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqShared<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        <L::LeafType as IsLeafType>::leaf_to_content(self)
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqShared<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        <L::LeafType as IsLeafType>::content_to_leaf(content)
    }
}

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
