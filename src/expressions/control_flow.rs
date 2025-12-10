use super::*;

pub(crate) struct IfExpression {
    if_token: Ident,
    condition: Expression,
    then_code: ScopedBlock,
    else_ifs: Vec<(Expression, ScopedBlock)>,
    else_code: Option<ScopedBlock>,
}

impl HasSpanRange for IfExpression {
    fn span_range(&self) -> SpanRange {
        let start = self.if_token.span();
        let end = if let Some(else_code) = &self.else_code {
            else_code.span()
        } else if let Some((_, last_else_if_code)) = self.else_ifs.last() {
            last_else_if_code.span()
        } else {
            self.then_code.span()
        };
        SpanRange::new_between(start, end)
    }
}

impl ParseSource for IfExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let if_token = input.parse_ident_matching("if")?;

        let condition = input.parse()?;
        let then_code = input.parse()?;

        let mut else_ifs = Vec::new();
        let mut else_code = None;
        while input.peek_ident_matching("else") {
            let _ = input.parse_ident_matching("else")?;
            if input.peek_ident_matching("if") {
                input.parse_ident_matching("if")?;

                let condition = input.parse()?;
                let then_code = input.parse()?;

                else_ifs.push((condition, then_code));
            } else {
                else_code = Some(input.parse()?);
                break;
            }
        }

        Ok(Self {
            if_token,
            condition,
            then_code,
            else_ifs,
            else_code,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        // In terms of control-flow segments, the possible execution paths
        // look like this, so we model that with path-based segments:
        //
        // IfCondA           < BlockA
        // ^< ElseIfCondB    < BlockB
        //    ^< ElseIfCondC < BlockC
        //       ^<----------- ElseBlock
        let outer_segment = context.enter_next_segment(SegmentKind::PathBased);

        let mut cond_segment = context.enter_path_segment(None, SegmentKind::Sequential);
        self.condition.control_flow_pass(context)?;
        context.exit_segment(cond_segment);

        let block_segment = context.enter_path_segment(Some(cond_segment), SegmentKind::Sequential);
        self.then_code.control_flow_pass(context)?;
        context.exit_segment(block_segment);

        for (condition, code) in &mut self.else_ifs {
            cond_segment = context.enter_path_segment(Some(cond_segment), SegmentKind::Sequential);
            condition.control_flow_pass(context)?;
            context.exit_segment(cond_segment);

            let block_segment =
                context.enter_path_segment(Some(cond_segment), SegmentKind::Sequential);
            code.control_flow_pass(context)?;
            context.exit_segment(block_segment);
        }

        if let Some(else_code) = &mut self.else_code {
            let block_segment =
                context.enter_path_segment(Some(cond_segment), SegmentKind::Sequential);
            else_code.control_flow_pass(context)?;
            context.exit_segment(block_segment);
        }

        context.exit_segment(outer_segment);

        Ok(())
    }
}

impl Evaluate for IfExpression {
    fn evaluate_unspanned(
        &self,
        interpreter: &mut Interpreter,
        requested_ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let evaluated_condition: bool = self
            .condition
            .evaluate_owned(interpreter)?
            .resolve_as("An if condition")?;

        if evaluated_condition {
            return self
                .then_code
                .evaluate_unspanned(interpreter, requested_ownership);
        }

        for (condition, code) in &self.else_ifs {
            let evaluated_condition: bool = condition
                .evaluate_owned(interpreter)?
                .resolve_as("An else if condition")?;
            if evaluated_condition {
                return code.evaluate_unspanned(interpreter, requested_ownership);
            }
        }

        if let Some(else_code) = &self.else_code {
            return else_code.evaluate_unspanned(interpreter, requested_ownership);
        }

        requested_ownership.map_from_owned(Value::None.into_owned(), self.span_range())
    }
}

pub(crate) struct WhileExpression {
    label: Option<CatchLabel>,
    while_token: Ident,
    condition: Expression,
    body: ScopedBlock,
    catch_location: CatchLocationId,
}

impl HasSpanRange for WhileExpression {
    fn span_range(&self) -> SpanRange {
        // We ignore the label, as it's not really part of the expression
        SpanRange::new_between(self.while_token.span(), self.body.span())
    }
}

