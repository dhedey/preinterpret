use super::*;

pub(crate) struct BeAnyMut;
impl IsForm for BeAnyMut {
    type Leaf<'a, T: IsValueLeaf> = crate::internal_prelude::AnyMut<'a, T>;
    type DynLeaf<'a, T: 'static + ?Sized> = crate::internal_prelude::AnyMut<'a, T>;

    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        leaf.map_optional(T::map_mut)
    }
}

impl MapFromArgument for BeAnyMut {
    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        todo!()
    }
}
