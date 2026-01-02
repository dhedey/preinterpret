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
