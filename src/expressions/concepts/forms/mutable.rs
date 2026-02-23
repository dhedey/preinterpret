use super::*;

pub(crate) type QqqMutable<T> = MutableReference<T>;

impl<L: IsValueLeaf> IsValueContent for QqqMutable<L> {
    type Type = L::Type;
    type Form = BeMutable;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqMutable<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqMutable<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeMutable;
impl IsForm for BeMutable {}

impl IsHierarchicalForm for BeMutable {
    type Leaf<'a, T: IsLeafType> = QqqMutable<T::Leaf>;
}

impl IsDynCompatibleForm for BeMutable {
    type DynLeaf<'a, D: 'static + ?Sized> = QqqMutable<D>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D>,
    {
        leaf.emplace_map(|content, emplacer| {
            let span = emplacer.current_span();
            match <T::Leaf>::map_mut(content) {
                Ok(mapped) => Ok(unsafe {
                    emplacer.emplace_unchecked(
                        mapped,
                        PathExtension::Tightened(T::type_kind()),
                        span,
                    )
                }),
                Err(this) => Err(unsafe {
                    emplacer.emplace_unchecked(this, PathExtension::Tightened(T::type_kind()), span)
                }),
            }
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
        value: ArgumentValue,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value
            .expect_mutable()
            .emplace_map(|inner, emplacer| inner.as_mut_value().into_mutable(emplacer)))
    }
}

// impl MapIntoReturned for BeMutable {
//     fn into_returned_value(
//         content: Content<'static, AnyType, Self>,
//     ) -> ExecutionResult<ReturnedValue> {
//         todo!("Return mutable")
//     }
// }
