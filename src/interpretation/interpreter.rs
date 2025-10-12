use super::*;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    scope_definitions: ScopeDefinitions,
    scopes: Vec<RuntimeScope>,
}

impl Interpreter {
    pub(crate) fn new(scope_definitions: ScopeDefinitions) -> Self {
        let root_scope_id = scope_definitions.root_scope;
        let mut interpreter = Self {
            config: Default::default(),
            scope_definitions,
            scopes: vec![],
        };
        interpreter.enter_scope_inner(root_scope_id, false);
        interpreter
    }

    fn scope_mut(&mut self, id: ScopeId) -> &mut RuntimeScope {
        self.scopes
            .iter_mut()
            .rev()
            .find(|s| s.id == id)
            .expect("Scope data not found in stack")
    }

    pub(crate) fn current_scope_id(&self) -> ScopeId {
        self.scopes.last().unwrap().id
    }

    pub(crate) fn enter_scope(&mut self, id: ScopeId) {
        self.enter_scope_inner(id, true);
    }

    fn enter_scope_inner(&mut self, id: ScopeId, check_parent: bool) {
        let new_scope = self.scope_definitions.scopes.get(id);
        if check_parent {
            assert!(new_scope.parent == Some(self.current_scope_id()));
        }
        let variables = {
            let mut map = HashMap::new();
            for definition_id in new_scope.definitions.iter() {
                map.insert(*definition_id, VariableContent::Uninitialized);
            }
            map
        };
        self.scopes.push(RuntimeScope { id, variables });
    }

    pub(crate) fn exit_scope(&mut self, scope_id: ScopeId) {
        assert!(scope_id == self.current_scope_id());
        self.scopes.pop();
    }

    pub(crate) fn catch_control_flow<T>(
        &mut self,
        input: ExecutionResult<T>,
        should_catch: impl FnOnce(&ControlFlowInterrupt) -> bool,
        return_to_scope: ScopeId,
    ) -> ExecutionResult<ExecutionOutcome<T>> {
        let output = match input {
            Ok(value) => Ok(ExecutionOutcome::Value(value)),
            Err(interrupt) => interrupt.into_outcome::<T>(should_catch),
        };
        if let Ok(ExecutionOutcome::ControlFlow(_)) = &output {
            self.handle_catch(return_to_scope)
        }
        output
    }

    fn handle_catch(&mut self, result_scope: ScopeId) {
        while self.current_scope_id() != result_scope {
            self.exit_scope(self.current_scope_id());
        }
    }

    pub(crate) fn define_variable(
        &mut self,
        definition_id: VariableDefinitionId,
        value: ExpressionValue,
    ) {
        let definition = self.scope_definitions.definitions.get(definition_id);
        let scope_data = self.scope_mut(definition.scope);
        scope_data.define_variable(definition_id, value)
    }

    pub(crate) fn resolve(
        &mut self,
        variable: &VariableReference,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<LateBoundValue> {
        let reference = self.scope_definitions.references.get(variable.id);
        let (definition, span, is_final) = (
            reference.definition,
            reference.reference_name_span,
            reference.is_final_reference,
        );
        let scope_data = self.scope_mut(reference.definition_scope);
        scope_data.resolve(definition, span, is_final, ownership)
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

struct RuntimeScope {
    id: ScopeId,
    variables: HashMap<VariableDefinitionId, VariableContent>,
}

impl RuntimeScope {
    fn define_variable(&mut self, definition_id: VariableDefinitionId, value: ExpressionValue) {
        self.variables
            .get_mut(&definition_id)
            .expect("Variable data not found in scope")
            .define(value);
    }

    fn resolve(
        &mut self,
        definition_id: VariableDefinitionId,
        span: Span,
        is_final: bool,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<LateBoundValue> {
        self.variables
            .get_mut(&definition_id)
            .expect("Variable data not found in scope")
            .resolve(span, is_final, ownership)
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
