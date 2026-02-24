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
}

impl IsDynCompatibleForm for BeAnyRef {
    type DynLeaf<'a, D: IsDynType> = crate::internal_prelude::AnyRef<'a, D::DynContent>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: IsDynType>(
        leaf: Self::Leaf<'a, T>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D::DynContent>,
    {
        leaf.emplace_map(|content, emplacer| {
            match <T::Leaf>::map_ref(content) {
                Ok(mapped) => Ok(unsafe {
                    // TODO[references]: Pass a span here by propagating from DynResolveFrom
                    emplacer.emplace(mapped, PathExtension::Tightened(D::type_kind()), None)
                }),
                Err(this) => Err(emplacer.revert()),
            }
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
        value: ArgumentValue,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value
            .expect_shared()
            .emplace_map(|inner, emplacer| inner.as_ref_value().into_shared_any_ref(emplacer)))
    }
}
