use super::*;

pub(crate) struct IfExpression {
    if_token: Ident,
    condition: Expression,
    then_code: ExpressionBlock,
    else_ifs: Vec<(Expression, ExpressionBlock)>,
    else_code: Option<ExpressionBlock>,
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

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
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

        for (condition, code) in &self.else_ifs {
            cond_segment = context.enter_path_segment(Some(cond_segment), SegmentKind::Sequential);
            condition.control_flow_pass(context)?;
            context.exit_segment(cond_segment);

            let block_segment =
                context.enter_path_segment(Some(cond_segment), SegmentKind::Sequential);
            code.control_flow_pass(context)?;
            context.exit_segment(block_segment);
        }

        if let Some(else_code) = &self.else_code {
            let block_segment =
                context.enter_path_segment(Some(cond_segment), SegmentKind::Sequential);
            else_code.control_flow_pass(context)?;
            context.exit_segment(block_segment);
        }

        context.exit_segment(outer_segment);

        Ok(())
    }
}

impl IfExpression {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        requested_ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        let evaluated_condition: bool = self
            .condition
            .evaluate_owned(interpreter)?
            .resolve_as("An if condition")?;

        if evaluated_condition {
            return self.then_code.evaluate(interpreter, requested_ownership);
        }

        for (condition, code) in &self.else_ifs {
            let evaluated_condition: bool = condition
                .evaluate_owned(interpreter)?
                .resolve_as("An else if condition")?;
            if evaluated_condition {
                return code.evaluate(interpreter, requested_ownership);
            }
        }

        if let Some(else_code) = &self.else_code {
            return else_code.evaluate(interpreter, requested_ownership);
        }

        requested_ownership.map_from_owned(ExpressionValue::None.into_owned(self.span_range()))
    }
}

pub(crate) struct WhileExpression {
    while_token: Ident,
    condition: Expression,
    body: ExpressionBlock,
}

impl HasSpanRange for WhileExpression {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.while_token.span(), self.body.span())
    }
}

impl ParseSource for WhileExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let while_token = input.parse_ident_matching("while")?;

        let condition = input.parse()?;
        let body = input.parse()?;

        Ok(Self {
            while_token,
            condition,
            body,
        })
    }

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        let segment = context.enter_next_segment(SegmentKind::LoopingSequential);
        self.condition.control_flow_pass(context)?;
        self.body.control_flow_pass(context)?;
        context.exit_segment(segment);
        Ok(())
    }
}

impl WhileExpression {
    pub(crate) fn evaluate_as_expression(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        self.evaluate(interpreter, false)
    }

    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        self.evaluate(interpreter, true).map(|_| ())
    }

    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        is_statement: bool,
    ) -> ExecutionResult<OwnedValue> {
        let span = self.body.span();
        let mut iteration_counter = interpreter.start_iteration_counter(&span);

        let scope = interpreter.current_scope_id();
        let mut output = vec![];
        while self
            .condition
            .evaluate_owned(interpreter)?
            .resolve_as("A while condition")?
        {
            iteration_counter.increment_and_check()?;
            match self.body.evaluate_owned(interpreter).catch_control_flow(
                interpreter,
                ControlFlowInterrupt::catch_any,
                scope,
            )? {
                ExecutionOutcome::Value(value) => {
                    if is_statement {
                        value.into_statement_result()?;
                    } else {
                        output.push(value.into_inner());
                    }
                }
                ExecutionOutcome::ControlFlow(control_flow_interrupt) => {
                    match control_flow_interrupt {
                        ControlFlowInterrupt::Break => break,
                        ControlFlowInterrupt::Continue => continue,
                    }
                }
            }
        }
        Ok(output.into_owned_value(self.span_range()))
    }
}

pub(crate) struct LoopExpression {
    loop_token: Ident,
    body: ExpressionBlock,
}

impl HasSpanRange for LoopExpression {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.loop_token.span(), self.body.span())
    }
}

