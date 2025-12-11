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

    pub(crate) fn error_span_range(&self) -> SpanRange {
        *self.span_range
    }

    /// Create an error for the resolution context.
    pub(crate) fn err<T, V: HasValueKind>(
        &self,
        articled_expected_value_kind: &str,
        value: V,
    ) -> ExecutionResult<T> {
        self.span_range.type_err(format!(
            "{} is expected to be {}, but it is {}",
            self.resolution_target,
            articled_expected_value_kind,
            value.articled_value_type()
        ))
    }
}

pub(crate) trait IsArgument: Sized {
    type ValueType: HierarchicalTypeData;
    const OWNERSHIP: ArgumentOwnership;
    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self>;
}

impl IsArgument for ArgumentValue {
    type ValueType = ValueTypeData;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::AsIs;

    fn from_argument(value: ArgumentValue, _span: SpanRange) -> ExecutionResult<Self> {
        Ok(value)
    }
}

impl<T: ResolvableShared<Value> + ResolvableArgumentTarget + ?Sized> IsArgument for Shared<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        T::resolve_shared(value.expect_shared(), span, "This argument")
    }
}

impl<T: 'static + ?Sized> IsArgument for AnyRef<'static, T>
where
    Shared<T>: IsArgument,
{
    type ValueType = <Shared<T> as IsArgument>::ValueType;
    const OWNERSHIP: ArgumentOwnership = <Shared<T> as IsArgument>::OWNERSHIP;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        Ok(Shared::<T>::from_argument(value, span)?.into())
    }
}

impl<T: ResolvableMutable<Value> + ResolvableArgumentTarget + ?Sized> IsArgument for Assignee<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Assignee { auto_create: false };

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        T::resolve_assignee(value.expect_assignee(), span, "This argument")
    }
}

impl<T: ResolvableMutable<Value> + ResolvableArgumentTarget + ?Sized> IsArgument for Mutable<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        T::resolve_mutable(value.expect_mutable(), span, "This argument")
    }
}

impl<T: 'static + ?Sized> IsArgument for AnyRefMut<'static, T>
where
    Mutable<T>: IsArgument,
{
    type ValueType = <Mutable<T> as IsArgument>::ValueType;
    const OWNERSHIP: ArgumentOwnership = <Mutable<T> as IsArgument>::OWNERSHIP;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        Ok(Mutable::<T>::from_argument(value, span)?.into())
    }
}

impl<T: ResolvableOwned<Value> + ResolvableArgumentTarget> IsArgument for Owned<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        T::resolve_owned(value.expect_owned(), span, "This argument")
    }
}

impl<T: ResolvableOwned<Value> + ResolvableArgumentTarget> IsArgument for T {
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        T::resolve_value(value.expect_owned(), span, "This argument")
    }
}

impl<T: ResolvableShared<Value> + ResolvableArgumentTarget + ToOwned> IsArgument for CopyOnWrite<T>
where
    T::Owned: ResolvableOwned<Value>,
{
    type ValueType = T::ValueType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::CopyOnWrite;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        value.expect_copy_on_write().map(
            |v| T::resolve_shared(v, span, "This argument"),
            |v| <T::Owned as ResolvableOwned<Value>>::resolve_owned(v, span, "This argument"),
        )
    }
}

impl<T: IsArgument> IsArgument for Spanned<T> {
    type ValueType = <T as IsArgument>::ValueType;
    const OWNERSHIP: ArgumentOwnership = <T as IsArgument>::OWNERSHIP;

    fn from_argument(value: ArgumentValue, span: SpanRange) -> ExecutionResult<Self> {
        Ok(Spanned(T::from_argument(value, span)?, span))
    }
}

pub(crate) trait ResolveAs<T> {
    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<T>;
}

impl<T: ResolvableOwned<V>, V> ResolveAs<T> for Spanned<Owned<V>> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<T> {
        let Spanned(value, span) = self;
        T::resolve_value(value, span, resolution_target)
    }
}

// Sadly this can't be changed Value => V because of spurious issues with
// https://github.com/rust-lang/rust/issues/48869
// Instead, we could introduce a different trait ResolveAs2 if needed.
impl<T: ResolvableOwned<Value>> ResolveAs<Owned<T>> for Spanned<Owned<Value>> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Owned<T>> {
        let Spanned(value, span) = self;
        T::resolve_owned(value, span, resolution_target)
    }
}

impl<T: ResolvableShared<Value> + ?Sized> ResolveAs<Shared<T>> for Spanned<Shared<Value>> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Shared<T>> {
        let Spanned(value, span) = self;
        T::resolve_shared(value, span, resolution_target)
    }
}

impl<'a, T: ResolvableShared<Value> + ?Sized> ResolveAs<&'a T> for Spanned<&'a Value> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<&'a T> {
        T::resolve_ref(self, resolution_target)
    }
}

