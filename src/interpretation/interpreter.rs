use super::*;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    variable_data: VariableData,
}

impl Interpreter {
    pub(crate) fn new(_scope_definitions: ScopeDefinitions) -> Self {
        Self {
            config: Default::default(),
            variable_data: VariableData::new(),
        }
    }

    pub(crate) fn define_variable(
        &mut self,
        variable: &VariableDefinition,
        value: ExpressionValue,
    ) {
        self.variable_data.define_variable(variable, value)
    }

    pub(crate) fn resolve_variable_binding(
        &self,
        variable: &VariableReference,
        make_error: impl FnOnce() -> SynError,
    ) -> ExecutionResult<VariableBinding> {
        self.variable_data.resolve_binding(variable, make_error)
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

    fn define_variable(&mut self, variable: &VariableDefinition, value: ExpressionValue) {
        self.variable_data.insert(
            variable.get_name(),
            VariableContent::new(value, variable.span_range()),
        );
    }

    fn resolve_binding(
        &self,
        variable: &VariableReference,
        make_error: impl FnOnce() -> SynError,
    ) -> ExecutionResult<VariableBinding> {
        let reference = self
            .variable_data
            .get(&variable.get_name())
            .ok_or_else(make_error)?
            .binding(variable);
        Ok(reference)
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
