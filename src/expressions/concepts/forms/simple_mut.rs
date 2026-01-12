use super::*;

pub(crate) type QqqMut<'a, T> = Content<'a, T, BeMut>;

/// It can't be an argument because arguments must be owned in some way;
/// so that the drop glue can work properly (because they may come from
/// e.g. a reference counted Shared handle)
#[derive(Copy, Clone)]
pub(crate) struct BeMut;
impl IsForm for BeMut {}

impl IsHierarchicalForm for BeMut {
    type Leaf<'a, T: IsLeafType> = &'a mut T::Leaf;
}

impl IsDynCompatibleForm for BeMut {
    type DynLeaf<'a, D: 'static + ?Sized> = &'a mut D;
}

impl IsDynMappableForm for BeMut {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        <T::Leaf>::map_mut(leaf)
    }
}

impl LeafAsRefForm for BeMut {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}
