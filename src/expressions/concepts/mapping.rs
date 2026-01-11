use super::*;

pub(crate) trait LeafMapper<F: IsHierarchicalForm> {
    type OutputForm: IsHierarchicalForm;
    type ShortCircuit<'a>;

    fn map_leaf<'a, L: IsValueLeaf>(
        self,
        leaf: F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsHierarchicalForm>::Leaf<'a, L>, Self::ShortCircuit<'a>>;
}

pub(crate) trait RefLeafMapper<F: IsHierarchicalForm> {
    type Output<'r, 'a: 'r, T: IsHierarchicalType>: MapperOutput<T>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType>;

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r F::Leaf<'a, T::Leaf>,
    ) -> Self::Output<'r, 'a, T>;
}

pub(crate) trait MutLeafMapper<F: IsHierarchicalForm> {
    type Output<'r, 'a: 'r, T: IsHierarchicalType>: MapperOutput<T>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType>;

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r mut F::Leaf<'a, T::Leaf>,
    ) -> Self::Output<'r, 'a, T>;
}

pub(crate) trait MapperOutput<T: IsHierarchicalType> {
    type ParentOutput: MapperOutput<<T as IsChildType>::ParentType>
    where
        T: IsChildType;

    fn to_parent_output(self) -> Self::ParentOutput
    where
        T: IsChildType;
}

pub(crate) struct MapperOutputValue<O>(pub(crate) O);

impl<T: IsHierarchicalType, O> MapperOutput<T> for MapperOutputValue<O> {
    type ParentOutput
        = Self
    where
        T: IsChildType;

    fn to_parent_output(self) -> Self::ParentOutput
    where
        T: IsChildType,
    {
        self
    }
}

pub(crate) struct MapperOutputContent<'a, T: IsHierarchicalType, F: IsHierarchicalForm>(
    pub(crate) Content<'a, T, F>,
);

impl<'a, T: IsLeafType, F: IsHierarchicalForm> MapperOutputContent<'a, T, F> {
    pub(crate) fn from_leaf(leaf: F::Leaf<'a, T::Leaf>) -> Self {
        Self(T::leaf_to_content(leaf))
    }
}

impl<'a, T: IsHierarchicalType, F: IsHierarchicalForm> MapperOutput<T>
    for MapperOutputContent<'a, T, F>
{
    type ParentOutput
        = MapperOutputContent<'a, T::ParentType, F>
    where
        T: IsChildType;

    fn to_parent_output(self) -> Self::ParentOutput
    where
        T: IsChildType,
    {
        MapperOutputContent::<'a, T::ParentType, F>(T::into_parent::<F>(self.0))
    }
}

impl<T: IsHierarchicalType, X: MapperOutput<T>> MapperOutput<T> for ExecutionResult<X> {
    type ParentOutput
        = ExecutionResult<X::ParentOutput>
    where
        T: IsChildType;

    fn to_parent_output(self) -> Self::ParentOutput
    where
        T: IsChildType,
    {
        self.map(|x| x.to_parent_output())
    }
}