impl ParseSource for WhileExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let label = input.parse_optional()?;
        let while_token = input.parse_ident_matching("while")?;
        let condition = input.parse()?;
        let body = input.parse()?;

        Ok(Self {
            label,
            while_token,
            condition,
            body,
            catch_location: CatchLocationId::new_placeholder(),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.catch_location = context.register_catch_location(CatchLocationData::Loop {
            label: self.label.as_ref().map(|l| l.ident_string()),
        });

        let segment = context.enter_next_segment(SegmentKind::LoopingSequential);
        self.condition.control_flow_pass(context)?;

        context.enter_catch(self.catch_location);
        self.body.control_flow_pass(context)?;
        context.exit_catch(self.catch_location);

        context.exit_segment(segment);
        Ok(())
    }
}

impl Evaluate for WhileExpression {
    fn evaluate_unspanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let span = self.body.span();
        let mut iteration_counter = interpreter.start_iteration_counter(&span);

        let scope = interpreter.current_scope_id();
        while self
            .condition
            .evaluate_owned(interpreter)?
            .resolve_as("A while condition")?
        {
            iteration_counter.increment_and_check()?;
            let body_result = self.body.evaluate_owned(interpreter);
            match interpreter.catch_control_flow(body_result, self.catch_location, scope)? {
                ExecutionOutcome::Value(value) => {
                    value.into_statement_result(self.body.span().span_range())?;
                }
                ExecutionOutcome::ControlFlow(control_flow_interrupt) => {
                    match control_flow_interrupt {
                        ControlFlowInterrupt::Break(break_interrupt) => {
                            return break_interrupt.into_value(self.span_range(), ownership);
                        }
                        ControlFlowInterrupt::Continue { .. } => {
                            continue;
                        }
                        ControlFlowInterrupt::Revert(_) => {
                            unreachable!("A revert should not match to a loop catch location")
                        }
                    }
                }
            }
        }
        ownership.map_none(self.span_range())
    }
}

pub(crate) struct LoopExpression {
    catch_location: CatchLocationId,
    label: Option<CatchLabel>,
    loop_token: Ident,
    body: ScopedBlock,
}

impl HasSpanRange for LoopExpression {
    fn span_range(&self) -> SpanRange {
        // We ignore the label, because it's not really part of the span of the resultant value
        SpanRange::new_between(self.loop_token.span(), self.body.span())
    }
}

impl ParseSource for LoopExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let label = input.parse_optional()?;
        let loop_token = input.parse_ident_matching("loop")?;
        let body = input.parse()?;
        Ok(Self {
            catch_location: CatchLocationId::new_placeholder(),
            label,
            loop_token,
            body,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.catch_location = context.register_catch_location(CatchLocationData::Loop {
            label: self.label.as_ref().map(|l| l.ident_string()),
        });

        let segment = context.enter_next_segment(SegmentKind::LoopingSequential);

        context.enter_catch(self.catch_location);
        self.body.control_flow_pass(context)?;
        context.exit_catch(self.catch_location);

        context.exit_segment(segment);
        Ok(())
    }
}

impl Evaluate for LoopExpression {
    fn evaluate_unspanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let span = self.body.span();
        let mut iteration_counter = interpreter.start_iteration_counter(&span);

        let scope = interpreter.current_scope_id();
        loop {
            iteration_counter.increment_and_check()?;

            let body_result = self.body.evaluate_owned(interpreter);
            match interpreter.catch_control_flow(body_result, self.catch_location, scope)? {
                ExecutionOutcome::Value(value) => {
                    value.into_statement_result(self.body.span().span_range())?;
                }
                ExecutionOutcome::ControlFlow(control_flow_interrupt) => {
                    match control_flow_interrupt {
                        ControlFlowInterrupt::Break(break_interrupt) => {
                            return break_interrupt.into_value(self.span_range(), ownership);
                        }
                        ControlFlowInterrupt::Continue { .. } => {
                            continue;
                        }
                        ControlFlowInterrupt::Revert(_) => {
                            unreachable!("A revert should not match to a loop catch location")
                        }
                    }
                }
            }
        }
    }
}

pub(crate) struct ForExpression {
    iteration_scope: ScopeId,
    catch_location: CatchLocationId,
    label: Option<CatchLabel>,
    for_token: Ident,
    pattern: Pattern,
    _in_token: Ident,
    iterable: Expression,
    body: ScopedBlock,
}

impl HasSpanRange for ForExpression {
    fn span_range(&self) -> SpanRange {
        // We ignore the label, because it's not really part of the span of the resultant value
        SpanRange::new_between(self.for_token.span(), self.body.span())
    }
}

