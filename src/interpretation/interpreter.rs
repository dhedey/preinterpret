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
    pub(crate) fn get_value_ref(&self) -> ExecutionResult<Ref<ExpressionValue>> {
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

    pub(crate) fn into_mut(self) -> ExecutionResult<MutableReference<ExpressionValue>> {
        MutableReference::new(self)
    }
}

impl HasSpanRange for VariableReference {
    fn span_range(&self) -> SpanRange {
        self.variable_span_range
    }
}

pub(crate) struct MutableReference<T: 'static> {
    mut_cell: MutRcRefCell<ExpressionValue, T>,
    span_range: SpanRange,
}

impl MutableReference<ExpressionValue> {
    fn new(reference: VariableReference) -> ExecutionResult<Self> {
        Ok(Self {
            mut_cell: MutRcRefCell::new(reference.data).map_err(|_| {
                reference.variable_span_range.execution_error(
                    "The variable cannot be modified as it is already being modified",
                )
            })?,
            span_range: reference.variable_span_range,
        })
    }

    pub(crate) fn into_stream(self) -> ExecutionResult<MutableReference<OutputStream>> {
        let stream = self.mut_cell.try_map(|value| match value {
            ExpressionValue::Stream(stream) => Ok(&mut stream.value),
            _ => self
                .span_range
                .execution_err("The variable is not a stream"),
        })?;
        Ok(MutableReference {
            mut_cell: stream,
            span_range: self.span_range,
        })
    }

    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: ExpressionValue,
    ) -> ExecutionResult<Self> {
        let indexed = self
            .mut_cell
            .try_map(|value| value.index_mut(access, index))?;
        Ok(Self {
            mut_cell: indexed,
            span_range: SpanRange::new_between(self.span_range.start(), access.span()),
        })
    }

    pub(crate) fn set(&mut self, content: impl ToExpressionValue) {
        *self.mut_cell = content.to_value(self.span_range);
    }
}

impl<T: 'static> MutableReference<T> {
    pub(crate) fn value_mut(&mut self) -> &mut T {
        &mut self.mut_cell
    }
}

impl<T: 'static> HasSpanRange for MutableReference<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}
