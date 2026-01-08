#![allow(unused)]
// TODO[unused-clearup]
use super::*;

/// Used for resolution of integers and floats from either typed or untyped literals.
pub(crate) struct OptionalSuffix<T>(pub(crate) T);

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

    #[inline]
    pub(crate) fn value_span_range(&self) -> SpanRange {
        *self.span_range
    }

    #[inline]
    pub(crate) fn error_span_range(&self) -> SpanRange {
        *self.span_range
    }

    /// Create an error for the resolution context.
    pub(crate) fn err<T, V: HasLeafKind>(
        &self,
        articled_expected_value_kind: &str,
        value: V,
    ) -> ExecutionResult<T> {
        self.span_range.type_err(format!(
            "{} is expected to be {}, but it is {}",
            self.resolution_target,
            articled_expected_value_kind,
            value.articled_kind()
        ))
    }
}

pub(crate) trait IsArgument: Sized {
    type ValueType: TypeData;
    const OWNERSHIP: ArgumentOwnership;
    fn from_argument(value: Spanned<ArgumentValue>) -> ExecutionResult<Self>;
}

impl IsArgument for ArgumentValue {
    type ValueType = AnyType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::AsIs;

    fn from_argument(Spanned(value, _): Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        Ok(value)
    }
}

impl<T: ResolvableShared<AnyValue> + ResolvableArgumentTarget + ?Sized> IsArgument for Shared<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        T::resolve_shared(argument.expect_shared(), "This argument")
    }
}

impl<T: 'static + ?Sized> IsArgument for AnyRef<'static, T>
where
    Shared<T>: IsArgument,
{
    type ValueType = <Shared<T> as IsArgument>::ValueType;
    const OWNERSHIP: ArgumentOwnership = <Shared<T> as IsArgument>::OWNERSHIP;

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        Ok(Shared::<T>::from_argument(argument)?.into())
    }
}

impl<T: ResolvableMutable<AnyValue> + ResolvableArgumentTarget + ?Sized> IsArgument
    for Assignee<T>
{
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Assignee { auto_create: false };

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        T::resolve_assignee(argument.expect_assignee(), "This argument")
    }
}

impl<T: ResolvableMutable<AnyValue> + ResolvableArgumentTarget + ?Sized> IsArgument for Mutable<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        T::resolve_mutable(argument.expect_mutable(), "This argument")
    }
}

impl<T: 'static + ?Sized> IsArgument for AnyMut<'static, T>
where
    Mutable<T>: IsArgument,
{
    type ValueType = <Mutable<T> as IsArgument>::ValueType;
    const OWNERSHIP: ArgumentOwnership = <Mutable<T> as IsArgument>::OWNERSHIP;

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        Ok(Mutable::<T>::from_argument(argument)?.into())
    }
}

impl<T: ResolvableOwned<AnyValue> + ResolvableArgumentTarget> IsArgument for T {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        T::resolve_value(argument.expect_owned(), "This argument")
    }
}

impl<T: ResolvableShared<AnyValue> + ResolvableArgumentTarget + ToOwned> IsArgument
    for CopyOnWrite<T>
where
    T::Owned: ResolvableOwned<AnyValue>,
{
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::CopyOnWrite;

    fn from_argument(Spanned(value, span): Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        value.expect_copy_on_write().map(
            |v| T::resolve_shared(v.spanned(span), "This argument"),
            |v| {
                <T::Owned as ResolvableOwned<AnyValue>>::resolve_value(
                    v.spanned(span),
                    "This argument",
                )
            },
        )
    }
}

impl<T: IsArgument> IsArgument for Spanned<T> {
    type ValueType = <T as IsArgument>::ValueType;
    const OWNERSHIP: ArgumentOwnership = <T as IsArgument>::OWNERSHIP;

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        let span = argument.1;
        Ok(Spanned(T::from_argument(argument)?, span))
    }
}

pub(crate) trait ResolveAs<T> {
    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<T>;
}

