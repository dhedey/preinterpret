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

// Clashes with other blanket impl it will replace!
//
// impl<
//     X: FromValueContent<'static, Type = T, Form = F>,
//     F: IsForm + MapFromArgument,
//     T: TypeData + DowncastFrom<ValueType, F>,
// > IsArgument for X {
//     type ValueType = T;
//     const OWNERSHIP: ArgumentOwnership = F::ARGUMENT_OWNERSHIP;
//     fn from_argument(Spanned(value, span_range): Spanned<ArgumentValue>) -> ExecutionResult<Self> {
//         let ownership_mapped = F::from_argument_value(value)?;
//         let type_mapped = T::resolve(ownership_mapped, span_range, "This argument")?;
//         Ok(X::from_actual(type_mapped))
//     }
// }

// TODO[concepts]: Replace with the above impl when ready
impl<F: IsFormOf<T> + MapFromArgument, T: TypeData + DowncastFrom<ValueType, F>> IsArgument
    for Actual<'static, T, F>
where
    for<'l> ValueType:
        IsHierarchicalType<Content<'l, F> = <F as form::IsFormOf<ValueType>>::Content<'l>>,
    F: IsHierarchicalForm,
{
    type ValueType = T;
    const OWNERSHIP: ArgumentOwnership = F::ARGUMENT_OWNERSHIP;
    fn from_argument(Spanned(value, span_range): Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        let ownership_mapped = F::from_argument_value(value)?;
        let type_mapped = T::resolve(ownership_mapped.0, span_range, "This argument")?;
        Ok(Actual(type_mapped))
    }
}

// Clashes with other blanket impl it will replace!
//
// impl<
//     X: IntoValueContent<'static, Type = T, Form = F>,
//     F: IsForm + MapIntoReturned,
//     T: UpcastTo<ValueType, F>,
// > IsReturnable for X {
//     fn to_returned_value(self) -> ExecutionResult<ReturnedValue> {
//         let type_mapped = self.into_actual()
//             .upcast::<ValueType>();
//         F::into_returned_value(type_mapped)
//     }
// }

// TODO[concepts]: Replace with the above impl when ready
impl<F: IsFormOf<T> + MapIntoReturned, T: UpcastTo<ValueType, F>> IsReturnable
    for Actual<'static, T, F>
{
    fn to_returned_value(self) -> ExecutionResult<ReturnedValue> {
        let type_mapped = self.into_actual().upcast::<ValueType>();
        F::into_returned_value(type_mapped)
    }
}
