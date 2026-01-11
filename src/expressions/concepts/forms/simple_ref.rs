use super::*;

pub(crate) type QqqRef<'a, T> = Content<'a, T, BeRef>;

/// It can't be an argument because arguments must be owned in some way;
/// so that the drop glue can work properly (because they may come from
/// e.g. a reference counted Shared handle)
#[derive(Copy, Clone)]
pub(crate) struct BeRef;
impl IsForm for BeRef {}

impl IsHierarchicalForm for BeRef {
    type Leaf<'a, T: IsValueLeaf> = &'a T;
}

impl IsDynCompatibleForm for BeRef {
    type DynLeaf<'a, T: 'static + ?Sized> = &'a T;
}

impl IsDynMappableForm for BeRef {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        T::map_ref(leaf)
    }
}

impl LeafAsRefForm for BeRef {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

pub(crate) struct ToRefMapper;

impl<F: LeafAsRefForm> RefLeafMapper<F> for ToRefMapper {
    type Output<'r, 'a: 'r, T: IsHierarchicalType> = Content<'r, T, BeRef>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType> {
        T::into_parent(output)
    }

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r <F as IsHierarchicalForm>::Leaf<'a, T::Leaf>,
    ) -> Self::Output<'r, 'a, T> {
        T::leaf_to_content(F::leaf_as_ref(leaf))
    }
}
