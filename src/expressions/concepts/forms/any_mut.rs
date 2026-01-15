use super::*;

impl<'a, L: IsValueLeaf> IsValueContent for AnyMut<'a, L> {
    type Type = L::Type;
    type Form = BeAnyMut;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for AnyMut<'a, L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        <L::LeafType as IsLeafType>::leaf_to_content(self)
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for AnyMut<'a, L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        <L::LeafType as IsLeafType>::content_to_leaf(content)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeAnyMut;
impl IsForm for BeAnyMut {}
impl IsHierarchicalForm for BeAnyMut {
    type Leaf<'a, T: IsLeafType> = AnyMut<'a, T::Leaf>;
}

impl IsDynCompatibleForm for BeAnyMut {
    type DynLeaf<'a, D: 'static + ?Sized> = AnyMut<'a, D>;
}

impl IsDynMappableForm for BeAnyMut {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        leaf.map_optional(<T::Leaf>::map_mut)
    }
}

impl LeafAsRefForm for BeAnyMut {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl LeafAsMutForm for BeAnyMut {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeAnyMut {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        todo!()
        // value.expect_mutable().as_any_mut()
    }
}
