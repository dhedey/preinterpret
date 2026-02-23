use super::*;

impl<L: IsValueLeaf> IsValueContent for &L {
    type Type = L::Type;
    type Form = BeRef;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for &'a L {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for &'a L {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

/// It can't be an argument because arguments must be owned in some way;
/// so that the drop glue can work properly (because they may come from
/// e.g. a reference counted Shared handle)
#[derive(Copy, Clone)]
pub(crate) struct BeRef;
impl IsForm for BeRef {}

impl IsHierarchicalForm for BeRef {
    type Leaf<'a, T: IsLeafType> = &'a T::Leaf;
}

impl IsDynCompatibleForm for BeRef {
    type DynLeaf<'a, D: 'static + ?Sized> = &'a D;

    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Result<Self::DynLeaf<'a, D>, Content<'a, T, Self>>
    where
        T::Leaf: CastDyn<D>,
    {
        <T::Leaf>::map_ref(leaf)
    }
}

impl LeafAsRefForm for BeRef {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        leaf
    }
}

pub(crate) trait IsSelfRefContent<'a>: IsSelfValueContent<'a>
where
    Self: IsValueContent<Form = BeRef>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
    fn into_shared<'b, T: 'static>(
        self,
        emplacer: &'b mut SharedEmplacer<'a, T>,
    ) -> Content<'static, Self::Type, BeShared>
    where
        Self: Sized,
    {
        struct __InlineMapper<'b, 'e2, X: 'static> {
            emplacer: &'b mut SharedEmplacer<'e2, X>,
        }
        impl<'b, 'e2, X> LeafMapper<BeRef> for __InlineMapper<'b, 'e2, X> {
            type Output<'a, T: IsHierarchicalType> = Content<'static, T, BeShared>;

            fn to_parent_output<'a, T: IsChildType>(
                output: Self::Output<'a, T>,
            ) -> Self::Output<'a, T::ParentType> {
                T::into_parent(output)
            }

            fn map_leaf<'l, T: IsLeafType>(
                self,
                leaf: <BeRef as IsHierarchicalForm>::Leaf<'l, T>,
            ) -> Self::Output<'l, T> {
                // SAFETY: 'l = 'a = 'e so this is valid
                let span = self.emplacer.current_span();
                unsafe {
                    self.emplacer.emplace_unchecked(
                        leaf,
                        PathExtension::Tightened(T::type_kind()),
                        span,
                    )
                }
            }
        };
        let __mapper = __InlineMapper { emplacer };
        <Self::Type>::map_with::<BeRef, _>(__mapper, self)
    }

    fn into_shared_any_ref<'b, T: 'static>(
        self,
        emplacer: &'b mut SharedEmplacer<'a, T>,
    ) -> Content<'static, Self::Type, BeAnyRef>
    where
        Self: Sized,
    {
        struct __InlineMapper<'b, 'e2, X: 'static> {
            emplacer: &'b mut SharedEmplacer<'e2, X>,
        }
        impl<'b, 'e2, X> LeafMapper<BeRef> for __InlineMapper<'b, 'e2, X> {
            type Output<'a, T: IsHierarchicalType> = Content<'static, T, BeAnyRef>;

            fn to_parent_output<'a, T: IsChildType>(
                output: Self::Output<'a, T>,
            ) -> Self::Output<'a, T::ParentType> {
                T::into_parent(output)
            }

            fn map_leaf<'l, T: IsLeafType>(
                self,
                leaf: <BeRef as IsHierarchicalForm>::Leaf<'l, T>,
            ) -> Self::Output<'l, T> {
                // SAFETY: 'l = 'a = 'e so this is valid
                let span = self.emplacer.current_span();
                let shared_ref: SharedReference<_> = unsafe {
                    self.emplacer.emplace_unchecked(
                        leaf,
                        PathExtension::Tightened(T::type_kind()),
                        span,
                    )
                };
                shared_ref.into()
            }
        };
        let __mapper = __InlineMapper { emplacer };
        <Self::Type>::map_with::<BeRef, _>(__mapper, self)
    }
}

impl<'a, C: IsSelfValueContent<'a>> IsSelfRefContent<'a> for C
where
    Self: IsValueContent<Form = BeRef>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
}
