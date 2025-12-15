use super::*;

type CopyOnWrite<T> = Actual<'static, T, BeCopyOnWrite>;

pub(crate) struct BeCopyOnWrite;
impl IsForm for BeCopyOnWrite {
    type Leaf<'a, T: IsValueLeaf> = CopyOnWriteContent<T, T>;
    type DynLeaf<'a, T: 'static + ?Sized> = CopyOnWriteContent<T, Box<T>>;
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::CopyOnWrite;

    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        _leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>> {
        // TODO: Add back once we add a map to LateBoundContent
        todo!()
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
