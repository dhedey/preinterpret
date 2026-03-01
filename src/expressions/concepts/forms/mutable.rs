use super::*;

impl<X: IsValueContent> IsValueContent for Mutable<X> {
    type Type = X::Type;
    type Form = BeMutable;
}

impl<'a, X: IsValueContent> IntoValueContent<'a> for Mutable<X>
where
    X: 'static,
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
    X::Form: IsHierarchicalForm,
    X::Form: LeafAsMutForm,
{
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self.emplace_map(|inner, emplacer| inner.as_mut_value().into_mutable(emplacer, None))
    }
}

// Note we can't implement this more widely than leaves.
// This is because e.g. Content<AnyType, BeMutable> has Mutable() in its leaves,
// This can't be mapped back to having Mutable(Content<AnyType, BeOwned>).
impl<'a, L: IsValueLeaf> FromValueContent<'a> for Mutable<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeMutable;
impl IsForm for BeMutable {}

impl IsHierarchicalForm for BeMutable {
    type Leaf<'a, T: IsLeafType> = Mutable<T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

impl IsDynCompatibleForm for BeMutable {
    type DynLeaf<'a, D: IsDynType> = Mutable<D::DynContent>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: IsDynType>(
        Spanned(leaf, span): Spanned<Self::Leaf<'a, T>>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D::DynContent>,
    {
        leaf.emplace_map(|content, emplacer| match <T::Leaf>::map_mut(content) {
            Ok(mapped) => {
                // SAFETY: PathExtension is correct for mapping to a dyn type
                let mapped_mut = unsafe {
                    MappedMut::new(mapped, PathExtension::TypeNarrowing(D::type_kind()), span)
                };
                Ok(emplacer.emplace(mapped_mut))
            }
            Err(_) => Err(emplacer.revert()),
        })
    }
}

impl LeafAsRefForm for BeMutable {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl LeafAsMutForm for BeMutable {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeMutable {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn from_argument_value(
        Spanned(value, span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value
            .expect_mutable()
            .emplace_map(|inner, emplacer| inner.as_mut_value().into_mutable(emplacer, Some(span))))
    }
}

// impl MapIntoReturned for BeMutable {
//     fn into_returned_value(
//         content: Content<'static, AnyType, Self>,
//     ) -> ExecutionResult<ReturnedValue> {
//         todo!("Return mutable")
//     }
// }
