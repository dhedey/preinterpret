use super::*;

type Owned<T> = Actual<'static, T, BeOwned>;

pub(crate) struct BeOwned;
impl IsForm for BeOwned {
    type Leaf<'a, T: IsValueLeaf> = T;
    type DynLeaf<'a, T: 'static + ?Sized> = Box<T>;
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        T::map_boxed(Box::new(leaf))
    }
}

impl MapFromArgument for BeOwned {
    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        // value.expect_owned()
        todo!()
    }
}