impl ParseSource for ForExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let label = input.parse_optional()?;
        let for_token = input.parse_ident_matching("for")?;
        let pattern = input.parse()?;
        let in_token = input.parse_ident_matching("in")?;
        let iterable = input.parse()?;
        let body = input.parse()?;

        Ok(Self {
            iteration_scope: ScopeId::new_placeholder(),
            catch_location: CatchLocationId::new_placeholder(),
            label,
            for_token,
            pattern,
            _in_token: in_token,
            iterable,
            body,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.catch_location = context.register_catch_location(CatchLocationData::Loop {
            label: self.label.as_ref().map(|l| l.ident_string()),
        });

        context.register_scope(&mut self.iteration_scope);
        self.iterable.control_flow_pass(context)?;

        let segment = context.enter_next_segment(SegmentKind::LoopingSequential);
        context.enter_scope(self.iteration_scope);

        context.enter_catch(self.catch_location);
        self.pattern.control_flow_pass(context)?;
        self.body.control_flow_pass(context)?;
        context.exit_catch(self.catch_location);

        context.exit_scope(self.iteration_scope);
        context.exit_segment(segment);

        Ok(())
    }
}

impl Evaluate for ForExpression {
    fn evaluate_unspanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let iterable: IterableValue = self
            .iterable
            .evaluate_owned(interpreter)?
            .resolve_as("A for loop iterable")?;

        let span = self.body.span();
        let scope = interpreter.current_scope_id();
        let mut iteration_counter = interpreter.start_iteration_counter(&span);

        for item in iterable.into_iterator()? {
            iteration_counter.increment_and_check()?;

            interpreter.enter_scope(self.iteration_scope);
            self.pattern.handle_destructure(interpreter, item)?;

            let body_result = self.body.evaluate_owned(interpreter);
            match interpreter.catch_control_flow(body_result, self.catch_location, scope)? {
                ExecutionOutcome::Value(value) => {
                    value.into_statement_result(self.body.span().span_range())?;
                }
                ExecutionOutcome::ControlFlow(control_flow_interrupt) => {
                    match control_flow_interrupt {
                        ControlFlowInterrupt::Break(break_interrupt) => {
                            return break_interrupt.into_value(self.span_range(), ownership);
                        }
                        ControlFlowInterrupt::Continue { .. } => {
                            continue;
                        }
                        ControlFlowInterrupt::Revert(_) => {
                            unreachable!("A revert should not match to a loop catch location")
                        }
                    }
                }
            }
            interpreter.exit_scope(self.iteration_scope);
        }
        ownership.map_none(self.span_range())
    }
}

pub(crate) struct AttemptExpression {
    catch_location: CatchLocationId,
    label: Option<CatchLabel>,
    attempt: AttemptKeyword,
    braces: Braces,
    arms: Vec<AttemptArm>,
}

struct AttemptArm {
    arm_scope: ScopeId,
    // The LHS's scope extends into the RHS
    lhs: UnscopedBlock,
    guard: Option<(Token![if], Expression)>,
    _arrow: Token![=>],
    rhs: Expression,
}

impl HasSpanRange for AttemptExpression {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.attempt.span(), self.braces.close())
    }
}

impl ParseSource for AttemptExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let label = input.parse_optional()?;
        let attempt = input.parse()?;
        let (braces, inner) = input.parse_braces()?;
        let mut arms = vec![];
        while !inner.is_empty() {
            let lhs = inner.parse()?;
            let guard = if inner.peek_ident_matching("if") {
                let if_token = inner.parse()?;
                let condition = inner.parse()?;
                Some((if_token, condition))
            } else {
                None
            };
            let arrow = inner.parse()?;
            let rhs: Expression = inner.parse()?;
            if inner.peek(Token![,]) {
                let _ = inner.parse::<Token![,]>()?;
            } else if !rhs.is_block() {
                inner.parse_err("Expected trailing comma after previous non-block attempt arm")?;
            }
            arms.push(AttemptArm {
                arm_scope: ScopeId::new_placeholder(),
                lhs,
                guard,
                _arrow: arrow,
                rhs,
            });
        }
        Ok(Self {
            catch_location: CatchLocationId::new_placeholder(),
            label,
            attempt,
            braces,
            arms,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.catch_location = context.register_catch_location(CatchLocationData::AttemptBlock {
            label: self.label.as_ref().map(|l| l.ident_string()),
        });

        let segment = context.enter_next_segment(SegmentKind::PathBased);
        let mut previous_attempt_segment = None;
        for arm in &mut self.arms {
            context.register_scope(&mut arm.arm_scope);

            context.enter_scope(arm.arm_scope);
            let attempt_segment = context
                .enter_path_segment(previous_attempt_segment, SegmentKind::RevertibleSequential);

            context.enter_catch(self.catch_location);
            arm.lhs.control_flow_pass(context)?;
            context.exit_catch(self.catch_location);

            if let Some((_, guard_expression)) = &mut arm.guard {
                guard_expression.control_flow_pass(context)?;
            }

            context.exit_segment(attempt_segment);
            previous_attempt_segment = Some(attempt_segment);

            let action_segment =
                context.enter_path_segment(previous_attempt_segment, SegmentKind::Sequential);
            arm.rhs.control_flow_pass(context)?;
            context.exit_segment(action_segment);
            context.exit_scope(arm.arm_scope);
        }
        context.exit_segment(segment);

        Ok(())
    }
}

