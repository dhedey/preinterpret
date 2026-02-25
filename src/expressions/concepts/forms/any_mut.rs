use super::*;

impl<'a, L: IsValueLeaf> IsValueContent for AnyMut<'a, L> {
    type Type = L::Type;
    type Form = BeAnyMut;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for AnyMut<'a, L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for AnyMut<'a, L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeAnyMut;
impl IsForm for BeAnyMut {}
impl IsHierarchicalForm for BeAnyMut {
    type Leaf<'a, T: IsLeafType> = AnyMut<'a, T::Leaf>;
}

impl IsDynCompatibleForm for BeAnyMut {
    type DynLeaf<'a, D: IsDynType> = AnyMut<'a, D::DynContent>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: IsDynType>(
        Spanned(leaf, span): Spanned<Self::Leaf<'a, T>>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D::DynContent>,
    {
        leaf.emplace_map(|content, emplacer| match <T::Leaf>::map_mut(content) {
            Ok(mapped) => {
                // SAFETY: PathExtension::Tightened correctly describes type narrowing
                let mapped_mut = unsafe {
                    MappedMut::new(mapped, PathExtension::Tightened(D::type_kind()), span)
                };
                Ok(emplacer.emplace(mapped_mut))
            }
            Err(_this) => Err(emplacer.revert()),
        })
    }
}

impl LeafAsRefForm for BeAnyMut {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

impl LeafAsMutForm for BeAnyMut {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf {
        leaf
    }
}

impl MapFromArgument for BeAnyMut {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn from_argument_value(
        Spanned(value, span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value.expect_mutable().emplace_map(|inner, emplacer| {
            inner.as_mut_value().into_mutable_any_mut(emplacer, span)
        }))
    }
}
