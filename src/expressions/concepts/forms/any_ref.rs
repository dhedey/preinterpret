use super::*;

impl<'a, L: IsValueLeaf> IsValueContent for AnyRef<'a, L> {
    type Type = L::Type;
    type Form = BeAnyRef;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for AnyRef<'a, L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for AnyRef<'a, L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeAnyRef;
impl IsForm for BeAnyRef {}

impl IsHierarchicalForm for BeAnyRef {
    type Leaf<'a, T: IsLeafType> = crate::internal_prelude::AnyRef<'a, T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

impl IsDynCompatibleForm for BeAnyRef {
    type DynLeaf<'a, D: IsDynType> = crate::internal_prelude::AnyRef<'a, D::DynContent>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: IsDynType>(
        Spanned(leaf, span): Spanned<Self::Leaf<'a, T>>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D::DynContent>,
    {
        leaf.emplace_map(|content, emplacer| match <T::Leaf>::map_ref(content) {
            Ok(mapped) => {
                // SAFETY: PathExtension::Tightened correctly describes type narrowing
                let mapped_ref = unsafe {
                    MappedRef::new(mapped, PathExtension::Tightened(D::type_kind()), span)
                };
                Ok(emplacer.emplace(mapped_ref))
            }
            Err(_this) => Err(emplacer.revert()),
        })
    }
}

impl LeafAsRefForm for BeAnyRef {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeAnyRef {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument_value(
        Spanned(value, span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value.expect_shared().emplace_map(|inner, emplacer| {
            inner
                .as_ref_value()
                .into_shared_any_ref(emplacer, Some(span))
        }))
    }
}
