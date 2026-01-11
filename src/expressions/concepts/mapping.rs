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
    type OutputForm: IsHierarchicalForm;
    type ShortCircuit<'a>;

    fn map_leaf<'r, 'a: 'r, L: IsValueLeaf>(
        self,
        leaf: &'r F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsHierarchicalForm>::Leaf<'r, L>, Self::ShortCircuit<'a>>;
}

pub(crate) trait MutLeafMapper<F: IsHierarchicalForm> {
    type OutputForm: IsHierarchicalForm;
    type ShortCircuit<'a>;

    fn map_leaf<'r, 'a: 'r, L: IsValueLeaf>(
        self,
        leaf: &'r mut F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsHierarchicalForm>::Leaf<'r, L>, Self::ShortCircuit<'a>>;
}

// pub(crate) trait MutLeafMapper<F: IsHierarchicalForm> {
//     type Output<'r, 'a: 'r>;

//     fn map_leaf<'r, 'a: 'r, T: IsType, L: IsValueLeaf>(
//         self,
//         leaf: &'r mut F::Leaf<'a, L>,
//     ) -> Self::Output<'r, 'a>
//     where
//         for<'x> &'x mut L: IsValueContent<'x, Form = BeMut, Type = T>,
//         T: UpcastTo<AnyType, BeMut>,
//         BeMut: IsFormOf<T, Content<'r> = &'r mut L>,
//     ;
// }

// impl<L: IsValueLeaf> IsMutValueLeaf for L
// where
//     for<'x> &'x mut L: IsValueContent<'x, Form = BeMut>,
//     for<'x> <&'x mut L as IsValueContent<'x>>::Type: UpcastTo<AnyType, BeMut>,
// {
//     type MutType<'a> = <&'a mut L as IsValueContent<'a>>::Type;
// }

// pub(crate) trait MutLeafMapper<F: IsHierarchicalForm> {
//     type Output<'r, 'a: 'r, T>: MapperOutput;

//     fn map_leaf<'r, 'a: 'r, T: IsType, L: IsValueLeaf>(
//         self,
//         leaf: &'r mut F::Leaf<'a, L>,
//     ) -> Self::Output<'r, 'a, T>;
// }

// pub(crate) trait MapperOutput {}

// pub(crate) trait MapperOutputToParent<P: IsHierarchicalType>: MapperOutput {
//     type ParentOutput: MapperOutput;

//     fn to_parent_output(self) -> Self::ParentOutput;
// }

// pub(crate) struct MapperOutputValue<O>(pub(crate) O);

// impl<O> MapperOutput for MapperOutputValue<O> {}

// impl<P: IsHierarchicalType, O> MapperOutputToParent<P> for MapperOutputValue<O> {
//     type ParentOutput = Self;

//     fn to_parent_output(self) -> Self::ParentOutput {
//         self
//     }
// }

// pub(crate) struct MapperOutputContent<'a, T: IsType, F: IsFormOf<T>>(pub(crate) F::Content<'a>);

// impl<'a, T: IsType, F: IsFormOf<T>> MapperOutput for MapperOutputContent<'a, T, F> {}

// impl<'a, P: IsHierarchicalType, C: IsChildType<ParentType = P>, F: IsHierarchicalForm + IsFormOf<C> + IsFormOf<P>>
//     MapperOutputToParent<P> for MapperOutputContent<'a, C, F>
// where
//     for<'l> C: IsHierarchicalType<Content<'l, F> = <F as IsFormOf<C>>::Content<'l>>,
//     for<'l> P: IsHierarchicalType<Content<'l, F> = <F as IsFormOf<P>>::Content<'l>>,
// {
//     type ParentOutput = MapperOutputContent<'a, P, F>;

//     fn to_parent_output(self) -> Self::ParentOutput {
//         MapperOutputContent::<'a, P, F>(C::into_parent::<F>(self.0))
//     }
// }
