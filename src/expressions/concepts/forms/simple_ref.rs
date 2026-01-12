use super::*;

pub(crate) type QqqRef<'a, T> = Content<'a, T, BeRef>;

/// It can't be an argument because arguments must be owned in some way;
/// so that the drop glue can work properly (because they may come from
/// e.g. a reference counted Shared handle)
#[derive(Copy, Clone)]
pub(crate) struct BeRef;
impl IsForm for BeRef {}

impl IsHierarchicalForm for BeRef {
    type Leaf<'a, T: IsLeafType> = &'a T::Leaf;
}

impl IsDynCompatibleForm for BeRef {
    type DynLeaf<'a, D: 'static + ?Sized> = &'a D;
}

impl IsDynMappableForm for BeRef {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        <T::Leaf>::map_ref(leaf)
    }
}

impl LeafAsRefForm for BeRef {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}
