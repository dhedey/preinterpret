use super::*;

/// A type representing a particular form of a value's ownership.
///
/// Examples include:
/// - [BeOwned] representing [Owned] (floating) value
/// - [BeShared] representing [Shared] references
/// - [BeMutable] representing [Mutable] references
/// - [BeCopyOnWrite] representing [CopyOnWrite] values
///
/// NB: The Sized + Clone bounds are just to make certain derive impls easier,
/// due to e.g. the poor auto-derive of Clone which requires Clone bounds on all
/// generics.
pub(crate) trait IsForm: Sized + Clone {}

pub(crate) trait IsHierarchicalForm: IsForm {
    /// The standard leaf for a hierachical type
    type Leaf<'a, T: IsLeafType>: IsValueContent<Type = T, Form = Self>
        + IntoValueContent<'a>
        + FromValueContent<'a>;
}

pub(crate) trait LeafAsRefForm: IsHierarchicalForm {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf;

    fn leaf_clone_to_owned_infallible<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
    ) -> T::Leaf {
        Self::leaf_as_ref(leaf).clone()
    }

    fn leaf_clone_to_owned_transparently<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
        error_span: SpanRange,
    ) -> ExecutionResult<T::Leaf> {
        let type_kind = T::type_kind();
        if type_kind.supports_transparent_cloning() {
            Ok(Self::leaf_clone_to_owned_infallible(leaf))
        } else {
            error_span.ownership_err(format!(
                "An owned value is required, but a reference was received, and {} does not support transparent cloning. You may wish to use .clone() explicitly.",
                type_kind.articled_display_name()
            ))
        }
    }
}

pub(crate) trait LeafAsMutForm: IsHierarchicalForm {
    fn leaf_as_mut<'r, 'a: 'r, T: IsLeafType>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T::Leaf;
}

pub(crate) trait IsDynCompatibleForm: IsForm {
    /// The container for a dyn Trait based type.
    /// The DynLeaf can be similar to the standard leaf, but must be
    /// able to support an unsized D.
    type DynLeaf<'a, D: 'static + ?Sized>;
}

pub(crate) trait IsDynMappableForm: IsHierarchicalForm + IsDynCompatibleForm {
    fn leaf_to_dyn<'a, T: IsLeafType, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>
    where
        T::Leaf: CastDyn<D>;
}

pub(crate) trait MapFromArgument: IsHierarchicalForm {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>>;
}

pub(crate) trait MapIntoReturned: IsHierarchicalForm {
    fn into_returned_value(
        value: Content<'static, AnyType, Self>,
    ) -> ExecutionResult<ReturnedValue>;
}