// Sadly this can't be changed Value => V because of spurious issues with
// https://github.com/rust-lang/rust/issues/48869
// Instead, we could introduce a different trait ResolveAs2 if needed.
impl<T: ResolvableOwned<AnyValue>> ResolveAs<T> for Spanned<AnyValue> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<T> {
        T::resolve_value(self, resolution_target)
    }
}

impl<T: ResolvableShared<AnyValue> + ?Sized> ResolveAs<Shared<T>> for Spanned<AnyValueShared> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Shared<T>> {
        T::resolve_shared(self, resolution_target)
    }
}

impl<'a, T: ResolvableShared<AnyValue> + ?Sized> ResolveAs<&'a T> for Spanned<&'a AnyValue> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<&'a T> {
        T::resolve_ref(self, resolution_target)
    }
}

impl<'a, T: ResolvableShared<AnyValue> + ?Sized> ResolveAs<Spanned<&'a T>>
    for Spanned<&'a AnyValue>
{
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Spanned<&'a T>> {
        T::resolve_spanned_ref(self, resolution_target)
    }
}

impl<T: ResolvableMutable<AnyValue> + ?Sized> ResolveAs<Mutable<T>> for Spanned<AnyValueMutable> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Mutable<T>> {
        T::resolve_mutable(self, resolution_target)
    }
}

impl<'a, T: ResolvableMutable<AnyValue> + ?Sized> ResolveAs<&'a mut T>
    for Spanned<&'a mut AnyValue>
{
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<&'a mut T> {
        T::resolve_ref_mut(self, resolution_target)
    }
}

impl<'a, T: ResolvableMutable<AnyValue> + ?Sized> ResolveAs<Spanned<&'a mut T>>
    for Spanned<&'a mut AnyValue>
{
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Spanned<&'a mut T>> {
        T::resolve_spanned_ref_mut(self, resolution_target)
    }
}

pub(crate) trait ResolvableArgumentTarget {
    type ValueType: TypeData;
}

pub(crate) trait ResolvableOwned<T>: Sized {
    fn resolve_from_value(value: T, context: ResolutionContext) -> ExecutionResult<Self>;

    fn resolve_spanned_from_value(
        value: T,
        context: ResolutionContext,
    ) -> ExecutionResult<Spanned<Owned<Self>>> {
        let span_range = context.span_range;
        Self::resolve_from_value(value, context).map(|x| x.spanned(*span_range))
    }

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_value(
        Spanned(value, span): Spanned<T>,
        resolution_target: &str,
    ) -> ExecutionResult<Self> {
        let context = ResolutionContext {
            span_range: &span,
            resolution_target,
        };
        Self::resolve_from_value(value, context)
    }
}

pub(crate) trait ResolvableShared<T> {
    fn resolve_from_ref<'a>(value: &'a T, context: ResolutionContext) -> ExecutionResult<&'a Self>;

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_shared(
        Spanned(value, span): Spanned<Shared<T>>,
        resolution_target: &str,
    ) -> ExecutionResult<Shared<Self>> {
        value.try_map(|v| {
            Self::resolve_from_ref(
                v,
                ResolutionContext {
                    span_range: &span,
                    resolution_target,
                },
            )
        })
    }

    fn resolve_ref<'a>(
        Spanned(value, span): Spanned<&'a T>,
        resolution_target: &str,
    ) -> ExecutionResult<&'a Self> {
        Self::resolve_from_ref(
            value,
            ResolutionContext {
                span_range: &span,
                resolution_target,
            },
        )
    }

    fn resolve_spanned_ref<'a>(
        Spanned(value, span): Spanned<&'a T>,
        resolution_target: &str,
    ) -> ExecutionResult<Spanned<&'a Self>> {
        Spanned(value, span).try_map(|v| {
            Self::resolve_from_ref(
                v,
                ResolutionContext {
                    span_range: &span,
                    resolution_target,
                },
            )
        })
    }
}

