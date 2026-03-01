use super::*;

/// A binding of a unique (mutable) reference to a value.
/// See [`ArgumentOwnership::Assignee`] for more details.
///
/// If you need span information, wrap with `Spanned<Assignee<T>>`.
pub(crate) struct Assignee<T: 'static + ?Sized>(pub(crate) Mutable<T>);

impl AnyValueAssignee {
    pub(crate) fn set(&mut self, content: impl IntoAnyValue) {
        *self.0 = content.into_any_value();
    }
}

impl<X: IsValueContent> IsValueContent for Assignee<X> {
    type Type = X::Type;
    type Form = BeAssignee;
}

impl<'a, X: IsValueContent> IntoValueContent<'a> for Assignee<X>
where
    X: 'static,
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
    X::Form: IsHierarchicalForm,
    X::Form: LeafAsMutForm,
{
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self.0
            .emplace_map(|inner, emplacer| inner.as_mut_value().into_assignee(emplacer, None))
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for Assignee<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

impl<T: 'static + ?Sized> Deref for Assignee<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static + ?Sized> DerefMut for Assignee<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeAssignee;
impl IsForm for BeAssignee {}

impl IsHierarchicalForm for BeAssignee {
    type Leaf<'a, T: IsLeafType> = Assignee<T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

impl IsDynCompatibleForm for BeAssignee {
    type DynLeaf<'a, D: IsDynType> = Assignee<D::DynContent>;

    fn leaf_to_dyn<'a, T: IsLeafType, D: IsDynType>(
        Spanned(leaf, span): Spanned<Self::Leaf<'a, T>>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D::DynContent>,
    {
        leaf.0
            .emplace_map(|content, emplacer| match <T::Leaf>::map_mut(content) {
                Ok(mapped) => {
                    // SAFETY: PathExtension is correct for mapping to a dyn type
                    let mapped_mut = unsafe {
                        MappedMut::new(mapped, PathExtension::TypeNarrowing(D::type_kind()), span)
                    };
                    Ok(Assignee(emplacer.emplace(mapped_mut)))
                }
                Err(_this) => Err(Assignee(emplacer.revert())),
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
