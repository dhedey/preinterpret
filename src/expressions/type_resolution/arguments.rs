#![allow(unused)]
// TODO[unused-clearup]
use super::*;

pub(crate) struct ResolutionContext<'a> {
    span_range: &'a SpanRange,
    resolution_target: &'a str,
}

impl<'a> ResolutionContext<'a> {
    pub(crate) fn new(span_range: &'a SpanRange, resolution_target: &'a str) -> Self {
        Self {
            span_range,
            resolution_target,
        }
    }

    /// Create an error for the resolution context.
    pub(crate) fn err<T>(
        &self,
        expected_value_kind: &str,
        value: impl Borrow<ExpressionValue>,
    ) -> ExecutionResult<T> {
        self.span_range.type_err(format!(
            "{} is expected to be {}, but it is {}",
            self.resolution_target,
            expected_value_kind,
            value.borrow().articled_value_type()
        ))
    }
}

pub(crate) trait FromResolved: Sized {
    type ValueType: HierarchicalTypeData;
    const OWNERSHIP: ResolvedValueOwnership;
    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self>;
}

impl FromResolved for ResolvedValue {
    type ValueType = ValueTypeData;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::AsIs;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(value)
    }
}

impl FromResolved for AssigneeValue {
    type ValueType = ValueTypeData;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Assignee;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(AssigneeValue(value.expect_mutable()))
    }
}

impl<T: ResolvableArgumentShared + ResolvableArgumentTarget + ?Sized> FromResolved for Shared<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Shared;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        T::resolve_shared(value.expect_shared(), "This argument")
    }
}

impl<T: 'static + ?Sized> FromResolved for AnyRef<'static, T>
where
    Shared<T>: FromResolved,
{
    type ValueType = <Shared<T> as FromResolved>::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = <Shared<T> as FromResolved>::OWNERSHIP;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(Shared::<T>::from_resolved(value)?.into())
    }
}

impl<T: ResolvableArgumentMutable + ResolvableArgumentTarget + ?Sized> FromResolved for Mutable<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Mutable;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        T::resolve_mutable(value.expect_mutable(), "This argument")
    }
}

impl<T: 'static + ?Sized> FromResolved for AnyRefMut<'static, T>
where
    Mutable<T>: FromResolved,
{
    type ValueType = <Mutable<T> as FromResolved>::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = <Mutable<T> as FromResolved>::OWNERSHIP;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(Mutable::<T>::from_resolved(value)?.into())
    }
}

impl<T: ResolvableArgumentOwned + ResolvableArgumentTarget> FromResolved for Owned<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        T::resolve_owned(value.expect_owned(), "This argument")
    }
}

impl<T: ResolvableArgumentOwned + ResolvableArgumentTarget> FromResolved for T {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        T::resolve_value(value.expect_owned(), "This argument")
    }
}

impl<T: ResolvableArgumentShared + ResolvableArgumentTarget + ToOwned> FromResolved
    for CopyOnWrite<T>
where
    T::Owned: ResolvableArgumentOwned,
{
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::CopyOnWrite;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        value.expect_copy_on_write().map(
            |v| T::resolve_shared(v, "This argument"),
            |v| <T::Owned as ResolvableArgumentOwned>::resolve_owned(v, "This argument"),
        )
    }
}

impl<T: FromResolved> FromResolved for Spanned<T> {
    type ValueType = <T as FromResolved>::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = <T as FromResolved>::OWNERSHIP;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        let span_range = value.span_range();
        Ok(Spanned {
            value: T::from_resolved(value)?,
            span_range,
        })
    }
}

pub(crate) trait ResolveAs<T> {
    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<T>;
}

impl<T: ResolvableArgumentOwned> ResolveAs<T> for OwnedValue {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<T> {
        T::resolve_value(self, resolution_target)
    }
}

impl<T: ResolvableArgumentOwned> ResolveAs<Owned<T>> for OwnedValue {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Owned<T>> {
        T::resolve_owned(self, resolution_target)
    }
}

impl<'a, T: ResolvableArgumentShared + ?Sized> ResolveAs<&'a T> for Spanned<&'a ExpressionValue> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<&'a T> {
        T::resolve_ref(self, resolution_target)
    }
}

impl<'a, T: ResolvableArgumentShared + ?Sized> ResolveAs<Spanned<&'a T>>
    for Spanned<&'a ExpressionValue>
{
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Spanned<&'a T>> {
        T::resolve_spanned_ref(self, resolution_target)
    }
}

impl<'a, T: ResolvableArgumentMutable + ?Sized> ResolveAs<&'a mut T>
    for Spanned<&'a mut ExpressionValue>
{
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<&'a mut T> {
        T::resolve_ref_mut(self, resolution_target)
    }
}

impl<'a, T: ResolvableArgumentMutable + ?Sized> ResolveAs<Spanned<&'a mut T>>
    for Spanned<&'a mut ExpressionValue>
{
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Spanned<&'a mut T>> {
        T::resolve_spanned_ref_mut(self, resolution_target)
    }
}

