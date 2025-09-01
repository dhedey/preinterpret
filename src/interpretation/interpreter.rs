use crate::internal_prelude::*;

use super::IsVariable;
use std::cell::*;
use std::rc::Rc;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    variable_data: VariableData,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            config: Default::default(),
            variable_data: VariableData::new(),
        }
    }

    pub(crate) fn define_variable(
        &mut self,
        variable: &(impl IsVariable + ?Sized),
        value: ExpressionValue,
    ) {
        self.variable_data.define_variable(variable, value)
    }

    pub(crate) fn get_variable_reference(
        &self,
        variable: &(impl IsVariable + ?Sized),
        make_error: impl FnOnce() -> SynError,
    ) -> ExecutionResult<VariableReference> {
        self.variable_data.get_reference(variable, make_error)
    }

    pub(crate) fn start_iteration_counter<'s, S: HasSpanRange>(
        &self,
        span_source: &'s S,
    ) -> IterationCounter<'s, S> {
        IterationCounter {
            span_source,
            count: 0,
            iteration_limit: self.config.iteration_limit,
        }
    }

    pub(crate) fn set_iteration_limit(&mut self, limit: Option<usize>) {
        self.config.iteration_limit = limit;
    }
}

struct VariableData {
    variable_data: HashMap<String, VariableContent>,
}

impl VariableData {
    fn new() -> Self {
        Self {
            variable_data: HashMap::new(),
        }
    }

    fn define_variable(&mut self, variable: &(impl IsVariable + ?Sized), value: ExpressionValue) {
        self.variable_data.insert(
            variable.get_name(),
            VariableContent::new(value, variable.span_range()),
        );
    }

    fn get_reference(
        &self,
        variable: &(impl IsVariable + ?Sized),
        make_error: impl FnOnce() -> SynError,
    ) -> ExecutionResult<VariableReference> {
        let reference = self
            .variable_data
            .get(&variable.get_name())
            .ok_or_else(make_error)?
            .create_reference(variable);
        Ok(reference)
    }
}

struct VariableContent {
    value: Rc<RefCell<ExpressionValue>>,
    #[allow(unused)]
    definition_span_range: SpanRange,
}

impl VariableContent {
    fn new(tokens: ExpressionValue, definition_span_range: SpanRange) -> Self {
        Self {
            value: Rc::new(RefCell::new(tokens)),
            definition_span_range,
        }
    }

    fn create_reference(&self, variable: &(impl IsVariable + ?Sized)) -> VariableReference {
        VariableReference {
            data: self.value.clone(),
            variable_span_range: variable.span_range(),
        }
    }
}

pub(crate) struct IterationCounter<'a, S: HasSpanRange> {
    span_source: &'a S,
    count: usize,
    iteration_limit: Option<usize>,
}

impl<S: HasSpanRange> IterationCounter<'_, S> {
    pub(crate) fn increment_and_check(&mut self) -> ExecutionResult<()> {
        self.count += 1;
        self.check()
    }

    pub(crate) fn check(&self) -> ExecutionResult<()> {
        if let Some(limit) = self.iteration_limit {
            if self.count > limit {
                return self.span_source.execution_err(format!("Iteration limit of {} exceeded.\nIf needed, the limit can be reconfigured with [!settings! {{ iteration_limit: X }}]", limit));
            }
        }
        Ok(())
    }
}

pub(crate) struct InterpreterConfig {
    iteration_limit: Option<usize>,
}

pub(crate) const DEFAULT_ITERATION_LIMIT: usize = 1000;
pub(crate) const DEFAULT_ITERATION_LIMIT_STR: &str = "1000";

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            iteration_limit: Some(DEFAULT_ITERATION_LIMIT),
        }
    }
}

#[derive(Clone)]
pub(crate) struct VariableReference {
    data: Rc<RefCell<ExpressionValue>>,
    variable_span_range: SpanRange,
}

impl VariableReference {
    pub(crate) fn get_value_ref(&self) -> ExecutionResult<Ref<'_, ExpressionValue>> {
        self.data.try_borrow().map_err(|_| {
            self.execution_error("The variable cannot be read if it is currently being modified")
        })
    }

    // Gets the cloned expression value, setting the span range appropriately
    pub(crate) fn get_value_cloned(&self) -> ExecutionResult<ExpressionValue> {
        Ok(self
            .get_value_ref()?
            .clone()
            .with_span_range(self.variable_span_range))
    }

    pub(crate) fn into_mut(self) -> ExecutionResult<MutableValueReference> {
        MutableValueReference::new_from_variable(self)
    }

    pub(crate) fn into_shared(self) -> ExecutionResult<SharedValueReference> {
        SharedValueReference::new_from_variable(self)
    }
}

impl HasSpanRange for VariableReference {
    fn span_range(&self) -> SpanRange {
        self.variable_span_range
    }
}

pub(crate) type MutableValueReference = MutableSubPlace<ExpressionValue>;

/// A mutable reference to a value inside a variable; along with a span of the whole access.
/// For example, for `x.y[4]`, this captures both:
/// * The mutable reference to the location under `x`
/// * The lexical span of the tokens `x.y[4]`
pub(crate) struct MutableSubPlace<T: 'static> {
    mut_cell: MutSubRcRefCell<ExpressionValue, T>,
    span_range: SpanRange,
}

