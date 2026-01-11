use super::*;

pub(crate) type QqqMut<'a, T> = Actual<'a, T, BeMut>;

/// It can't be an argument because arguments must be owned in some way;
/// so that the drop glue can work properly (because they may come from
/// e.g. a reference counted Shared handle)
#[derive(Copy, Clone)]
pub(crate) struct BeMut;
impl IsForm for BeMut {}

impl IsHierarchicalForm for BeMut {
    type Leaf<'a, T: IsValueLeaf> = &'a mut T;
}

impl IsDynCompatibleForm for BeMut {
    type DynLeaf<'a, T: 'static + ?Sized> = &'a mut T;
}

impl IsDynMappableForm for BeMut {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        T::map_mut(leaf)
    }
}

impl LeafAsRefForm for BeMut {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T {
        leaf
    }
}

pub(crate) struct ToMutMapper;

impl<F: LeafAsMutForm> MutLeafMapper<F> for ToMutMapper {
    type OutputForm = BeMut;
    type ShortCircuit<'a> = Infallible;

    fn map_leaf<'r, 'a: 'r, L: IsValueLeaf>(
        self,
        leaf: &'r mut F::Leaf<'a, L>,
    ) -> Result<&'r mut L, Infallible> {
        Ok(F::leaf_as_mut(leaf))
    }
}

// impl<F: LeafAsMutForm + IsFormOf<AnyType>> MutLeafMapper<F> for ToMutMapper {
//     type Output<'r, 'a: 'r> = AnyValueContent<'r, BeMut>;

//     fn map_leaf<'r, 'a: 'r, L: IsValueLeaf>(
//         self,
//         leaf: &'r mut F::Leaf<'a, L>,
//     ) -> Self::Output<'r, 'a>
//     where
//         for<'x> &'x mut L: IsValueContent<'x, Form = BeMut>,
//         for<'x> <&'x mut L as IsValueContent<'x>>::Type: UpcastTo<AnyType, BeMut>,
//         BeMut: for<'x> IsFormOf<<&'x mut L as IsValueContent<'x>>::Type, Content<'r> = &'r mut L>,
//     {
//         let my_mut = F::leaf_as_mut(leaf);
//         <<&'r mut L as IsValueContent<'r>>::Type as UpcastTo<AnyType, BeMut>>::upcast_to(my_mut)
//     }
// }

// impl<F: LeafAsMutForm + IsFormOf<AnyType>> MutLeafMapper<F> for ToMutMapper {
//     type Output<'r, 'a: 'r> = AnyValueContent<'r, BeMut>;

//     fn map_leaf<'r, 'a: 'r, T: IsType, L: IsValueLeaf>(
//         self,
//         leaf: &'r mut F::Leaf<'a, L>,
//     ) -> Self::Output<'r, 'a>
//     where
//         for<'x> &'x mut L: IsValueContent<'x, Form = BeMut, Type = T>,
//         T: UpcastTo<AnyType, BeMut>,
//         BeMut: IsFormOf<T, Content<'r> = &'r mut L>,
//     {
//         let my_mut = F::leaf_as_mut(leaf);
//         <<&'r mut L as IsValueContent<'r>>::Type as UpcastTo<AnyType, BeMut>>::upcast_to(my_mut)
//     }
// }
