use super::*;

// If you add a new variant, also update:
// * ResolvableArgumentOwned for IterableValue
// * IsArgument for IterableRef
// * The parent of the value's TypeData to be IterableTypeData
pub(crate) enum IterableValue {
    Iterator(IteratorValue),
    Array(ArrayValue),
    Stream(StreamValue),
    Object(ObjectValue),
    Range(RangeValue),
    String(StringValue),
}

impl ResolvableArgumentTarget for IterableValue {
    type ValueType = IterableTypeData;
}

impl ResolvableArgumentOwned for IterableValue {
    fn resolve_from_value(value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
        Ok(match value {
            Value::Array(x) => Self::Array(x),
            Value::Object(x) => Self::Object(x),
            Value::Stream(x) => Self::Stream(x),
            Value::Range(x) => Self::Range(x),
            Value::Iterator(x) => Self::Iterator(x),
            Value::String(x) => Self::String(x),
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
            fn into_iter(this: IterableValue) -> ExecutionResult<IteratorValue> {
                this.into_iterator()
            }

            fn len(this: Spanned<IterableRef>) -> ExecutionResult<usize> {
                this.len()
            }

            fn is_empty(this: Spanned<IterableRef>) -> ExecutionResult<bool> {
                Ok(this.len()? == 0)
            }

            [context] fn zip(this: IterableValue) -> ExecutionResult<ArrayValue> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, true)
            }

            [context] fn zip_truncated(this: IterableValue) -> ExecutionResult<ArrayValue> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, false)
            }

            fn intersperse(this: IterableValue, separator: Value, settings: Option<IntersperseSettings>) -> ExecutionResult<ArrayValue> {
                run_intersperse(this, separator, settings.unwrap_or_default())
            }

            [context] fn to_vec(this: IterableValue) -> ExecutionResult<Vec<Value>> {
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
    pub(crate) fn into_iterator(self) -> ExecutionResult<IteratorValue> {
        Ok(match self {
            IterableValue::Array(value) => IteratorValue::new_for_array(value),
            IterableValue::Stream(value) => IteratorValue::new_for_stream(value),
            IterableValue::Iterator(value) => value,
            IterableValue::Range(value) => IteratorValue::new_for_range(value)?,
            IterableValue::Object(value) => IteratorValue::new_for_object(value)?,
            IterableValue::String(value) => IteratorValue::new_for_string(value)?,
        })
    }
}

pub(crate) enum IterableRef<'a> {
    Iterator(AnyRef<'a, IteratorValue>),
    Array(AnyRef<'a, ArrayValue>),
    Stream(AnyRef<'a, OutputStream>),
    Range(AnyRef<'a, RangeValue>),
    Object(AnyRef<'a, ObjectValue>),
    String(AnyRef<'a, str>),
}

impl IsArgument for IterableRef<'static> {
    type ValueType = IterableTypeData;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument(value: ArgumentValue) -> ExecutionResult<Self> {
        Ok(match value.kind() {
            ValueKind::Iterator => IterableRef::Iterator(IsArgument::from_argument(value)?),
            ValueKind::Array => IterableRef::Array(IsArgument::from_argument(value)?),
            ValueKind::Stream => IterableRef::Stream(IsArgument::from_argument(value)?),
            ValueKind::Range => IterableRef::Range(IsArgument::from_argument(value)?),
            ValueKind::Object => IterableRef::Object(IsArgument::from_argument(value)?),
            ValueKind::String => IterableRef::String(IsArgument::from_argument(value)?),
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
