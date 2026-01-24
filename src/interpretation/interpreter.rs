use super::*;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    scope_definitions: ScopeDefinitions,
    scopes: Vec<RuntimeScope>,
    no_mutation_above: Vec<(ScopeId, MutationBlockReason)>,
    output_handler: OutputHandler,
    input_handler: InputHandler,
}

impl Interpreter {
    pub(crate) fn new(scope_definitions: ScopeDefinitions) -> Self {
        let root_scope_id = scope_definitions.root_scope;
        let mut interpreter = Self {
            config: Default::default(),
            scope_definitions,
            scopes: vec![],
            no_mutation_above: vec![],
            output_handler: OutputHandler::new(OutputStream::new()),
            input_handler: InputHandler::new(),
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

    pub(crate) fn enter_scope_starting_with_revertible_segment<T>(
        &mut self,
        scope_id: ScopeId,
        catch_location_id: CatchLocationId,
        revertible_segment: impl FnOnce(&mut Self) -> ExecutionResult<T>,
        guard_clause: Option<impl FnOnce(&mut Self) -> ExecutionResult<bool>>,
        reason: MutationBlockReason,
    ) -> ExecutionResult<AttemptOutcome<T>> {
        self.enter_scope_inner(scope_id, true);
        self.no_mutation_above.push((scope_id, reason));
        unsafe {
            // SAFETY: This is paired with `unfreeze_existing` below,
            // without any early returns in the middle
            self.output_handler.freeze_existing(reason);
            // SAFETY: This is paired with `commit_fork` or `rollback_fork` below
            self.input_handler.start_fork();
        }
        let revertible_result = revertible_segment(self);
        let result = self.convert_revertible_result(
            revertible_result,
            guard_clause,
            catch_location_id,
            scope_id,
        );
        unsafe {
            // SAFETY: This is paired with `start_fork` above
            match &result {
                Ok(AttemptOutcome::Completed(_)) => self.input_handler.commit_fork(),
                Ok(AttemptOutcome::Reverted) | Err(_) => self.input_handler.rollback_fork(),
            }
            // SAFETY: This is paired with `freeze_existing` above
            self.output_handler.unfreeze_existing();
        }
        self.no_mutation_above.pop();
        result
    }

    // Creating a separate function makes it easier to verify safety invariants
    // around early returns
    fn convert_revertible_result<T>(
        &mut self,
        revertible_result: ExecutionResult<T>,
        guard_clause: Option<impl FnOnce(&mut Self) -> ExecutionResult<bool>>,
        catch_location_id: CatchLocationId,
        scope_id: ScopeId,
    ) -> ExecutionResult<AttemptOutcome<T>> {
        match revertible_result {
            Ok(value) => {
                let guard_result = if let Some(guard_clause) = guard_clause {
                    guard_clause(self)
                } else {
                    Ok(true)
                };
                // If a guard clause errors, we treat this as a standard error
                // outside of the attempt arm catch. BUT we should still revert
                // any mutations made in the arm.
                match guard_result {
                    Ok(true) => Ok(AttemptOutcome::Completed(value)),
                    Ok(false) => Ok(AttemptOutcome::Reverted),
                    Err(err) => Err(err),
                }
            }
            Err(err) if err.is_catchable_by_attempt_block(catch_location_id) => {
                self.handle_catch(scope_id);
                Ok(AttemptOutcome::Reverted)
            }
            Err(mut err) => {
                if let Some((kind, error)) = err.error_mut() {
                    *error = core::mem::take(error).add_context_if_none(format!("NOTE: {} is not caught by an attempt block. If you wish to catch this, detect it before it is thrown and use the `revert` statement.", kind.as_str().indefinite_articled(true)));
                }
                Err(err)
            }
        }
    }

    fn enter_scope_inner(&mut self, id: ScopeId, check_parent: bool) {
        let new_scope = self.scope_definitions.scopes.get(id);
        if check_parent {
            assert!(new_scope.parent == Some(self.current_scope_id()));
        }
        let variables = {
            let mut map = HashMap::new();
            for definition_id in new_scope.definitions.iter() {
                map.insert(*definition_id, VariableState::Uninitialized);
            }
            map
        };
        self.scopes.push(RuntimeScope { id, variables });
    }

    pub(crate) fn exit_scope(&mut self, scope_id: ScopeId) {
        assert!(
            scope_id == self.current_scope_id(),
            "Attempted to exit scope {:?} but current scope is {:?}",
            scope_id,
            self.current_scope_id()
        );
        self.scopes.pop();
    }

    pub(crate) fn catch_control_flow<T>(
        &mut self,
        input: ExecutionResult<Spanned<T>>,
        catch_location_id: CatchLocationId,
        return_to_scope: ScopeId,
    ) -> ExecutionResult<ExecutionOutcome<T>> {
        let output = match input {
            Ok(value) => Ok(ExecutionOutcome::Value(value)),
            Err(interrupt) => interrupt.into_outcome::<T>(catch_location_id),
        };
        if let Ok(ExecutionOutcome::ControlFlow(_)) = &output {
            self.handle_catch(return_to_scope)
        }
        output
    }

    fn handle_catch(&mut self, result_scope: ScopeId) {
        // Note: OutputHandler safety upon error control flow is handled in that code.
        while self.current_scope_id() != result_scope {
            self.exit_scope(self.current_scope_id());
        }
    }

    pub(crate) fn define_variable(&mut self, definition_id: VariableDefinitionId, value: AnyValue) {
        let definition = self.scope_definitions.definitions.get(definition_id);
        let scope_data = self.scope_mut(definition.scope);
        scope_data.define_variable(definition_id, value)
    }

    pub(crate) fn resolve(
        &mut self,
        variable: &VariableReference,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<Spanned<LateBoundValue>> {
        let reference = self.scope_definitions.references.get(variable.id);
        let (definition, span, is_final) = (
            reference.definition,
            reference.reference_name_span,
            reference.is_final_reference,
        );
        let blocked_from_mutation = match self.no_mutation_above.last() {
            Some(&(no_mutation_above_scope, reason)) => 'result: {
                for scope in self.scopes.iter().rev() {
                    match scope.id {
                        id if id == reference.definition_scope => break 'result None,
                        id if id == no_mutation_above_scope => break 'result Some(reason),
                        _ => {}
                    }
                }
                panic!("Definition scope expected in scope stack due to control flow analysis");
            }
            None => None,
        };
        let scope_data = self.scope_mut(reference.definition_scope);
        scope_data.resolve(definition, span, is_final, ownership, blocked_from_mutation)
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

    // Input
    pub(crate) fn start_parse<T>(
        &mut self,
        stream: OutputStream,
        f: impl FnOnce(&mut Interpreter, ParserHandle) -> ExecutionResult<T>,
    ) -> ExecutionResult<T> {
        stream.parse_with(|input| {
            let handle = unsafe {
                // SAFETY: This is paired with `finish_parse` below,
                // without any early returns in the middle
                self.input_handler.start_parse(input)
            };
            let result = self.parse_with(handle, |interpreter| f(interpreter, handle));
            let finish_result = unsafe {
                // SAFETY: This is paired with `start_parse` above
                self.input_handler.finish_parse(handle)
            };
            // Combine results: if original failed, return that error; otherwise check finish_result
            match (result, finish_result) {
                (Ok(value), Ok(())) => Ok(value),
                (Err(err), _) => Err(err),
                (Ok(_), Err(err)) => Err(err.into()),
            }
        })
    }

    pub(crate) fn parse_with<T>(
        &mut self,
        handle: ParserHandle,
        f: impl FnOnce(&mut Interpreter) -> ExecutionResult<T>,
    ) -> ExecutionResult<T> {
        unsafe {
            // SAFETY: This is paired with `pop_current_handle` below,
            // without any early returns in the middle
            self.input_handler.push_current_handle(handle);
        }
        let result = f(self);
        unsafe {
            // SAFETY: This is paired with `push_current_handle` above,
            // without any early returns in the middle
            self.input_handler.pop_current_handle(handle);
        }
        result
    }

    pub(crate) fn parser(
        &mut self,
        handle: ParserHandle,
        error_span_range: SpanRange,
    ) -> ExecutionResult<OutputParseStream<'_>> {
        let stack = self
            .input_handler
            .get(handle)
            .ok_or_else(|| error_span_range.value_error("This parser is no longer available"))?;
        Ok(stack.current())
    }

    pub(crate) fn parse_group<T>(
        &mut self,
        required_delimiter: Option<Delimiter>,
        f: impl FnOnce(&mut Interpreter, Delimiter, DelimSpan) -> ExecutionResult<T>,
    ) -> ExecutionResult<T> {
        let (delimiter, delim_span) = self
            .input_handler
            .current_stack()
            .parse_and_enter_group(required_delimiter)?;
        let result = f(self, delimiter, delim_span);
        self.input_handler
            .current_stack()
            .exit_group(Some(delimiter))
            .expect("exit_group can't fail since we pass the same delimiter we got from parse_and_enter_group");
        result
    }

    /// Enter a group with the specified delimiter.
    /// Must be paired with `exit_input_group`.
    pub(crate) fn enter_input_group(
        &mut self,
        required_delimiter: Option<Delimiter>,
    ) -> ExecutionResult<(Delimiter, DelimSpan)> {
        self.input_handler
            .current_stack()
            .parse_and_enter_group(required_delimiter)
            .map_err(|e| e.into())
    }

    /// Exit the current input group.
    /// Must be paired with a prior `enter_input_group`.
    pub(crate) fn exit_input_group(
        &mut self,
        expected_delimiter: Option<Delimiter>,
    ) -> ExecutionResult<()> {
        self.input_handler
            .current_stack()
            .exit_group(expected_delimiter)
            .map_err(|e| e.into())
    }

    /// Returns true if there is an active input group that can be exited.
    pub(crate) fn has_active_input_group(&mut self) -> bool {
        self.input_handler.current_stack().has_active_group()
    }

    pub(crate) fn input<'a>(&'a mut self) -> ParseStream<'a, Output> {
        self.input_handler.current_input()
    }