pub(crate) trait ResolvableArgumentTarget {
    type ValueType: HierarchicalTypeData;
}

pub(crate) trait ResolvableArgumentOwned: Sized {
    fn resolve_from_value(
        value: ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self>;

    fn resolve_owned_from_value(
        value: ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Owned<Self>> {
        let span_range = *context.span_range;
        Self::resolve_from_value(value, context).map(|v| Owned::new(v, span_range))
    }

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_value(
        value: Owned<ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<Self> {
        let (value, span_range) = value.deconstruct();
        let context = ResolutionContext {
            span_range: &span_range,
            resolution_target,
        };
        Self::resolve_from_value(value, context)
    }

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_owned(
        value: Owned<ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<Owned<Self>> {
        let (value, span_range) = value.deconstruct();
        let context = ResolutionContext {
            span_range: &span_range,
            resolution_target,
        };
        Self::resolve_from_value(value, context).map(|v| Owned::new(v, span_range))
    }
}

pub(crate) trait ResolvableArgumentShared {
    fn resolve_from_ref<'a>(
        value: &'a ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a Self>;

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_shared(
        value: Shared<ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<Shared<Self>> {
        value.try_map(|v, span_range| {
            Self::resolve_from_ref(
                v,
                ResolutionContext {
                    span_range,
                    resolution_target,
                },
            )
        })
    }

    fn resolve_ref<'a>(
        value: Spanned<&'a ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<&'a Self> {
        Self::resolve_from_ref(
            value.value,
            ResolutionContext {
                span_range: &value.span_range,
                resolution_target,
            },
        )
    }

    fn resolve_spanned_ref<'a>(
        value: Spanned<&'a ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<Spanned<&'a Self>> {
        value.try_map(|v, span_range| {
            Self::resolve_from_ref(
                v,
                ResolutionContext {
                    span_range,
                    resolution_target,
                },
            )
        })
    }
}

pub(crate) trait ResolvableArgumentMutable {
    fn resolve_from_mut<'a>(
        value: &'a mut ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a mut Self>;

    fn resolve_mutable(
        value: Mutable<ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<Mutable<Self>> {
        value.try_map(|v, span_range| {
            Self::resolve_from_mut(
                v,
                ResolutionContext {
                    span_range,
                    resolution_target,
                },
            )
        })
    }

    fn resolve_ref_mut<'a>(
        value: Spanned<&'a mut ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<&'a mut Self> {
        Self::resolve_from_mut(
            value.value,
            ResolutionContext {
                span_range: &value.span_range,
                resolution_target,
            },
        )
    }

    fn resolve_spanned_ref_mut<'a>(
        value: Spanned<&'a mut ExpressionValue>,
        resolution_target: &str,
    ) -> ExecutionResult<Spanned<&'a mut Self>> {
        value.try_map(|value, span_range| {
            Self::resolve_from_mut(
                value,
                ResolutionContext {
                    span_range,
                    resolution_target,
                },
            )
        })
    }
}

impl ResolvableArgumentTarget for ExpressionValue {
    type ValueType = ValueTypeData;
}

impl ResolvableArgumentOwned for ExpressionValue {
    fn resolve_from_value(
        value: ExpressionValue,
        _context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        Ok(value)
    }
}
impl ResolvableArgumentShared for ExpressionValue {
    fn resolve_from_ref<'a>(
        value: &'a ExpressionValue,
        _context: ResolutionContext,
    ) -> ExecutionResult<&'a Self> {
        Ok(value)
    }
}
impl ResolvableArgumentMutable for ExpressionValue {
    fn resolve_from_mut<'a>(
        value: &'a mut ExpressionValue,
        _context: ResolutionContext,
    ) -> ExecutionResult<&'a mut Self> {
        Ok(value)
    }
}

macro_rules! impl_resolvable_argument_for {
    ($value_type:ty, ($value:ident, $context:ident) -> $type:ty $body:block) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_value(
                $value: ExpressionValue,
                $context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                $body
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref<'a>(
                $value: &'a ExpressionValue,
                $context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                $body
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut<'a>(
                $value: &'a mut ExpressionValue,
                $context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                $body
            }
        }
    };
}

pub(crate) use impl_resolvable_argument_for;

macro_rules! impl_delegated_resolvable_argument_for {
    ($value_type:ty, ($value:ident: $delegate:ty) -> $type:ty { $expr:expr }) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_value(
                input_value: ExpressionValue,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                let $value: $delegate =
                    ResolvableArgumentOwned::resolve_from_value(input_value, context)?;
                Ok($expr)
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref<'a>(
                input_value: &'a ExpressionValue,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                let $value: &$delegate =
                    ResolvableArgumentShared::resolve_from_ref(input_value, context)?;
                Ok(&$expr)
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut<'a>(
                input_value: &'a mut ExpressionValue,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                let $value: &mut $delegate =
                    ResolvableArgumentMutable::resolve_from_mut(input_value, context)?;
                Ok(&mut $expr)
            }
        }
    };
}

pub(crate) use impl_delegated_resolvable_argument_for;
