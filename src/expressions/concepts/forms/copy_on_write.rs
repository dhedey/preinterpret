use super::*;

pub(crate) type QqqCopyOnWrite<T> = Actual<'static, T, BeCopyOnWrite>;

#[derive(Copy, Clone)]
pub(crate) struct BeCopyOnWrite;
impl IsForm for BeCopyOnWrite {}

impl IsHierarchicalForm for BeCopyOnWrite {
    type Leaf<'a, T: IsValueLeaf> = CopyOnWriteContent<T, T>;
}

impl IsDynCompatibleForm for BeCopyOnWrite {
    type DynLeaf<'a, T: 'static + ?Sized> = CopyOnWriteContent<T, Box<T>>;
}

impl IsDynMappableForm for BeCopyOnWrite {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        _leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        // TODO: Add back once we add a map to CopyOnWriteContent
        todo!()
    }
}

impl MapFromArgument for BeCopyOnWrite {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::CopyOnWrite;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, AnyType, Self>> {
        todo!()
        // value.expect_copy_on_write()
    }
}

/// Typically O == T or more specifically, <T as ToOwned>::Owned
pub(crate) enum CopyOnWriteContent<T: 'static + ?Sized, O: 'static> {
    /// An owned value that can be used directly
    Owned(Owned<O>),
    /// For use when the CopyOnWrite value effectively represents the owned value (post-clone).
    /// In this case, returning a Cow is just an optimization and we can always clone infallibly.
    SharedWithInfallibleCloning(Shared<T>),
    /// For use when the CopyOnWrite value represents a pre-cloned read-only value.
    /// A transparent clone may fail in this case at use time.
    SharedWithTransparentCloning(Shared<T>),
}

// pub(crate) trait IsSelfCopyOnWriteContent<'a>: IsSelfValueContent<'a>
// where
//     Self: IsValueContent<'a, Form = BeCopyOnWrite>,
//     BeCopyOnWrite: IsFormOf<<Self as IsValueContent<'a>>::Type, Content<'a> = Self>,
// {
//     fn acts_as_shared_reference(&self) -> bool {
//         struct ThisMapper;
//         impl
//         self.map_ref_with(mapper)
//     }
// }

// impl<'a, C: IsSelfValueContent<'a>> IsSelfCopyOnWriteContent<'a> for A
// where
//     Self: IsValueContent<'a, Form = BeCopyOnWrite>,
// {}
