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
    articled_value_name: "an iterable (e.g. array, list, etc.)",
);

pub(crate) type IterableValue = Box<dyn IsIterable>;
pub(crate) type IterableAnyRef<'a> = AnyRef<'a, dyn IsIterable>;

define_type_features! {
    impl IterableType,
    pub(crate) mod iterable_interface {
        methods {
            fn into_iter(this: IterableValue) -> ExecutionResult<IteratorValue> {
                this.into_iterator()
            }

            fn len(Spanned(this, span_range): Spanned<IterableAnyRef>) -> ExecutionResult<usize> {
                this.len(span_range)
            }

            fn is_empty(Spanned(this, span_range): Spanned<IterableAnyRef>) -> ExecutionResult<bool> {
                Ok(this.len(span_range)? == 0)
            }

            [context] fn zip(this: IterableValue) -> ExecutionResult<ArrayValue> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, true)
            }

            [context] fn zip_truncated(this: IterableValue) -> ExecutionResult<ArrayValue> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, false)
            }

            fn intersperse(this: IterableValue, separator: AnyValue, settings: Option<IntersperseSettings>) -> ExecutionResult<ArrayValue> {
                run_intersperse(this, separator, settings.unwrap_or_default())
            }

            [context] fn to_vec(this: IterableValue) -> ExecutionResult<Vec<AnyValue>> {
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
        unary_operations {
            fn cast_into_iterator(this: IterableValue) -> ExecutionResult<IteratorValue> {
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
