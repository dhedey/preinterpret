use super::*;

type Assignee<T> = Actual<'static, T, BeAssignee>;

pub(crate) struct BeAssignee;
impl IsForm for BeAssignee {
    type Leaf<'a, T: IsValueLeaf> = MutableSubRcRefCell<Value, T>;
    type DynLeaf<'a, T: 'static + ?Sized> = MutableSubRcRefCell<Value, T>;
    const ARGUMENT_OWNERSHIP: ArgumentOwnership =
        ArgumentOwnership::Assignee { auto_create: false };

    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_mut)
    }
}