impl ParseSource for LoopExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let loop_token = input.parse_ident_matching("loop")?;
        let body = input.parse()?;
        Ok(Self { loop_token, body })
    }

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        let segment = context.enter_next_segment(SegmentKind::LoopingSequential);
        self.body.control_flow_pass(context)?;
        context.exit_segment(segment);
        Ok(())
    }
}

impl LoopExpression {
    pub(crate) fn evaluate_as_expression(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        self.evaluate(interpreter, false)
    }

    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        self.evaluate(interpreter, true).map(|_| ())
    }

    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        is_statement: bool,
    ) -> ExecutionResult<OwnedValue> {
        let span = self.body.span();
        let mut iteration_counter = interpreter.start_iteration_counter(&span);

        let scope = interpreter.current_scope_id();
        let mut output = vec![];
        loop {
            iteration_counter.increment_and_check()?;

            match self.body.evaluate_owned(interpreter).catch_control_flow(
                interpreter,
                ControlFlowInterrupt::catch_any,
                scope,
            )? {
                ExecutionOutcome::Value(value) => {
                    if is_statement {
                        value.into_statement_result()?;
                    } else {
                        output.push(value.into_inner());
                    }
                }
                ExecutionOutcome::ControlFlow(control_flow_interrupt) => {
                    match control_flow_interrupt {
                        ControlFlowInterrupt::Break => break,
                        ControlFlowInterrupt::Continue => continue,
                    }
                }
            }
        }
        Ok(output.into_owned_value(self.span_range()))
    }
}

pub(crate) struct ForExpression {
    iteration_scope: ScopeId,
    for_token: Ident,
    pattern: Pattern,
    _in_token: Ident,
    iterable: Expression,
    body: ExpressionBlock,
}

impl HasSpanRange for ForExpression {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.for_token.span(), self.body.span())
    }
}

impl ParseSource for ForExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let for_token = input.parse_ident_matching("for")?;
        let iteration_scope = input.register_scope();
        let pattern = input.parse()?;
        let in_token = input.parse_ident_matching("in")?;
        let iterable = input.parse()?;
        let body = input.parse()?;

        Ok(Self {
            iteration_scope,
            for_token,
            pattern,
            _in_token: in_token,
            iterable,
            body,
        })
    }

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        self.iterable.control_flow_pass(context)?;

        let segment = context.enter_next_segment(SegmentKind::LoopingSequential);
        context.enter_scope(self.iteration_scope);

        self.pattern.control_flow_pass(context)?;
        self.body.control_flow_pass(context)?;

        context.exit_scope(self.iteration_scope);
        context.exit_segment(segment);

        Ok(())
    }
}

impl ForExpression {
    pub(crate) fn evaluate_as_expression(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        self.evaluate(interpreter, false)
    }

    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        self.evaluate(interpreter, true).map(|_| ())
    }

    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        is_statement: bool,
    ) -> ExecutionResult<OwnedValue> {
        let iterable: IterableValue = self
            .iterable
            .evaluate_owned(interpreter)?
            .resolve_as("A for loop iterable")?;

        let span = self.body.span();
        let scope = interpreter.current_scope_id();
        let mut iteration_counter = interpreter.start_iteration_counter(&span);

        let mut output = vec![];
        for item in iterable.into_iterator()? {
            iteration_counter.increment_and_check()?;

            interpreter.enter_scope(self.iteration_scope);
            self.pattern.handle_destructure(interpreter, item)?;

            match self.body.evaluate_owned(interpreter).catch_control_flow(
                interpreter,
                ControlFlowInterrupt::catch_any,
                scope,
            )? {
                ExecutionOutcome::Value(value) => {
                    if is_statement {
                        value.into_statement_result()?;
                    } else {
                        output.push(value.into_inner());
                    }
                }
                ExecutionOutcome::ControlFlow(control_flow_interrupt) => {
                    match control_flow_interrupt {
                        ControlFlowInterrupt::Break => break,
                        ControlFlowInterrupt::Continue => continue,
                    }
                }
            }
            interpreter.exit_scope(self.iteration_scope);
        }
        Ok(output.into_owned_value(self.span_range()))
    }
}
