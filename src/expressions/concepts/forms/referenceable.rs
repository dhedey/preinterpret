use super::*;

type Referenceable<T> = Actual<'static, T, BeReferenceable>;

// Roughly equivalent to an owned, but wrapped so that it can be turned into a Shared/Mutable easily.
pub(crate) struct BeReferenceable;
impl IsForm for BeReferenceable {
    type Leaf<'a, T: IsValueLeaf> = Rc<RefCell<T>>;
    type DynLeaf<'a, T: 'static + ?Sized> = Rc<RefCell<T>>;

    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        _leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        // Can't map Rc<RefCell<T>> to Rc<RefCell<D>> directly
        panic!("Casting to dyn is not supported for Referenceable form")
    }
}

impl MapFromArgument for BeReferenceable {
    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        // Rc::new(RefCell::new(value.expect_owned()))
        todo!()
    }
}

impl<'a, T: IsHierarchyType> Actual<'a, T, BeOwned> {
    pub(crate) fn into_referencable(self) -> Actual<'a, T, BeReferenceable> {
        match self.map_with::<OwnedToReferencableMapper>() {
            Ok(output) => output,
        }
    }
}

pub(crate) struct OwnedToReferencableMapper;

impl GeneralMapper<BeOwned> for OwnedToReferencableMapper {
    type OutputForm = BeReferenceable;
    type ShortCircuit<'a> = std::convert::Infallible;

    fn map_leaf<
        'a,
        L: IsValueLeaf,
        T: for<'l> IsType<Content<'l, BeOwned> = <BeOwned as IsForm>::Leaf<'l, L>>,
    >(
        leaf: <BeOwned as IsForm>::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsForm>::Leaf<'a, L>, Self::ShortCircuit<'a>> {
        Ok(Rc::new(RefCell::new(leaf)))
    }
}
