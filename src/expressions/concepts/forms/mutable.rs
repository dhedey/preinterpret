use super::*;

pub(crate) type QqqMutable<T> = MutableSubRcRefCell<AnyValue, T>;

#[derive(Copy, Clone)]
pub(crate) struct BeMutable;
impl IsForm for BeMutable {}

impl IsHierarchicalForm for BeMutable {
    type Leaf<'a, T: IsValueLeaf> = QqqMutable<T>;
}

impl IsDynCompatibleForm for BeMutable {
    type DynLeaf<'a, T: 'static + ?Sized> = QqqMutable<T>;
}

impl IsDynMappableForm for BeMutable {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_mut)
    }
}

impl LeafAsRefForm for BeMutable {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

impl LeafAsMutForm for BeMutable {
    fn leaf_as_mut<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T {
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
