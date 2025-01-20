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

    pub(crate) fn config(&self) -> &InterpreterConfig {
        &self.config
    }

    pub(crate) fn mut_config(&mut self) -> &mut InterpreterConfig {
        &mut self.config
    }
}

pub(crate) struct InterpreterConfig {
    iteration_limit: Option<usize>,
}

pub(crate) const DEFAULT_ITERATION_LIMIT: usize = 10000;

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            iteration_limit: Some(DEFAULT_ITERATION_LIMIT),
        }
    }
}

impl InterpreterConfig {
    pub(crate) fn set_iteration_limit(&mut self, limit: Option<usize>) {
        self.iteration_limit = limit;
    }

    pub(crate) fn check_iteration_count(
        &self,
        span_source: &impl HasSpanRange,
        count: usize,
    ) -> Result<()> {
        if let Some(limit) = self.iteration_limit {
            if count > limit {
                return span_source.err(format!("Iteration limit of {} exceeded", limit));
            }
        }
        Ok(())
    }
}
