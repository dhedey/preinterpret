use super::*;

pub(crate) struct EmbeddedExpression {
    marker: Token![#],
    parentheses: Parentheses,
    content: Expression,
}

impl ParseSource for EmbeddedExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let marker = input.parse()?;
        let (parentheses, inner) = input.parse_parentheses()?;
        let content = inner.parse()?;
        Ok(Self {
            marker,
            parentheses,
            content,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl HasSpanRange for EmbeddedExpression {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.parentheses.close())
    }
}

impl Interpret for EmbeddedExpression {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let value = self.content.evaluate_shared(interpreter)?;
        value.output_to(
            Grouping::Flattened,
            &mut ToStreamContext::new(interpreter.output(self)?, self.span_range()),
        )
    }
}

pub(crate) struct EmbeddedStatements {
    marker: Token![#],
    braces: Braces,
    content: ExpressionBlockContent,
}

impl ParseSource for EmbeddedStatements {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let marker = input.parse()?;
        let (braces, inner) = input.parse_braces()?;
        let content = inner.parse()?;
        Ok(Self {
            marker,
            braces,
            content,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl HasSpanRange for EmbeddedStatements {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.braces.close())
    }
}

impl Interpret for EmbeddedStatements {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let value = self
            .content
            .evaluate(
                interpreter,
                self.span_range(),
                RequestedValueOwnership::shared(),
            )?
            .expect_shared();
        value.output_to(
            Grouping::Flattened,
            &mut ToStreamContext::new(interpreter.output(self)?, self.span_range()),
        )
    }
}

pub(crate) struct ExpressionBlock {
    pub(super) label: Option<CatchLabel>,
    pub(super) scoped_block: ScopedBlock,
}

impl ParseSource for ExpressionBlock {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let label = input.parse_optional()?;
        let scoped_block = input.parse()?;
        Ok(Self {
            label,
            scoped_block,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        // If this block has a label, register a catch location for it
        if self.label.is_some() {
            context.register_catch_location_with_optional_label(
                self.label.as_mut(),
                CatchLocationKind::LabeledBlock,
            );
        }
        self.scoped_block.control_flow_pass(context)
    }
}

impl HasSpanRange for ExpressionBlock {
    fn span_range(&self) -> SpanRange {
        if let Some(label) = &self.label {
            SpanRange::new_between(label.span_range(), self.scoped_block.span_range())
        } else {
            self.scoped_block.span_range()
        }
    }
}

impl ExpressionBlock {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        let scope = interpreter.current_scope_id();
        let output_result = self.scoped_block.evaluate(interpreter, ownership);

        // If this block has a label, catch breaks targeting this specific catch location
        let output = if let Some(label) = &self.label {
            match interpreter.catch_control_flow(
                output_result,
                |ctrl| ControlFlowInterrupt::catch_labelled_break(ctrl, label.catch_location_id),
                scope,
            )? {
                ExecutionOutcome::Value(value) => value,
                ExecutionOutcome::ControlFlow(ControlFlowInterrupt::Break(break_interrupt)) => {
                    break_interrupt.into_value(self.span_range(), ownership)?
                }
                ExecutionOutcome::ControlFlow(_) => {
                    unreachable!("Only break control flow should be catchable by labeled blocks")
                }
            }
        } else {
            // No label, just evaluate the block normally
            output_result?
        };
        Ok(output)
    }
}

pub(crate) struct ScopedBlock {
    pub(super) braces: Braces,
    pub(super) scope: ScopeId,
    pub(super) content: ExpressionBlockContent,
}

impl ParseSource for ScopedBlock {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (braces, inner) = input.parse_braces()?;
        let content = inner.parse()?;
        Ok(Self {
            braces,
            scope: ScopeId::new_placeholder(),
            content,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        context.register_scope(&mut self.scope);
        context.enter_scope(self.scope);
        self.content.control_flow_pass(context)?;
        context.exit_scope(self.scope);
        Ok(())
    }
}

impl HasSpan for ScopedBlock {
    fn span(&self) -> Span {
        self.braces.join()
    }
}

impl ScopedBlock {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        interpreter.enter_scope(self.scope);
        let output = self
            .content
            .evaluate(interpreter, self.span().into(), ownership)?;
        interpreter.exit_scope(self.scope);
        Ok(output)
    }

    pub(crate) fn evaluate_owned(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        self.evaluate(interpreter, RequestedValueOwnership::owned())
            .map(|x| x.expect_owned())
    }
}

pub(crate) struct UnscopedBlock {
    pub(super) braces: Braces,
    pub(super) content: ExpressionBlockContent,
}

impl ParseSource for UnscopedBlock {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (braces, inner) = input.parse_braces()?;
        let content = inner.parse()?;
        Ok(Self { braces, content })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl HasSpan for UnscopedBlock {
    fn span(&self) -> Span {
        self.braces.join()
    }
}

impl UnscopedBlock {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        self.content
            .evaluate(interpreter, self.span().into(), ownership)
    }

    pub(crate) fn evaluate_owned(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        self.evaluate(interpreter, RequestedValueOwnership::owned())
            .map(|x| x.expect_owned())
    }
}

pub(crate) struct ExpressionBlockContent {
    statements: Vec<(Statement, Option<Token![;]>)>,
}

impl ParseSource for ExpressionBlockContent {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let mut statements = Vec::new();
        while !input.is_empty() {
            let statement: Statement = input.parse()?;
            let requires_semicolon = statement.requires_semicolon(input.is_empty());
            match (requires_semicolon, input.peek(Token![;])) {
                (true, false) => {
                    if input.is_empty() {
                        return input.parse_err("Expected `;` at the end of this statement.");
                    } else {
                        return input.parse_err("Invalid statement continuation. Possibly the previous statement is missing a semicolon?");
                    }
                }
                (_, true) => {
                    let semicolon = input.parse()?;
                    statements.push((statement, Some(semicolon)));
                }
                (false, false) => {
                    statements.push((statement, None));
                }
            }
        }
        Ok(Self { statements })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        for (statement, _semicolon) in self.statements.iter_mut() {
            statement.control_flow_pass(context)?;
        }
        Ok(())
    }
}

impl ExpressionBlockContent {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        output_span_range: SpanRange,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        for (i, (statement, semicolon)) in self.statements.iter().enumerate() {
            let is_last = i == self.statements.len() - 1;
            if is_last && semicolon.is_none() {
                let value = statement.evaluate_as_returning_expression(interpreter, ownership)?;
                return Ok(value.with_span_range(output_span_range));
            } else {
                statement.evaluate_as_statement(interpreter)?;
            }
        }
        ownership.map_from_owned(ExpressionValue::None.into_owned(output_span_range))
    }
}
