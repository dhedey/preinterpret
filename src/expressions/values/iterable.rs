use super::*;

pub(crate) trait IsIterable: 'static {
    fn into_iterator(self: Box<Self>) -> FunctionResult<IteratorValue>;
    fn iterable_len(&self, error_span_range: SpanRange) -> FunctionResult<usize>;
}

define_dyn_type!(
    pub(crate) IterableType,
    content: dyn IsIterable,
    dyn_kind: DynTypeKind::Iterable,
    type_name: "iterable",
    articled_value_name: "an iterable (e.g. array, list, etc.)",
);

pub(crate) type IterableValue = Box<dyn IsIterable>;
pub(crate) type IterableAnyRef<'a> = AnyRef<'a, dyn IsIterable>;

define_type_features! {
    impl IterableType,
    pub(crate) mod iterable_interface {
        methods {
            fn into_iter(this: IterableValue) -> FunctionResult<IteratorValue> {
                this.into_iterator()
            }

            fn len(Spanned(this, span_range): Spanned<IterableAnyRef>) -> FunctionResult<usize> {
                this.iterable_len(span_range)
            }

            fn is_empty(Spanned(this, span_range): Spanned<IterableAnyRef>) -> FunctionResult<bool> {
                Ok(this.iterable_len(span_range)? == 0)
            }

            [context] fn zip(this: IterableValue) -> FunctionResult<ArrayValue> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range(), context.interpreter)?.run_zip(context.interpreter, true)
            }

            [context] fn zip_truncated(this: IterableValue) -> FunctionResult<ArrayValue> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range(), context.interpreter)?.run_zip(context.interpreter, false)
            }

            [context] fn intersperse(this: IterableValue, separator: AnyValue, settings: Option<IntersperseSettings>) -> FunctionResult<ArrayValue> {
                run_intersperse(this, separator, settings.unwrap_or_default(), context.interpreter)
            }

            [context] fn to_vec(this: IterableValue) -> FunctionResult<Vec<AnyValue>> {
                let error_span_range = context.span_range();
                let mut counter = context.interpreter.start_iteration_counter(&error_span_range);
                let mut iterator = this.into_iterator()?;
                let max_hint = iterator.do_size_hint().1;
                let mut vec = if let Some(max) = max_hint {
                    Vec::with_capacity(max)
                } else {
                    Vec::new()
                };
                loop {
                    let item = match iterator.do_next(context.interpreter)? {
                        Some(item) => item,
                        None => break,
                    };
                    counter.increment_and_check()?;
                    vec.push(item);
                }
                Ok(vec)
            }
        }
        unary_operations {
            fn cast_into_iterator(this: IterableValue) -> FunctionResult<IteratorValue> {
                this.into_iterator()
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast { target: CastTarget(AnyValueLeafKind::Iterator(_)), .. } =>{
                        unary_definitions::cast_into_iterator()
                    }
                    _ => return None,
                })
            }
        }
    }
}
