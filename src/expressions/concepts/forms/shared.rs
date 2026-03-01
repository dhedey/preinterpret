use super::*;

impl<X: IsValueContent> IsValueContent for Shared<X> {
    type Type = X::Type;
    type Form = BeShared;
}

impl<'a, X: IsValueContent> IntoValueContent<'a> for Shared<X>
where
    X: 'static,
    X: IsSelfValueContent<'static>,
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
    X::Form: IsHierarchicalForm,
    X::Form: LeafAsRefForm,
{
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self.emplace_map(|inner, emplacer| inner.as_ref_value().into_shared(emplacer, None))
    }
}

// Note we can't implement this more widely than leaves.
// This is because e.g. Content<AnyType, BeShared> has Shared() in its leaves,
// This can't be mapped back to having Shared(Content<AnyType, BeOwned>).
impl<'a, L: IsValueLeaf> FromValueContent<'a> for Shared<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeShared;
impl IsForm for BeShared {}

impl IsHierarchicalForm for BeShared {
    type Leaf<'a, T: IsLeafType> = Shared<T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

impl IsDynCompatibleForm for BeShared {
    type DynLeaf<'a, D: IsDynType> = Shared<D::DynContent>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: IsDynType>(
        Spanned(leaf, span): Spanned<Self::Leaf<'a, T>>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D::DynContent>,
    {
        leaf.emplace_map(|content, emplacer| match <T::Leaf>::map_ref(content) {
            Ok(mapped) => {
                // SAFETY: PathExtension is correct for mapping to a dyn type
                let mapped_ref = unsafe {
                    MappedRef::new(mapped, PathExtension::TypeNarrowing(D::type_kind()), span)
                };
                Ok(emplacer.emplace(mapped_ref))
            }
            Err(_) => Err(emplacer.revert()),
        })
    }
}

impl LeafAsRefForm for BeShared {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeShared {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument_value(
        Spanned(value, span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value
            .expect_shared()
            .emplace_map(|inner, emplacer| inner.as_ref_value().into_shared(emplacer, Some(span))))
    }
}

// impl MapIntoReturned for BeShared {
//     fn into_returned_value(
//         content: Content<'static, AnyType, Self>,
//     ) -> FunctionResult<ReturnedValue> {
//         todo!("Return shared")
//     }
// }
