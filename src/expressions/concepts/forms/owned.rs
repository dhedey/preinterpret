use super::*;

/// Just [`T`]! This exists simply to be a name for symmetry with e.g. Shared<T> or Mutable<T>.
pub(crate) type Owned<T> = T;

/// Represents floating owned values.
///
/// If you need span information, wrap with `Spanned<AnyValue>`. For example, with `x.y[4]`, this would capture both:
/// * The output owned value
/// * The lexical span of the tokens `x.y[4]`
#[derive(Copy, Clone)]
pub(crate) struct BeOwned;
impl IsForm for BeOwned {}

impl IsHierarchicalForm for BeOwned {
    type Leaf<'a, T: IsLeafType> = T::Leaf;
}

impl IsDynCompatibleForm for BeOwned {
    type DynLeaf<'a, D: 'static + ?Sized> = Box<D>;
}

impl IsDynMappableForm for BeOwned {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        <T::Leaf>::map_boxed(Box::new(leaf))
    }
}

impl LeafAsRefForm for BeOwned {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl LeafAsMutForm for BeOwned {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeOwned {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        Ok(value.expect_owned())
    }
}

pub(crate) struct ToOwnedInfallibleMapper;

impl<F: LeafAsRefForm> RefLeafMapper<F> for ToOwnedInfallibleMapper {
    type Output<'r, 'a: 'r, T: IsHierarchicalType> = Content<'static, T, BeOwned>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType> {
        T::into_parent(output)
    }

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r F::Leaf<'a, T>,
    ) -> Self::Output<'r, 'a, T> {
        T::leaf_to_content(F::leaf_clone_to_owned_infallible(leaf))
    }
}

pub(crate) struct ToOwnedTransparentlyMapper {
    pub(crate) span_range: SpanRange,
}

impl<F: LeafAsRefForm> RefLeafMapper<F> for ToOwnedTransparentlyMapper {
    type Output<'r, 'a: 'r, T: IsHierarchicalType> = ExecutionResult<Content<'static, T, BeOwned>>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType> {
        Ok(T::into_parent(output?))
    }

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r <F as IsHierarchicalForm>::Leaf<'a, T>,
    ) -> Self::Output<'r, 'a, T> {
        Ok(T::leaf_to_content(F::leaf_clone_to_owned_transparently(
            leaf,
            self.span_range,
        )?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_resolve_owned() {
        let owned_value: Owned<_> = 42u64;
        let resolved = owned_value
            .spanned(Span::call_site().span_range())
            .downcast_resolve::<u64>("My value")
            .unwrap();
        assert_eq!(resolved, 42u64);
    }

    #[test]
    fn can_as_ref_owned() {
        let owned_value = 42u64;
        let as_ref: QqqRef<U64Type> = owned_value.as_ref_value();
        assert_eq!(*as_ref, 42u64);
    }

    #[test]
    fn can_as_mut_owned() {
        let mut owned_value = 42u64;
        *owned_value.as_mut_value() = 41u64;
        assert_eq!(owned_value, 41u64);
    }
}
