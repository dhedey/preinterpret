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

pub(crate) trait IsFormOf<T: IsType>: IsForm {
    type Content<'a>;
}

pub(crate) trait IsFormOfForKind<T: IsType, X: TypeVariant>: IsForm {
    type KindedContent<'a>;
}

impl<T: IsType, F: IsFormOfForKind<T, T::Variant>> IsFormOf<T> for F {
    type Content<'a> = F::KindedContent<'a>;
}

pub(crate) trait IsHierarchicalForm: IsForm {
    /// The standard leaf for a hierachical type
    type Leaf<'a, T: IsValueLeaf>;
}

impl<T: IsHierarchicalType, F: IsHierarchicalForm> IsFormOfForKind<T, HierarchicalTypeVariant>
    for F
{
    type KindedContent<'a> = T::Content<'a, F>;
}

pub(crate) trait LeafAsRefForm: IsHierarchicalForm {
    fn leaf_as_ref<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r Self::Leaf<'a, T>) -> &'r T;
}

pub(crate) trait LeafAsMutForm: IsHierarchicalForm {
    fn leaf_as_mut<'r, 'a: 'r, T: IsValueLeaf>(leaf: &'r mut Self::Leaf<'a, T>) -> &'r mut T;
}

pub(crate) trait IsDynCompatibleForm: IsForm {
    /// The container for a dyn Trait based type.
    /// The DynLeaf can be similar to the standard leaf, but must be
    /// able to support an unsized D.
    type DynLeaf<'a, D: 'static + ?Sized>;
}

impl<T: IsDynType, F: IsDynCompatibleForm> IsFormOfForKind<T, DynTypeVariant> for F {
    type KindedContent<'a> = F::DynLeaf<'a, T::DynContent>;
}

pub(crate) trait IsDynMappableForm: IsHierarchicalForm + IsDynCompatibleForm {
    fn leaf_to_dyn<'a, T: IsValueLeaf + CastDyn<D>, D: ?Sized + 'static>(
        leaf: Self::Leaf<'a, T>,
    ) -> Option<Self::DynLeaf<'a, D>>;
}

pub(crate) trait MapFromArgument: IsFormOf<ValueType> {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>>;
}

pub(crate) trait MapIntoReturned: IsFormOf<ValueType> {
    fn into_returned_value(
        value: Actual<'static, ValueType, Self>,
    ) -> ExecutionResult<ReturnedValue>;
}
