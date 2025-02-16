use crate::internal_prelude::*;

use super::IsVariable;
use std::cell::*;
use std::collections::hash_map::Entry;
use std::rc::Rc;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    variable_data: HashMap<String, VariableData>,
}

#[derive(Clone)]
pub(crate) struct VariableData {
    value: Rc<RefCell<ExpressionValue>>,
    /// In the store, this is the span range of the original let variable declaration.
    /// When this is a variable reference, this is the span range of the variable
    /// which created the reference.
    span_range: SpanRange,
}

impl VariableData {
    fn new(tokens: ExpressionValue, span_range: SpanRange) -> Self {
        Self {
            value: Rc::new(RefCell::new(tokens)),
            span_range,
        }
    }

    pub(crate) fn get_ref(&self) -> ExecutionResult<Ref<ExpressionValue>> {
        self.value.try_borrow().map_err(|_| {
            self.execution_error("The variable cannot be read if it is currently being modified")
        })
    }

    // Gets the cloned expression value, setting the span range appropriately
    pub(crate) fn get_cloned(&self) -> ExecutionResult<ExpressionValue> {
        Ok(self.get_ref()?.clone().with_span_range(self.span_range))
    }

    pub(crate) fn get_mut(&self) -> ExecutionResult<RefMut<ExpressionValue>> {
        self.value.try_borrow_mut().map_err(|_| {
            self.execution_error(
                "The variable cannot be modified if it is already currently being modified",
            )
        })
    }

    pub(crate) fn get_mut_stream(&self) -> ExecutionResult<RefMut<OutputStream>> {
        let mut_guard = self.get_mut()?;
        RefMut::filter_map(mut_guard, |mut_guard| match mut_guard {
            ExpressionValue::Stream(stream) => Some(&mut stream.value),
            _ => None,
        })
        .map_err(|_| self.execution_error("The variable is not a stream"))
    }

    pub(crate) fn set(&self, content: ExpressionValue) -> ExecutionResult<()> {
        *self.get_mut()? = content;
        Ok(())
    }

    pub(crate) fn cheap_clone(&self, span_range: SpanRange) -> Self {
        Self {
            value: self.value.clone(),
            span_range,
        }
    }
}

impl HasSpanRange for VariableData {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            config: Default::default(),
            variable_data: Default::default(),
        }
    }

    pub(crate) fn set_variable(
        &mut self,
        variable: &(impl IsVariable + ?Sized),
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        match self.variable_data.entry(variable.get_name()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().set(value)?;
            }
            Entry::Vacant(entry) => {
                entry.insert(VariableData::new(value, variable.span_range()));
            }
        }
        Ok(())
    }

    pub(crate) fn get_existing_variable_data(
        &self,
        variable: &(impl IsVariable + ?Sized),
        make_error: impl FnOnce() -> SynError,
    ) -> ExecutionResult<VariableData> {
        let data = self
            .variable_data
            .get(&variable.get_name())
            .ok_or_else(make_error)?;
        Ok(data.cheap_clone(variable.span_range()))
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
