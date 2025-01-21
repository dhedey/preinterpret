use crate::internal_prelude::*;

use super::IsVariable;
use std::cell::*;
use std::collections::hash_map::Entry;
use std::rc::Rc;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    variable_data: HashMap<String, VariableData>,
    loop_condition: LoopCondition,
}

#[derive(Clone, Copy)]
#[must_use]
pub(crate) enum LoopCondition {
    None,
    Continue,
    Break,
}

#[derive(Clone)]
pub(crate) struct VariableData {
    value: Rc<RefCell<InterpretedStream>>,
}

impl VariableData {
    fn new(tokens: InterpretedStream) -> Self {
        Self {
            value: Rc::new(RefCell::new(tokens)),
        }
    }

    pub(crate) fn get<'d>(
        &'d self,
        variable: &impl IsVariable,
    ) -> Result<Ref<'d, InterpretedStream>> {
        self.value.try_borrow().map_err(|_| {
            variable.error("The variable cannot be read if it is currently being modified")
        })
    }

    pub(crate) fn get_mut<'d>(
        &'d self,
        variable: &impl IsVariable,
    ) -> Result<RefMut<'d, InterpretedStream>> {
        self.value.try_borrow_mut().map_err(|_| {
            variable
                .error("The variable cannot be modified if it is already currently being modified")
        })
    }

    pub(crate) fn set(&self, variable: &impl IsVariable, content: InterpretedStream) -> Result<()> {
        *self.get_mut(variable)? = content;
        Ok(())
    }

    pub(crate) fn cheap_clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            config: Default::default(),
            variable_data: Default::default(),
            loop_condition: LoopCondition::None,
        }
    }

    pub(super) fn set_variable(
        &mut self,
        variable: &impl IsVariable,
        tokens: InterpretedStream,
    ) -> Result<()> {
        match self.variable_data.entry(variable.get_name()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().set(variable, tokens)?;
            }
            Entry::Vacant(entry) => {
                entry.insert(VariableData::new(tokens));
            }
        }
        Ok(())
    }

    pub(crate) fn get_existing_variable_data(
        &self,
        variable: &impl IsVariable,
        make_error: impl FnOnce() -> Error,
    ) -> Result<&VariableData> {
        self.variable_data
            .get(&variable.get_name())
            .ok_or_else(make_error)
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

    /// Panics if called with [`LoopCondition::None`]
    pub(crate) fn start_loop_action(&mut self, source: Span, action: LoopCondition) -> Error {
        self.loop_condition = action;
        match action {
            LoopCondition::None => panic!("Not allowed"),
            LoopCondition::Continue => {
                source.error("The continue command is only allowed inside a loop")
            }
            LoopCondition::Break => source.error("The break command is only allowed inside a loop"),
        }
    }

    pub(crate) fn outstanding_loop_condition(&mut self) -> LoopCondition {
        std::mem::replace(&mut self.loop_condition, LoopCondition::None)
    }
}

pub(crate) struct IterationCounter<'a, S: HasSpanRange> {
    span_source: &'a S,
    count: usize,
    iteration_limit: Option<usize>,
}

impl<S: HasSpanRange> IterationCounter<'_, S> {
    pub(crate) fn add_and_check(&mut self, count: usize) -> Result<()> {
        self.count = self.count.wrapping_add(count);
        self.check()
    }

    pub(crate) fn increment_and_check(&mut self) -> Result<()> {
        self.count += 1;
        self.check()
    }

    pub(crate) fn check(&self) -> Result<()> {
        if let Some(limit) = self.iteration_limit {
            if self.count > limit {
                return self.span_source.err(format!("Iteration limit of {} exceeded.\nIf needed, the limit can be reconfigured with [!settings! {{ iteration_limit: X }}]", limit));
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
