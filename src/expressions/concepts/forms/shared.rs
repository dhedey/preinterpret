use super::*;

pub(crate) type QqqShared<T> = SharedSubRcRefCell<AnyValue, T>;

#[derive(Copy, Clone)]
pub(crate) struct BeShared;
impl IsForm for BeShared {}

impl IsHierarchicalForm for BeShared {
    type Leaf<'a, T: IsValueLeaf> = QqqShared<T>;
    type LeafLifetimeCapture = UNSAFE_DECLARTION_LeafDoesNotCaptureLifetime;
}

impl IsDynCompatibleForm for BeShared {
    type DynLeaf<'a, T: 'static + ?Sized> = QqqShared<T>;
}

impl IsDynMappableForm for BeShared {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_ref)
    }
}

impl LeafAsRefForm for BeShared {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

impl MapFromArgument for BeShared {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, AnyType, Self>> {
        // value.expect_shared()
        todo!()
    }
}
