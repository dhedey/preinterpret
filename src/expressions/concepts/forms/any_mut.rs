use super::*;

#[derive(Copy, Clone)]
pub(crate) struct BeAnyMut;
impl IsForm for BeAnyMut {}
impl IsHierarchicalForm for BeAnyMut {
    type Leaf<'a, T: IsValueLeaf> = crate::internal_prelude::AnyMut<'a, T>;
}

impl IsDynCompatibleForm for BeAnyMut {
    type DynLeaf<'a, T: 'static + ?Sized> = crate::internal_prelude::AnyMut<'a, T>;
}

impl IsDynMappableForm for BeAnyMut {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_mut)
    }
}

impl LeafAsRefForm for BeAnyMut {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

impl LeafAsMutForm for BeAnyMut {
    fn leaf_as_mut<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T {
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
