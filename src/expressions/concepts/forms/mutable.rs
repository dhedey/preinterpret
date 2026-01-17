use super::*;

pub(crate) type QqqMutable<T> = MutableSubRcRefCell<AnyValue, T>;

impl<L: IsValueLeaf> IsValueContent for QqqMutable<L> {
    type Type = L::Type;
    type Form = BeMutable;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqMutable<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqMutable<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeMutable;
impl IsForm for BeMutable {}

impl IsHierarchicalForm for BeMutable {
    type Leaf<'a, T: IsLeafType> = QqqMutable<T::Leaf>;
}

impl IsDynCompatibleForm for BeMutable {
    type DynLeaf<'a, D: 'static + ?Sized> = QqqMutable<D>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        leaf.map_optional(<T::Leaf>::map_mut)
    }
}

impl LeafAsRefForm for BeMutable {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl LeafAsMutForm for BeMutable {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeMutable {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        // value.expect_mutable()
        todo!()
    }
}
