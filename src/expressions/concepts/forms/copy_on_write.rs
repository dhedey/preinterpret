use super::*;

#[derive(Copy, Clone)]
pub(crate) struct BeCopyOnWrite;
impl IsForm for BeCopyOnWrite {}

impl IsHierarchicalForm for BeCopyOnWrite {
    type Leaf<'a, T: IsLeafType> = CopyOnWriteContent<T::Leaf, T::Leaf>;
}

impl IsDynCompatibleForm for BeCopyOnWrite {
    type DynLeaf<'a, D: 'static + ?Sized> = CopyOnWriteContent<D, Box<D>>;
}

impl IsDynMappableForm for BeCopyOnWrite {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>,
    {
        // TODO: Add back once we add a map to CopyOnWriteContent
        todo!()
    }
}

impl MapFromArgument for BeCopyOnWrite {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::CopyOnWrite;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        todo!()
        // value.expect_copy_on_write()
    }
}

impl LeafAsRefForm for BeCopyOnWrite {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        match leaf {
            CopyOnWriteContent::Owned(owned) => owned,
            CopyOnWriteContent::SharedWithInfallibleCloning(shared) => shared,
            CopyOnWriteContent::SharedWithTransparentCloning(shared) => shared,
        }
    }

    fn leaf_clone_to_owned_infallible<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
    ) -> T::Leaf {
        todo!("Need to copy the custom implementation")
    }

    fn leaf_clone_to_owned_transparently<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
        error_span: SpanRange,
    ) -> ExecutionResult<T::Leaf> {
        todo!("Need to copy the custom implementation")
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

pub(crate) trait IsSelfCopyOnWriteContent<'a>: IsSelfValueContent<'a>
where
    Self: IsValueContent<'a, Form = BeCopyOnWrite>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
    fn acts_as_shared_reference(&self) -> bool {
        map_via_leaf! {
            input: &'r (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F = BeCopyOnWrite, T>(leaf) -> (bool) {
                match leaf {
                    CopyOnWriteContent::Owned(_) => false,
                    CopyOnWriteContent::SharedWithInfallibleCloning(_) => false,
                    CopyOnWriteContent::SharedWithTransparentCloning(_) => true,
                }
            }
        }
    }
}

impl<'a, C: IsSelfValueContent<'a>> IsSelfCopyOnWriteContent<'a> for C
where
    Self: IsValueContent<'a, Form = BeCopyOnWrite>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
}
