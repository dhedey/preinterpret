use super::*;

pub(crate) trait IsIterable: 'static {
    fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue>;
    fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize>;
}

define_dyn_type!(
    dyn IsIterable => "an iterable (e.g. array, list, etc.)",
    pub(crate) IterableType
);