impl MutableSubPlace<ExpressionValue> {
    pub(crate) fn new_from_owned(value: ExpressionValue) -> Self {
        let span_range = value.span_range();
        Self {
            // Unwrap is safe because it's a new refcell
            mut_cell: MutSubRcRefCell::new(Rc::new(RefCell::new(value))).unwrap(),
            span_range,
        }
    }

    fn new_from_variable(reference: VariableReference) -> ExecutionResult<Self> {
        Ok(Self {
            mut_cell: MutSubRcRefCell::new(reference.data).map_err(|_| {
                reference.variable_span_range.execution_error(
                    "The variable cannot be modified as it is already being modified",
                )
            })?,
            span_range: reference.variable_span_range,
        })
    }

    pub(crate) fn into_stream(self) -> ExecutionResult<MutableSubPlace<OutputStream>> {
        let stream = self.mut_cell.try_map(|value| match value {
            ExpressionValue::Stream(stream) => Ok(&mut stream.value),
            _ => self
                .span_range
                .execution_err("The variable is not a stream"),
        })?;
        Ok(MutableSubPlace {
            mut_cell: stream,
            span_range: self.span_range,
        })
    }

    pub(crate) fn resolve_indexed_with_autocreate(
        self,
        access: IndexAccess,
        index: ExpressionValue,
    ) -> ExecutionResult<Self> {
        let indexed = self
            .mut_cell
            .try_map(|value| value.index_mut_with_autocreate(access, index))?;
        Ok(Self {
            mut_cell: indexed,
            span_range: SpanRange::new_between(self.span_range.start(), access.span()),
        })
    }

    pub(crate) fn resolve_property(self, access: PropertyAccess) -> ExecutionResult<Self> {
        let span_range = SpanRange::new_between(self.span_range.start(), access.span_range());
        let indexed = self.mut_cell.try_map(|value| value.property_mut(access))?;
        Ok(Self {
            mut_cell: indexed,
            span_range,
        })
    }

    pub(crate) fn set(&mut self, content: impl ToExpressionValue) {
        *self.mut_cell = content.to_value(self.span_range);
    }
}

impl<T> AsMut<T> for MutableSubPlace<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.mut_cell
    }
}

impl<T> AsRef<T> for MutableSubPlace<T> {
    fn as_ref(&self) -> &T {
        &self.mut_cell
    }
}

impl<T: 'static> HasSpanRange for MutableSubPlace<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl WithSpanExt for MutableSubPlace<ExpressionValue> {
    fn with_span(self, span: Span) -> Self {
        Self {
            mut_cell: self.mut_cell,
            span_range: SpanRange::new_single(span),
        }
    }
}

pub(crate) type SharedValueReference = SharedSubPlace<ExpressionValue>;

/// A mutable reference to a value inside a variable; along with a span of the whole access.
/// For example, for `x.y[4]`, this captures both:
/// * The mutable reference to the location under `x`
/// * The lexical span of the tokens `x.y[4]`
pub(crate) struct SharedSubPlace<T: 'static> {
    shared_cell: SharedSubRcRefCell<ExpressionValue, T>,
    span_range: SpanRange,
}

impl SharedSubPlace<ExpressionValue> {
    pub(crate) fn new_from_owned(value: ExpressionValue) -> Self {
        let span_range = value.span_range();
        Self {
            // Unwrap is safe because it's a new refcell
            shared_cell: SharedSubRcRefCell::new(Rc::new(RefCell::new(value))).unwrap(),
            span_range,
        }
    }

    fn new_from_variable(reference: VariableReference) -> ExecutionResult<Self> {
        Ok(Self {
            shared_cell: SharedSubRcRefCell::new(reference.data).map_err(|_| {
                reference
                    .variable_span_range
                    .execution_error("The variable cannot be read as it is already being modified")
            })?,
            span_range: reference.variable_span_range,
        })
    }

    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: &ExpressionValue,
    ) -> ExecutionResult<Self> {
        let indexed = self
            .shared_cell
            .try_map(|value| value.index_ref(access, index))?;
        Ok(Self {
            shared_cell: indexed,
            span_range: SpanRange::new_between(self.span_range.start(), access.span()),
        })
    }

    pub(crate) fn resolve_property(self, access: PropertyAccess) -> ExecutionResult<Self> {
        let span_range = SpanRange::new_between(self.span_range.start(), access.span_range());
        let indexed = self
            .shared_cell
            .try_map(|value| value.property_ref(access))?;
        Ok(Self {
            shared_cell: indexed,
            span_range,
        })
    }
}

impl<T> AsRef<T> for SharedSubPlace<T> {
    fn as_ref(&self) -> &T {
        &self.shared_cell
    }
}

impl<T: 'static> HasSpanRange for SharedSubPlace<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl WithSpanExt for SharedSubPlace<ExpressionValue> {
    fn with_span(self, span: Span) -> Self {
        Self {
            shared_cell: self.shared_cell,
            span_range: SpanRange::new_single(span),
        }
    }
}