    // Output
    pub(crate) fn in_output_group<F, R>(
        &mut self,
        delimiter: Delimiter,
        span: Span,
        f: F,
    ) -> ExecutionResult<R>
    where
        F: FnOnce(&mut Interpreter) -> ExecutionResult<R>,
    {
        unsafe {
            // SAFETY: This is paired with `finish_inner_buffer_as_group`
            self.output_handler.start_inner_buffer();
        }
        let result = f(self);
        unsafe {
            // SAFETY: This is paired with `start_inner_buffer`,
            // even if `f` returns an Err propogating a control flow interrupt.
            self.output_handler
                .finish_inner_buffer_as_group(delimiter, span);
        }
        result
    }

    pub(crate) fn capture_output<F>(&mut self, f: F) -> ExecutionResult<OutputStream>
    where
        F: FnOnce(&mut Interpreter) -> ExecutionResult<()>,
    {
        unsafe {
            // SAFETY: This is paired with `finish_inner_buffer_as_separate_stream`
            self.output_handler.start_inner_buffer();
        }
        let result = f(self);
        let output = unsafe {
            // SAFETY: This is paired with `start_inner_buffer`,
            // even if `f` returns an Err propogating a control flow interrupt.
            self.output_handler.finish_inner_buffer_as_separate_stream()
        };
        let () = result?;
        Ok(output)
    }

