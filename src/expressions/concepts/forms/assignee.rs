use super::*;

pub(crate) struct QqqAssignee<T: 'static + ?Sized>(pub(crate) MutableReference<T>);

impl<L: IsValueLeaf> IsValueContent for QqqAssignee<L> {
    type Type = L::Type;
    type Form = BeAssignee;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqAssignee<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqAssignee<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeAssignee;
impl IsForm for BeAssignee {}

impl IsHierarchicalForm for BeAssignee {
    type Leaf<'a, T: IsLeafType> = QqqAssignee<T::Leaf>;
}

impl IsDynCompatibleForm for BeAssignee {
    type DynLeaf<'a, D: 'static + ?Sized> = QqqAssignee<D>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D>,
    {
        leaf.0
            .replace_legacy(|content, emplacer| match <T::Leaf>::map_mut(content) {
                Ok(mapped) => Ok(QqqAssignee(unsafe {
                    emplacer.emplace_unchecked_legacy(mapped)
                })),
                Err(this) => Err(QqqAssignee(unsafe {
                    emplacer.emplace_unchecked_legacy(this)
                })),
            })
    }
}

impl LeafAsRefForm for BeAssignee {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        &leaf.0
    }
}

impl LeafAsMutForm for BeAssignee {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf {
        &mut leaf.0
    }
}

impl MapFromArgument for BeAssignee {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership =
        ArgumentOwnership::Assignee { auto_create: false };

    fn from_argument_value(
        value: ArgumentValue,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value.expect_assignee().into_content())
    }
}
