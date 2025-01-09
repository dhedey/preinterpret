use crate::internal_prelude::*;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    variables: HashMap<String, InterpretedStream>,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            config: Default::default(),
            variables: Default::default(),
        }
    }

    pub(crate) fn set_variable(&mut self, name: String, tokens: InterpretedStream) {
        self.variables.insert(name, tokens);
    }

    pub(crate) fn get_variable(&self, name: &str) -> Option<&InterpretedStream> {
        self.variables.get(name)
    }

    pub(crate) fn config(&self) -> &InterpreterConfig {
        &self.config
    }
}

pub(crate) struct InterpreterConfig {
    iteration_limit: Option<usize>,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            iteration_limit: Some(10000),
        }
    }
}

impl InterpreterConfig {
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