pub(crate) trait ResolvableMutable<T> {
    fn resolve_from_mut<'a>(
        value: &'a mut T,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a mut Self>;

    fn resolve_assignee(
        Spanned(value, span): Spanned<Assignee<T>>,
        resolution_target: &str,
    ) -> ExecutionResult<Assignee<Self>> {
        Ok(Assignee(Self::resolve_mutable(
            Spanned(value.0, span),
            resolution_target,
        )?))
    }

    fn resolve_mutable(
        Spanned(value, span): Spanned<Mutable<T>>,
        resolution_target: &str,
    ) -> ExecutionResult<Mutable<Self>> {
        value.try_map(|v| {
            Self::resolve_from_mut(
                v,
                ResolutionContext {
                    span_range: &span,
                    resolution_target,
                },
            )
        })
    }

    fn resolve_ref_mut<'a>(
        Spanned(value, span): Spanned<&'a mut T>,
        resolution_target: &str,
    ) -> ExecutionResult<&'a mut Self> {
        Self::resolve_from_mut(
            value,
            ResolutionContext {
                span_range: &span,
                resolution_target,
            },
        )
    }

    fn resolve_spanned_ref_mut<'a>(
        Spanned(value, span): Spanned<&'a mut T>,
        resolution_target: &str,
    ) -> ExecutionResult<Spanned<&'a mut Self>> {
        Spanned(value, span).try_map(|value| {
            Self::resolve_from_mut(
                value,
                ResolutionContext {
                    span_range: &span,
                    resolution_target,
                },
            )
        })
    }
}

impl ResolvableArgumentTarget for AnyValue {
    type ValueType = AnyType;
}

impl ResolvableOwned<AnyValue> for AnyValue {
    fn resolve_from_value(value: AnyValue, _context: ResolutionContext) -> ExecutionResult<Self> {
        Ok(value)
    }
}
impl ResolvableShared<AnyValue> for AnyValue {
    fn resolve_from_ref<'a>(
        value: &'a AnyValue,
        _context: ResolutionContext,
    ) -> ExecutionResult<&'a Self> {
        Ok(value)
    }
}
impl ResolvableMutable<AnyValue> for AnyValue {
    fn resolve_from_mut<'a>(
        value: &'a mut AnyValue,
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

        impl ResolvableOwned<AnyValue> for $type {
            fn resolve_from_value(
                $value: AnyValue,
                $context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                $body
            }
        }

        impl ResolvableShared<AnyValue> for $type {
            fn resolve_from_ref<'a>(
                $value: &'a AnyValue,
                $context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                $body
            }
        }

        impl ResolvableMutable<AnyValue> for $type {
            fn resolve_from_mut<'a>(
                $value: &'a mut AnyValue,
                $context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                $body
            }
        }
    };
}

pub(crate) use impl_resolvable_argument_for;

macro_rules! impl_delegated_resolvable_argument_for {
    (($value:ident: $delegate:ty) -> $type:ty { $expr:expr }) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = <$delegate as ResolvableArgumentTarget>::ValueType;
        }

        impl ResolvableOwned<Value> for $type {
            fn resolve_from_value(
                input_value: Value,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                let $value: $delegate =
                    ResolvableOwned::<Value>::resolve_from_value(input_value, context)?;
                Ok($expr)
            }
        }

        impl ResolvableShared<Value> for $type {
            fn resolve_from_ref<'a>(
                input_value: &'a Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                let $value: &$delegate =
                    ResolvableShared::<Value>::resolve_from_ref(input_value, context)?;
                Ok(&$expr)
            }
        }

        impl ResolvableMutable<Value> for $type {
            fn resolve_from_mut<'a>(
                input_value: &'a mut Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                let $value: &mut $delegate =
                    ResolvableMutable::<Value>::resolve_from_mut(input_value, context)?;
                Ok(&mut $expr)
            }
        }
    };
}

pub(crate) use impl_delegated_resolvable_argument_for;
