use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionIterator {
    iterator: ExpressionIteratorInner,
    #[allow(unused)]
    pub(crate) span_range: SpanRange,
}

impl ExpressionIterator {
    pub(crate) fn new_for_array(array: ExpressionArray) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Array(array.items.into_iter()),
            span_range: array.span_range,
        }
    }

    pub(crate) fn new_for_stream(stream: ExpressionStream) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Stream(stream.value.into_iter()),
            span_range: stream.span_range,
        }
    }

    pub(crate) fn new_for_range(range: ExpressionRange) -> ExecutionResult<Self> {
        let iterator = range.inner.into_iterable()?.resolve_iterator()?;
        Ok(Self {
            iterator: ExpressionIteratorInner::Other(iterator),
            span_range: range.span_range,
        })
    }

    pub(crate) fn new_custom(
        iterator: Box<dyn CustomExpressionIterator>,
        span_range: SpanRange,
    ) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Other(iterator),
            span_range,
        }
    }

    pub(crate) fn singleton_value(mut self) -> Option<ExpressionValue> {
        let first = self.next()?;
        if self.next().is_none() {
            Some(first)
        } else {
            None
        }
    }

    pub(super) fn output_grouped_items_to(self, output: &mut OutputStream) -> ExecutionResult<()> {
        const LIMIT: usize = 10_000;
        let span_range = self.span_range;
        for (i, item) in self.enumerate() {
            if i > LIMIT {
                return span_range.execution_err(format!("Only a maximum of {} items can be output to a stream from an iterator, to protect you from infinite loops. This can't currently be reconfigured with the iteration limit.", LIMIT));
            }
            item.output_to(
                Grouping::Grouped,
                output,
                StreamOutputBehaviour::PermitArrays,
            )?;
        }
        Ok(())
    }

    pub(crate) fn concat_recursive_into(
        self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        if behaviour.output_array_structure {
            output.push_str("[<iterator>");
        }
        let max = self.size_hint().1;
        let span_range = self.span_range;
        for (i, item) in self.enumerate() {
            if i >= behaviour.iterator_limit {
                if behaviour.error_after_iterator_limit {
                    return span_range.execution_err(format!("To protect against infinite loops, only a maximum of {} items can be output to a string from an iterator. Try casting `as stream` to avoid this limit. This can't currently be reconfigured with the iteration limit.", behaviour.iterator_limit));
                } else {
                    if behaviour.output_array_structure {
                        match max {
                            Some(max) => output.push_str(&format!(
                                ", ..<{} further items>",
                                max.saturating_sub(i)
                            )),
                            None => output.push_str(", ..<possibly unbounded>"),
                        }
                    }
                    break;
                }
            }
            if i == 0 {
                output.push(' ');
            }
            if i != 0 && behaviour.output_array_structure {
                output.push(',');
            }
            if i != 0 && behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            item.concat_recursive_into(output, behaviour)?;
        }
        if behaviour.output_array_structure {
            output.push(']');
        }
        Ok(())
    }
}

impl ToExpressionValue for ExpressionIteratorInner {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator {
            iterator: self,
            span_range,
        })
    }
}

impl ToExpressionValue for Box<dyn CustomExpressionIterator> {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator::new_custom(self, span_range))
    }
}

impl ToExpressionValue for ExpressionIterator {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator {
            iterator: self.iterator,
            span_range,
        })
    }
}

impl HasValueType for ExpressionIterator {
    fn value_type(&self) -> &'static str {
        "iterator"
    }
}

#[derive(Clone)]
enum ExpressionIteratorInner {
    Array(<Vec<ExpressionValue> as IntoIterator>::IntoIter),
    Stream(<OutputStream as IntoIterator>::IntoIter),
    Other(Box<dyn CustomExpressionIterator>),
}

impl<T: Iterator<Item = ExpressionValue> + Clone + 'static> CustomExpressionIterator for T {
    fn clone_box(&self) -> Box<dyn CustomExpressionIterator> {
        Box::new(self.clone())
    }
}

pub(crate) trait CustomExpressionIterator: Iterator<Item = ExpressionValue> {
    fn clone_box(&self) -> Box<dyn CustomExpressionIterator>;
}

impl Clone for Box<dyn CustomExpressionIterator> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl Iterator for ExpressionIterator {
    type Item = ExpressionValue;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iterator {
            ExpressionIteratorInner::Array(iter) => iter.next(),
            ExpressionIteratorInner::Stream(iter) => {
                let item = iter.next()?;
                let span = item.span();
                let stream: OutputStream = item.into();
                Some(stream.coerce_into_value(span.span_range()))
            }
            ExpressionIteratorInner::Other(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iterator {
            ExpressionIteratorInner::Array(iter) => iter.size_hint(),
            ExpressionIteratorInner::Stream(iter) => iter.size_hint(),
            ExpressionIteratorInner::Other(iter) => iter.size_hint(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct IteratorTypeData;

impl MethodResolutionTarget for IteratorTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => return None,
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Boolean
                | CastTarget::Char
                | CastTarget::Integer(_)
                | CastTarget::Float(_) => {
                    wrap_unary!([Op=operation](this: Owned<ExpressionIterator>) -> ExecutionResult<ResolvedValue> {
                        let (this, input_span_range) = this.deconstruct();
                        match this.singleton_value() {
                            Some(value) => operation.evaluate(Owned::new(value, input_span_range)),
                            None => input_span_range.execution_err("Only an iterator with one item can be cast to this value")
                        }
                    })
                }
                _ => return None,
            },
        })
    }
}
