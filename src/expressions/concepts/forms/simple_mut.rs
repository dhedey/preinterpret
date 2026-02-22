use super::*;

impl<L: IsValueLeaf> IsValueContent for &mut L {
    type Type = L::Type;
    type Form = BeMut;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for &'a mut L {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for &'a mut L {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

/// It can't be an argument because arguments must be owned in some way;
/// so that the drop glue can work properly (because they may come from
/// e.g. a reference counted Shared handle)
#[derive(Copy, Clone)]
pub(crate) struct BeMut;
impl IsForm for BeMut {}

impl IsHierarchicalForm for BeMut {
    type Leaf<'a, T: IsLeafType> = &'a mut T::Leaf;
}

impl IsDynCompatibleForm for BeMut {
    type DynLeaf<'a, D: 'static + ?Sized> = &'a mut D;

    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D>,
    {
        <T::Leaf>::map_mut(leaf)
    }
}

impl LeafAsRefForm for BeMut {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

pub(crate) trait IsSelfMutContent<'a>: IsSelfValueContent<'a>
where
    Self: IsValueContent<Form = BeMut>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
    fn into_mutable<'b, T: 'static>(
        self,
        emplacer: &'b mut MutableEmplacer<'a, T>,
    ) -> Content<'static, Self::Type, BeMutable>
    where
        Self: Sized,
    {
        struct __InlineMapper<'b, 'e2, X: 'static> {
            emplacer: &'b mut MutableEmplacer<'e2, X>,
        }
        impl<'b, 'e2, X> LeafMapper<BeMut> for __InlineMapper<'b, 'e2, X> {
            type Output<'a, T: IsHierarchicalType> = Content<'static, T, BeMutable>;

            fn to_parent_output<'a, T: IsChildType>(
                output: Self::Output<'a, T>,
            ) -> Self::Output<'a, T::ParentType> {
                T::into_parent(output)
            }

            fn map_leaf<'l, T: IsLeafType>(
                self,
                leaf: <BeMut as IsHierarchicalForm>::Leaf<'l, T>,
            ) -> Self::Output<'l, T> {
                // SAFETY: 'l = 'a = 'e so this is valid
                unsafe { self.emplacer.emplace_unchecked_legacy(leaf) }
            }
        };
        let __mapper = __InlineMapper { emplacer };
        <Self::Type>::map_with::<BeMut, _>(__mapper, self)
    }

    fn into_assignee<'b, T: 'static>(
        self,
        emplacer: &'b mut MutableEmplacer<'a, T>,
    ) -> Content<'static, Self::Type, BeAssignee>
    where
        Self: Sized,
    {
        struct __InlineMapper<'b, 'e2, X: 'static> {
            emplacer: &'b mut MutableEmplacer<'e2, X>,
        }
        impl<'b, 'e2, X> LeafMapper<BeMut> for __InlineMapper<'b, 'e2, X> {
            type Output<'a, T: IsHierarchicalType> = Content<'static, T, BeAssignee>;

            fn to_parent_output<'a, T: IsChildType>(
                output: Self::Output<'a, T>,
            ) -> Self::Output<'a, T::ParentType> {
                T::into_parent(output)
            }

            fn map_leaf<'l, T: IsLeafType>(
                self,
                leaf: <BeMut as IsHierarchicalForm>::Leaf<'l, T>,
            ) -> Self::Output<'l, T> {
                // SAFETY: 'l = 'a = 'e so this is valid
                unsafe { QqqAssignee(self.emplacer.emplace_unchecked_legacy(leaf)) }
            }
        };
        let __mapper = __InlineMapper { emplacer };
        <Self::Type>::map_with::<BeMut, _>(__mapper, self)
    }

    fn into_mutable_any_mut<'b, T: 'static>(
        self,
        emplacer: &'b mut MutableEmplacer<'a, T>,
    ) -> Content<'static, Self::Type, BeAnyMut>
    where
        Self: Sized,
    {
        struct __InlineMapper<'b, 'e2, X: 'static> {
            emplacer: &'b mut MutableEmplacer<'e2, X>,
        }
        impl<'b, 'e2, X> LeafMapper<BeMut> for __InlineMapper<'b, 'e2, X> {
            type Output<'a, T: IsHierarchicalType> = Content<'static, T, BeAnyMut>;

            fn to_parent_output<'a, T: IsChildType>(
                output: Self::Output<'a, T>,
            ) -> Self::Output<'a, T::ParentType> {
                T::into_parent(output)
            }

            fn map_leaf<'l, T: IsLeafType>(
                self,
                leaf: <BeMut as IsHierarchicalForm>::Leaf<'l, T>,
            ) -> Self::Output<'l, T> {
                // SAFETY: 'l = 'a = 'e so this is valid
                let mutable_ref: MutableReference<_> =
                    unsafe { self.emplacer.emplace_unchecked_legacy(leaf) };
                mutable_ref.into()
            }
        };
        let __mapper = __InlineMapper { emplacer };
        <Self::Type>::map_with::<BeMut, _>(__mapper, self)
    }
}

impl<'a, C: IsSelfValueContent<'a>> IsSelfMutContent<'a> for C
where
    Self: IsValueContent<Form = BeMut>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
}
