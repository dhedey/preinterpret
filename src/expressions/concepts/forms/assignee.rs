use super::*;

pub(crate) struct QqqAssignee<T: 'static + ?Sized>(pub(crate) MutableSubRcRefCell<AnyValue, T>);

impl<L: IsValueLeaf> IsValueContent for QqqAssignee<L> {
    type Type = L::Type;
    type Form = BeAssignee;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqAssignee<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        <L::LeafType as IsLeafType>::leaf_to_content(self)
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqAssignee<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        <L::LeafType as IsLeafType>::content_to_leaf(content)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeAssignee;
impl IsForm for BeAssignee {}

impl IsHierarchicalForm for BeAssignee {
    type Leaf<'a, T: IsLeafType> = QqqAssignee<T::Leaf>;
}

impl IsDynCompatibleForm for BeAssignee {
    type DynLeaf<'a, D: 'static + ?Sized> = QqqAssignee<D>;
}

impl IsDynMappableForm for BeAssignee {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        leaf.0.map_optional(<T::Leaf>::map_mut).map(QqqAssignee)
    }
}

impl LeafAsRefForm for BeAssignee {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        &leaf.0
    }
}

impl LeafAsMutForm for BeAssignee {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf {
        &mut leaf.0
    }
}

impl MapFromArgument for BeAssignee {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership =
        ArgumentOwnership::Assignee { auto_create: false };

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        todo!()
        // value.expect_assignee()
    }
}
