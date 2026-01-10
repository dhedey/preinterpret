use super::*;

pub(crate) type QqqMut<'a, T> = Actual<'a, T, BeMut>;

/// It can't be an argument because arguments must be owned in some way;
/// so that the drop glue can work properly (because they may come from
/// e.g. a reference counted Shared handle)
#[derive(Copy, Clone)]
pub(crate) struct BeMut;
impl IsForm for BeMut {}

impl IsHierarchicalForm for BeMut {
    type Leaf<'a, T: IsValueLeaf> = &'a mut T;
    type LeafLifetimeCapture = LeafCapturesLifetime;
}

impl IsDynCompatibleForm for BeMut {
    type DynLeaf<'a, T: 'static + ?Sized> = &'a mut T;
}

impl IsDynMappableForm for BeMut {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        T::map_mut(leaf)
    }
}

impl LeafAsRefForm for BeMut {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

pub(crate) struct ToMutMapper;

impl<F: LeafAsMutForm> MutLeafMapper<F> for ToMutMapper {
    type OutputForm = BeMut;
    type ShortCircuit<'a> = Infallible;

    fn map_leaf<'r, 'a: 'r, L: IsValueLeaf>(
        self,
        leaf: &'r mut F::Leaf<'a, L>,
    ) -> Result<&'r mut L, Infallible> {
        Ok(F::leaf_as_mut(leaf))
    }
}
