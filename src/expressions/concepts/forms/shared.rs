use super::*;

pub(crate) type QqqShared<T> = SharedReference<T>;

impl<L: IsValueLeaf> IsValueContent for QqqShared<L> {
    type Type = L::Type;
    type Form = BeShared;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqShared<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqShared<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeShared;
impl IsForm for BeShared {}

impl IsHierarchicalForm for BeShared {
    type Leaf<'a, T: IsLeafType> = QqqShared<T::Leaf>;
}

impl IsDynCompatibleForm for BeShared {
    type DynLeaf<'a, D: IsDynType> = QqqShared<D::DynContent>;

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
            .emplace_map(|inner, emplacer| inner.as_ref_value().into_shared(emplacer, span)))
    }
}

// impl MapIntoReturned for BeShared {
//     fn into_returned_value(
//         content: Content<'static, AnyType, Self>,
//     ) -> FunctionResult<ReturnedValue> {
//         todo!("Return shared")
//     }
// }