    pub(crate) fn output(
        &mut self,
        span_source: &impl HasSpanRange,
    ) -> ExecutionResult<&mut OutputStream> {
        self.output_handler.current_output_mut(span_source)
    }

    pub(crate) fn complete(self) -> OutputStream {
        self.output_handler.complete()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum MutationBlockReason {
    AttemptRevertibleSegment,
}

impl MutationBlockReason {
    pub(crate) fn error_message(&self, mutation_kind: &str) -> String {
        match self {
            MutationBlockReason::AttemptRevertibleSegment => {
                format!("It is not possible to {mutation_kind} here. An attempt arm is in two parts: {{ /* revertible */ }} => {{ /* unconditional */ }}. You may define variables in the revertible part, but all mutations of state outside the attempt block must be moved to the unconditional part")
            }
        }
    }
}

#[must_use]
pub(crate) enum AttemptOutcome<T> {
    Completed(T),
    Reverted,
}

struct RuntimeScope {
    id: ScopeId,
    variables: HashMap<VariableDefinitionId, VariableState>,
}

impl RuntimeScope {
    fn define_variable(&mut self, definition_id: VariableDefinitionId, value: AnyValue) {
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
        ownership: RequestedOwnership,
        blocked_from_mutation: Option<MutationBlockReason>,
    ) -> ExecutionResult<Spanned<LateBoundValue>> {
        self.variables
            .get_mut(&definition_id)
            .expect("Variable data not found in scope")
            .resolve(span, is_final, ownership, blocked_from_mutation)
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
                return self.span_source.control_flow_err(format!("Iteration limit of {} exceeded.\nIf needed, the limit can be reconfigured with preinterpret::set_iteration_limit(XXX)", limit));
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
