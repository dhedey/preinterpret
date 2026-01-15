use super::*;

impl<L: IsValueLeaf> IsValueContent for &L {
    type Type = L::Type;
    type Form = BeRef;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for &'a L {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for &'a L {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

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
