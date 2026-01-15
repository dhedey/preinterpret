use super::*;

impl<L: IsValueLeaf> IsValueContent for &mut L {
    type Type = L::Type;
    type Form = BeMut;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for &'a mut L {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        <L::LeafType as IsLeafType>::leaf_to_content(self)
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for &'a mut L {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        <L::LeafType as IsLeafType>::content_to_leaf(content)
    }
}

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
