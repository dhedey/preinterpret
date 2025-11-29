use super::*;

// If you add a new variant, also update:
// * ResolvableArgumentOwned for IterableValue
// * FromResolved for IterableRef
// * The parent of the value's TypeData to be IterableTypeData
pub(crate) enum IterableValue {
    Iterator(IteratorExpression),
    Array(ArrayExpression),
    Stream(StreamExpression),
    Object(ObjectExpression),
    Range(RangeExpression),
    String(StringExpression),
}

impl ResolvableArgumentTarget for IterableValue {
    type ValueType = IterableTypeData;
}

impl ResolvableArgumentOwned for IterableValue {
    fn resolve_from_value(
        value: ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        Ok(match value {
            ExpressionValue::Array(x) => Self::Array(x),
            ExpressionValue::Object(x) => Self::Object(x),
            ExpressionValue::Stream(x) => Self::Stream(x),
            ExpressionValue::Range(x) => Self::Range(x),
            ExpressionValue::Iterator(x) => Self::Iterator(x),
            ExpressionValue::String(x) => Self::String(x),
            _ => {
                return context.err(
                    "iterable (iterator, array, object, stream, range or string)",
                    value,
                );
            }
        })
    }
}

define_interface! {
    struct IterableTypeData,
    parent: ValueTypeData,
    pub(crate) mod iterable_interface {
        pub(crate) mod methods {
            fn into_iter(this: IterableValue) -> ExecutionResult<IteratorExpression> {
                this.into_iterator()
            }

            fn len(this: Spanned<IterableRef>) -> ExecutionResult<usize> {
                this.len()
            }

            fn is_empty(this: Spanned<IterableRef>) -> ExecutionResult<bool> {
                Ok(this.len()? == 0)
            }

            [context] fn zip(this: IterableValue) -> ExecutionResult<ArrayExpression> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, true)
            }

            [context] fn zip_truncated(this: IterableValue) -> ExecutionResult<ArrayExpression> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, false)
            }

            fn intersperse(this: IterableValue, separator: ExpressionValue, settings: Option<IntersperseSettings>) -> ExecutionResult<ArrayExpression> {
                run_intersperse(this, separator, settings.unwrap_or_default())
            }

            [context] fn to_vec(this: IterableValue) -> ExecutionResult<Vec<ExpressionValue>> {
                let error_span_range = context.span_range();
                let mut counter = context.interpreter.start_iteration_counter(&error_span_range);
                let iterator = this.into_iterator()?;
                let max_hint = iterator.size_hint().1;
                let mut vec = if let Some(max) = max_hint {
                    Vec::with_capacity(max)
                } else {
                    Vec::new()
                };
                for item in iterator {
                    counter.increment_and_check()?;
                    vec.push(item);
                }
                Ok(vec)
            }
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {}
        interface_items {
        }
    }
}

impl IterableValue {
    pub(crate) fn into_iterator(self) -> ExecutionResult<IteratorExpression> {
        Ok(match self {
            IterableValue::Array(value) => IteratorExpression::new_for_array(value),
            IterableValue::Stream(value) => IteratorExpression::new_for_stream(value),
            IterableValue::Iterator(value) => value,
            IterableValue::Range(value) => IteratorExpression::new_for_range(value)?,
            IterableValue::Object(value) => IteratorExpression::new_for_object(value)?,
            IterableValue::String(value) => IteratorExpression::new_for_string(value)?,
        })
    }
}

pub(crate) enum IterableRef<'a> {
    Iterator(AnyRef<'a, IteratorExpression>),
    Array(AnyRef<'a, ArrayExpression>),
    Stream(AnyRef<'a, OutputStream>),
    Range(AnyRef<'a, RangeExpression>),
    Object(AnyRef<'a, ObjectExpression>),
    String(AnyRef<'a, str>),
}

impl FromResolved for IterableRef<'static> {
    type ValueType = IterableTypeData;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Shared;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(match value.kind() {
            ValueKind::Iterator => IterableRef::Iterator(FromResolved::from_resolved(value)?),
            ValueKind::Array => IterableRef::Array(FromResolved::from_resolved(value)?),
            ValueKind::Stream => IterableRef::Stream(FromResolved::from_resolved(value)?),
            ValueKind::Range => IterableRef::Range(FromResolved::from_resolved(value)?),
            ValueKind::Object => IterableRef::Object(FromResolved::from_resolved(value)?),
            ValueKind::String => IterableRef::String(FromResolved::from_resolved(value)?),
            _ => {
                return value.type_err(
                    "Expected iterable (iterator, array, object, stream, range or string)",
                )
            }
        })
    }
}

impl Spanned<IterableRef<'_>> {
    pub(crate) fn len(&self) -> ExecutionResult<usize> {
        match &self.value {
            IterableRef::Iterator(iterator) => iterator.len(self.span_range),
            IterableRef::Array(value) => Ok(value.items.len()),
            IterableRef::Stream(value) => Ok(value.len()),
            IterableRef::Range(value) => value.len(self.span_range),
            IterableRef::Object(value) => Ok(value.entries.len()),
            // NB - this is different to string.len() which counts bytes
            IterableRef::String(value) => Ok(value.chars().count()),
        }
    }
}
