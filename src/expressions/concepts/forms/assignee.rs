use super::*;

type QqqAssignee<T> = Actual<'static, T, BeAssignee>;

pub(crate) struct BeAssignee;
impl IsForm for BeAssignee {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership =
        ArgumentOwnership::Assignee { auto_create: false };
}

impl IsHierarchicalForm for BeAssignee {
    type Leaf<'a, T: IsValueLeaf> = MutableSubRcRefCell<Value, T>;
}

impl IsDynCompatibleForm for BeAssignee {
    type DynLeaf<'a, T: 'static + ?Sized> = MutableSubRcRefCell<Value, T>;
}

impl IsDynMappableForm for BeAssignee {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_mut)
    }
}
