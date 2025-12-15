use super::*;

type Mutable<T> = Actual<'static, T, BeMutable>;

pub(crate) struct BeMutable;
impl IsForm for BeMutable {
    type Leaf<'a, T: IsValueLeaf> = MutableSubRcRefCell<Value, T>;
    type DynLeaf<'a, T: 'static + ?Sized> = MutableSubRcRefCell<Value, T>;
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_mut)
    }
}
