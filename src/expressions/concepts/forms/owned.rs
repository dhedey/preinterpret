use super::*;

pub(crate) type QqqOwned<T> = Actual<'static, T, BeOwned>;

pub(crate) type QqqOwnedValue = Owned<ValueType>;

/// Represents floating owned values.
///
/// If you need span information, wrap with `Spanned<OwnedValue>`. For example, with `x.y[4]`, this would capture both:
/// * The output owned value
/// * The lexical span of the tokens `x.y[4]`
#[derive(Copy, Clone)]
pub(crate) struct BeOwned;
impl IsForm for BeOwned {}

impl IsHierarchicalForm for BeOwned {
    type Leaf<'a, T: IsValueLeaf> = T;
}

impl IsDynCompatibleForm for BeOwned {
    type DynLeaf<'a, T: 'static + ?Sized> = Box<T>;
}

impl IsDynMappableForm for BeOwned {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        T::map_boxed(Box::new(leaf))
    }
}

impl LeafAsRefForm for BeOwned {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

impl LeafAsMutForm for BeOwned {
    fn leaf_as_mut<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T {
        leaf
    }
}

impl MapFromArgument for BeOwned {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        Ok(value.expect_owned().0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_resolve_owned() {
        let owned_value: QqqOwned<U64Type> = 42u64;
        let resolved = owned_value
            .spanned(Span::call_site().span_range())
            .downcast_resolve::<u64>("My value")
            .unwrap();
        assert_eq!(resolved, 42u64);
    }

    #[test]
    fn can_as_ref_owned() {
        let owned_value: QqqOwned<U64Type> = 42u64;
        let as_ref: QqqRef<U64Type> = owned_value.as_ref_value();
        assert_eq!(*as_ref, 42u64);
    }

    #[test]
    fn can_as_mut_owned() {
        let mut owned_value: QqqOwned<U64Type> = 42u64;
        *owned_value.as_mut_value() = 41u64;
        assert_eq!(owned_value, 41u64);
    }
}