impl Evaluate for AttemptExpression {
    fn evaluate_unspanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        // We need a separate method to correctly capture the lifetimes of the guard clause
        fn guard_clause<'a>(
            guard: Option<&'a (Token![if], Expression)>,
        ) -> Option<impl for<'b> FnOnce(&'b mut Interpreter) -> ExecutionResult<bool> + 'a>
        {
            guard.map(|(_, guard_expression)| {
                move |interpreter: &mut Interpreter| -> ExecutionResult<bool> {
                    guard_expression
                        .evaluate_owned(interpreter)?
                        .resolve_as("The guard condition of an attempt arm")
                }
            })
        }
        for arm in self.arms.iter() {
            let attempt_outcome = interpreter.enter_scope_starting_with_revertible_segment(
                arm.arm_scope,
                self.catch_location,
                |interpreter| -> ExecutionResult<()> {
                    arm.lhs
                        .evaluate_owned(interpreter)?
                        .resolve_as("The returned value from the left half of an attempt arm")
                },
                guard_clause(arm.guard.as_ref()),
                MutationBlockReason::AttemptRevertibleSegment,
            )?;
            match attempt_outcome {
                AttemptOutcome::Completed(()) => { /* proceed to rhs */ }
                AttemptOutcome::Reverted => {
                    interpreter.exit_scope(arm.arm_scope);
                    continue;
                }
            }
            let output = arm.rhs.evaluate(interpreter, ownership)?;
            interpreter.exit_scope(arm.arm_scope);
            return Ok(output);
        }
        self.braces.control_flow_err("No attempt arm ran successfully. You may wish to add a fallback arm `{} => { None }` to ignore the error or to propagate a better message: `{} => { %[<tokens for error span>].error(\"Error message\") }`.")
    }
}

pub(crate) struct ParseExpression {
    parse_ident: ParseKeyword,
    input: Expression,
    _fat_arrow: Unused<Token![=>]>,
    _left_bar: Unused<Token![|]>,
    parser_variable: VariableDefinition,
    _right_bar: Unused<Token![|]>,
    scope: ScopeId,
    body: UnscopedBlock,
}

impl HasSpanRange for ParseExpression {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.parse_ident.span(), self.body.span())
    }
}

impl ParseSource for ParseExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let parse_ident = input.parse()?;
        let input_expression = input.parse()?;
        let _fat_arrow = input.parse()?;
        let _left_bar = input.parse()?;
        let parser_variable = input.parse()?;
        let _right_bar = input.parse()?;
        let body = input.parse()?;
        Ok(Self {
            parse_ident,
            input: input_expression,
            _fat_arrow,
            _left_bar,
            parser_variable,
            _right_bar,
            scope: ScopeId::new_placeholder(),
            body,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        context.register_scope(&mut self.scope);
        self.input.control_flow_pass(context)?;
        context.enter_scope(self.scope);
        self.parser_variable.control_flow_pass(context)?;
        self.body.control_flow_pass(context)?;
        context.exit_scope(self.scope);
        Ok(())
    }
}

impl Evaluate for ParseExpression {
    fn evaluate_unspanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let input = self
            .input
            .evaluate_owned(interpreter)?
            .resolve_as("The input to a parse expression")?;

        interpreter.enter_scope(self.scope);

        let output = interpreter.start_parse(input, |interpreter, handle| {
            self.parser_variable.define(interpreter, handle);
            self.body.evaluate_unspanned(interpreter, ownership)
        })?;

        interpreter.exit_scope(self.scope);
        Ok(output)
    }
}
