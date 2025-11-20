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
    pub(super) label: Option<syn::Lifetime>,
    pub(super) braces: Braces,
    pub(super) scope: ScopeId,
    pub(super) content: ExpressionBlockContent,
}

impl ParseSource for ExpressionBlock {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        // Check if there's a label before the block
        let label = if input.cursor().lifetime().is_some() {
            let lifetime: syn::Lifetime = input.parse()?;
            input.parse::<Token![:]>()?;
            Some(lifetime)
        } else {
            None
        };

        let (braces, inner) = input.parse_braces()?;
        let content = inner.parse()?;
        Ok(Self {
            label,
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

impl HasSpan for ExpressionBlock {
    fn span(&self) -> Span {
        if let Some(label) = &self.label {
            label.apostrophe.join(self.braces.close()).unwrap_or(self.braces.join())
        } else {
            self.braces.join()
        }
    }
}

impl ExpressionBlock {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        interpreter.enter_scope(self.scope);
        let output_result = self
            .content
            .evaluate(interpreter, self.span().into(), ownership);

        // If this block has a label, check if a break with matching label occurred
        if let Some(block_label) = &self.label {
            let scope = self.scope;
            match output_result.catch_control_flow(
                interpreter,
                |ctrl| {
                    // Catch breaks with our label
                    matches!(ctrl,
                        ControlFlowInterrupt::Break { label: Some(l), .. }
                        if l.ident.to_string() == block_label.ident.to_string()
                    )
                },
                scope,
            )? {
                ExecutionOutcome::Value(value) => {
                    interpreter.exit_scope(self.scope);
                    Ok(value)
                }
                ExecutionOutcome::ControlFlow(ControlFlowInterrupt::Break { value, .. }) => {
                    // This break is for us! Return the value
                    interpreter.exit_scope(self.scope);
                    let span_range: SpanRange = self.span().into();
                    ownership.map_from_owned(
                        value.unwrap_or_else(|| ().into_owned_value(span_range))
                    )
                }
                ExecutionOutcome::ControlFlow(other) => {
                    // Shouldn't happen, but propagate it
                    interpreter.exit_scope(self.scope);
                    Err(ExecutionInterrupt::control_flow(other, self.span()))
                }
            }
        } else {
            // No label, just return the result normally
            let output = output_result?;
            interpreter.exit_scope(self.scope);
            Ok(output)
        }
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
