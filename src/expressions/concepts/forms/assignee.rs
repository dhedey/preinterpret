use super::*;

pub(crate) struct QqqAssignee<T: 'static + ?Sized>(pub(crate) MutableSubRcRefCell<AnyValue, T>);

#[derive(Copy, Clone)]
pub(crate) struct BeAssignee;
impl IsForm for BeAssignee {}

impl IsHierarchicalForm for BeAssignee {
    type Leaf<'a, T: IsValueLeaf> = QqqAssignee<T>;
}

impl IsDynCompatibleForm for BeAssignee {
    type DynLeaf<'a, T: 'static + ?Sized> = QqqAssignee<T>;
}

impl IsDynMappableForm for BeAssignee {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.0.map_optional(T::map_mut).map(QqqAssignee)
    }
}

impl LeafAsRefForm for BeAssignee {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        &leaf.0
    }
}

impl LeafAsMutForm for BeAssignee {
    fn leaf_as_mut<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T {
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
