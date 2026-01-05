use super::*;

pub(crate) trait IsIterable: 'static {
    fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue>;
    fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize>;
}

define_dyn_type!(
    pub(crate) IterableType,
    content: dyn IsIterable,
    dyn_kind: DynTypeKind::Iterable,
    type_name: "iterable",
    articled_display_name: "an iterable (e.g. array, list, etc.)",
);

// If you add a new variant, also update:
// * ResolvableOwned<Value> for IterableValue
// * IsArgument for IterableRef
// * The parent of the value's TypeData to be IterableType
pub(crate) enum IterableValue {
    Iterator(IteratorValue),
    Array(ArrayValue),
    Stream(OutputStream),
    Object(ObjectValue),
    Range(RangeValue),
    String(String),
}

impl ResolvableArgumentTarget for IterableValue {
    type ValueType = IterableType;
}

impl ResolvableOwned<Value> for IterableValue {
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
                    "an iterable (iterator, array, object, stream, range or string)",
                    value,
                );
            }
        })
    }
}

define_type_features! {
    impl IterableType,
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
            fn cast_into_iterator(this: IterableValue) -> ExecutionResult<IteratorValue> {
                this.into_iterator()
            }
        }
        pub(crate) mod binary_operations {}
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast { target: CastTarget(ValueLeafKind::Iterator(_)), .. } =>{
                        unary_definitions::cast_into_iterator()
                    }
                    _ => return None,
                })
            }
        }
    }
}

impl IterableValue {
    pub(crate) fn into_iterator(self) -> ExecutionResult<IteratorValue> {
        match self {
            IterableValue::Array(value) => Box::new(value).into_iterator(),
            IterableValue::Stream(value) => Box::new(value).into_iterator(),
            IterableValue::Iterator(value) => Box::new(value).into_iterator(),
            IterableValue::Range(value) => Box::new(value).into_iterator(),
            IterableValue::Object(value) => Box::new(value).into_iterator(),
            IterableValue::String(value) => Box::new(value).into_iterator(),
        }
    }
}

pub(crate) enum IterableRef<'a> {
    Iterator(AnyRef<'a, IteratorValue>),
    Array(AnyRef<'a, ArrayValue>),
    Stream(AnyRef<'a, OutputStream>),
    Range(AnyRef<'a, RangeValue>),
    Object(AnyRef<'a, ObjectValue>),
    String(AnyRef<'a, String>),
}

impl IsArgument for IterableRef<'static> {
    type ValueType = IterableType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        Ok(match argument.kind() {
            ValueLeafKind::Iterator(_) => {
                IterableRef::Iterator(IsArgument::from_argument(argument)?)
            }
            ValueLeafKind::Array(_) => IterableRef::Array(IsArgument::from_argument(argument)?),
            ValueLeafKind::Stream(_) => IterableRef::Stream(IsArgument::from_argument(argument)?),
            ValueLeafKind::Range(_) => IterableRef::Range(IsArgument::from_argument(argument)?),
            ValueLeafKind::Object(_) => IterableRef::Object(IsArgument::from_argument(argument)?),
            ValueLeafKind::String(_) => IterableRef::String(IsArgument::from_argument(argument)?),
            _ => {
                return argument.type_err(
                    "Expected iterable (iterator, array, object, stream, range or string)",
                );
            }
        })
    }
}

impl Spanned<IterableRef<'_>> {
    pub(crate) fn len(&self) -> ExecutionResult<usize> {
        let Spanned(value, span) = self;
        let span = *span;
        match value {
            IterableRef::Iterator(iterator) => iterator.len(span),
            IterableRef::Array(value) => value.len(span),
            IterableRef::Stream(value) => <OutputStream as IsIterable>::len(value, span),
            IterableRef::Range(value) => value.len(span),
            IterableRef::Object(value) => value.len(span),
            IterableRef::String(value) => <String as IsIterable>::len(value, span),
        }
    }
}
