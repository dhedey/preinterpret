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
    fn interpret_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.content.evaluate_shared(interpreter)?.output_to(
            Grouping::Flattened,
            &mut ToStreamContext::new(output, self.span_range()),
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
    fn interpret_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.content
            .evaluate(
                interpreter,
                self.span_range(),
                RequestedValueOwnership::shared(),
            )?
            .expect_shared()
            .output_to(
                Grouping::Flattened,
                &mut ToStreamContext::new(output, self.span_range()),
            )?;
        Ok(())
    }
}

pub(crate) struct ExpressionBlock {
    pub(super) braces: Braces,
    pub(super) scope: ScopeId,
    pub(super) content: ExpressionBlockContent,
}

impl ParseSource for ExpressionBlock {
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

impl HasSpan for ExpressionBlock {
    fn span(&self) -> Span {
        self.braces.join()
    }
}

impl ExpressionBlock {
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
