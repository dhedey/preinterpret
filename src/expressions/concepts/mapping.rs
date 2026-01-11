use super::*;

pub(crate) trait LeafMapper<F: IsHierarchicalForm> {
    type Output<'a, T: IsHierarchicalType>;

    fn to_parent_output<'a, T: IsChildType>(
        output: Self::Output<'a, T>,
    ) -> Self::Output<'a, T::ParentType>;

    fn map_leaf<'a, T: IsLeafType>(self, leaf: F::Leaf<'a, T::Leaf>) -> Self::Output<'a, T>;
}

pub(crate) trait RefLeafMapper<F: IsHierarchicalForm> {
    type Output<'r, 'a: 'r, T: IsHierarchicalType>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType>;

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r F::Leaf<'a, T::Leaf>,
    ) -> Self::Output<'r, 'a, T>;
}

pub(crate) trait MutLeafMapper<F: IsHierarchicalForm> {
    type Output<'r, 'a: 'r, T: IsHierarchicalType>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType>;

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r mut F::Leaf<'a, T::Leaf>,
    ) -> Self::Output<'r, 'a, T>;
}