impl<'a, T: ResolvableShared<Value> + ?Sized> ResolveAs<Spanned<&'a T>> for Spanned<&'a Value> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Spanned<&'a T>> {
        T::resolve_spanned_ref(self, resolution_target)
    }
}

impl<T: ResolvableMutable<Value> + ?Sized> ResolveAs<Mutable<T>> for Spanned<Mutable<Value>> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Mutable<T>> {
        let Spanned(value, span) = self;
        T::resolve_mutable(value, span, resolution_target)
    }
}

impl<'a, T: ResolvableMutable<Value> + ?Sized> ResolveAs<&'a mut T> for Spanned<&'a mut Value> {
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<&'a mut T> {
        T::resolve_ref_mut(self, resolution_target)
    }
}

impl<'a, T: ResolvableMutable<Value> + ?Sized> ResolveAs<Spanned<&'a mut T>>
    for Spanned<&'a mut Value>
{
    fn resolve_as(self, resolution_target: &str) -> ExecutionResult<Spanned<&'a mut T>> {
        T::resolve_spanned_ref_mut(self, resolution_target)
    }
}

pub(crate) trait ResolvableArgumentTarget {
    type ValueType: HierarchicalTypeData;
}

pub(crate) trait ResolvableOwned<T>: Sized {
    fn resolve_from_value(value: T, context: ResolutionContext) -> ExecutionResult<Self>;

    fn resolve_owned_from_value(
        value: T,
        context: ResolutionContext,
    ) -> ExecutionResult<Owned<Self>> {
        Self::resolve_from_value(value, context).map(Owned::new)
    }

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_value(
        value: Owned<T>,
        span: SpanRange,
        resolution_target: &str,
    ) -> ExecutionResult<Self> {
        let context = ResolutionContext {
            span_range: &span,
            resolution_target,
        };
        Self::resolve_from_value(value.into_inner(), context)
    }

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_owned(
        value: Owned<T>,
        span: SpanRange,
        resolution_target: &str,
    ) -> ExecutionResult<Owned<Self>> {
        let context = ResolutionContext {
            span_range: &span,
            resolution_target,
        };
        Self::resolve_from_value(value.into_inner(), context).map(Owned::new)
    }
}

pub(crate) trait ResolvableShared<T> {
    fn resolve_from_ref<'a>(value: &'a T, context: ResolutionContext) -> ExecutionResult<&'a Self>;

    /// The `resolution_target` should be capitalized, e.g. "This argument" or "The value destructed with an object pattern"
    fn resolve_shared(
        value: Shared<T>,
        span: SpanRange,
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
        value: Spanned<&'a T>,
        resolution_target: &str,
    ) -> ExecutionResult<&'a Self> {
        let span_range = value.span_range();
        Self::resolve_from_ref(
            *value,
            ResolutionContext {
                span_range: &span_range,
                resolution_target,
            },
        )
    }

    fn resolve_spanned_ref<'a>(
        value: Spanned<&'a T>,
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

pub(crate) trait ResolvableMutable<T> {
    fn resolve_from_mut<'a>(
        value: &'a mut T,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a mut Self>;

    fn resolve_assignee(
        value: Assignee<T>,
        span: SpanRange,
        resolution_target: &str,
    ) -> ExecutionResult<Assignee<Self>> {
        Ok(Assignee(Self::resolve_mutable(
            value.0,
            span,
            resolution_target,
        )?))
    }

    fn resolve_mutable(
        value: Mutable<T>,
        span: SpanRange,
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
        value: Spanned<&'a mut T>,
        resolution_target: &str,
    ) -> ExecutionResult<&'a mut Self> {
        let Spanned(inner, span_range) = value;
        Self::resolve_from_mut(
            inner,
            ResolutionContext {
                span_range: &span_range,
                resolution_target,
            },
        )
    }

    fn resolve_spanned_ref_mut<'a>(
        value: Spanned<&'a mut T>,
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

impl ResolvableArgumentTarget for Value {
    type ValueType = ValueTypeData;
}

impl ResolvableOwned<Value> for Value {
    fn resolve_from_value(value: Value, _context: ResolutionContext) -> ExecutionResult<Self> {
        Ok(value)
    }
}
impl ResolvableShared<Value> for Value {
    fn resolve_from_ref<'a>(
        value: &'a Value,
        _context: ResolutionContext,
    ) -> ExecutionResult<&'a Self> {
        Ok(value)
    }
}
impl ResolvableMutable<Value> for Value {
    fn resolve_from_mut<'a>(
        value: &'a mut Value,
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

        impl ResolvableOwned<Value> for $type {
            fn resolve_from_value(
                $value: Value,
                $context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                $body
            }
        }

        impl ResolvableShared<Value> for $type {
            fn resolve_from_ref<'a>(
                $value: &'a Value,
                $context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                $body
            }
        }

        impl ResolvableMutable<Value> for $type {
            fn resolve_from_mut<'a>(
                $value: &'a mut Value,
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
