use super::*;

// NOTE: These should be moved out soon

pub(crate) struct IterableType;

pub(crate) trait IsIterable: 'static {
    fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue>;
    fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize>;
}

impl IsType for IterableType {
    type Content<'a, F: IsForm> = F::DynLeaf<'a, dyn IsIterable>;

    fn articled_type_name() -> &'static str {
        "an iterable (e.g. array, list, etc.)"
    }
}

impl IsDynType for IterableType {
    type DynContent = dyn IsIterable;
}

impl<'a> IsValueContent<'a> for dyn IsIterable {
    type Type = IterableType;
    type Form = BeOwned;
}

impl IsDynLeaf for dyn IsIterable {}

impl<T: IsHierarchyType, F: IsForm> DowncastFrom<T, F> for IterableType {
    fn downcast_from<'a>(content: <T as IsType>::Content<'a, F>) -> Option<Self::Content<'a, F>> {
        match T::map_with::<'a, F, DynMapper<dyn IsIterable>>(content) {
            Ok(_) => panic!("DynMapper is expected to always short-circuit"),
            Err(dyn_leaf) => dyn_leaf,
        }
    }
}

pub(crate) struct DynMapper<D: ?Sized>(std::marker::PhantomData<D>);

impl<F: IsForm> GeneralMapper<F> for DynMapper<dyn IsIterable> {
    type OutputForm = F; // Unused
    type ShortCircuit<'a> = Option<F::DynLeaf<'a, dyn IsIterable>>;

    fn map_leaf<'a, L: IsValueLeaf, T: IsType>(
        leaf: F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsForm>::Leaf<'a, L>, Self::ShortCircuit<'a>>
// where
        //     T: for<'l> IsType<Content<'l, F> = F::Leaf<'l, L>>,
        //     T: for<'l> IsType<Content<'l, Self::OutputForm> = <Self::OutputForm as IsForm>::Leaf<'l, L>>
    {
        Err(F::leaf_to_dyn(leaf))
    }
}
