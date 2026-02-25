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
    type DynLeaf<'a, D: IsDynType> = QqqAssignee<D::DynContent>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: IsDynType>(
        Spanned(leaf, span): Spanned<Self::Leaf<'a, T>>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D::DynContent>,
    {
        leaf.0
            .emplace_map(|content, emplacer| match <T::Leaf>::map_mut(content) {
                Ok(mapped) => {
                    // SAFETY: PathExtension::Tightened correctly describes type narrowing
                    let mapped_mut = unsafe {
                        MappedMut::new(mapped, PathExtension::Tightened(D::type_kind()), span)
                    };
                    Ok(QqqAssignee(emplacer.emplace(mapped_mut)))
                }
                Err(_this) => Err(QqqAssignee(emplacer.revert())),
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
        Spanned(value, _span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        Ok(value.expect_assignee().into_content())
    }
}
