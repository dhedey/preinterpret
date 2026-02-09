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

impl OutputToStream for EmbeddedExpression {
    fn output_to_stream(&self, output: &mut OutputInterpreter) -> ExecutionResult<()> {
        let value = output.with_interpreter(|i| self.content.evaluate_shared(i))?;
        value
            .as_ref_value()
            .output_to(
                Grouping::Flattened,
                &mut ToStreamContext::new(output, self.span_range()),
            )
            .into_execution_result()
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

impl OutputToStream for EmbeddedStatements {
    fn output_to_stream(&self, output: &mut OutputInterpreter) -> ExecutionResult<()> {
        let value = output
            .with_interpreter(|i| {
                self.content
                    .evaluate_spanned(i, self.span_range(), RequestedOwnership::shared())
            })?
            .0
            .expect_shared();
        value
            .as_ref_value()
            .output_to(
                Grouping::Flattened,
                &mut ToStreamContext::new(output, self.span_range()),
            )
            .into_execution_result()
    }
}

impl EmbeddedStatements {
    pub(crate) fn consume(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        self.content
            .evaluate_spanned(interpreter, self.span_range(), RequestedOwnership::owned())?
            .map(|v| v.expect_owned())
            .into_statement_result()
    }
}

pub(crate) struct ExpressionBlock {
    pub(super) label: Option<(CatchLabel, CatchLocationId)>,
    pub(super) scoped_block: ScopedBlock,
}

impl ParseSource for ExpressionBlock {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let label = input.parse_optional()?;

        // We add some special error handling here to help users avoid confusion
        // between object literals and blocks.
        let (inner, delim_span) = match input.cursor().any_group() {
            Some((inner, Delimiter::Brace, delim_span, _)) => (inner, delim_span),
            _ => {
                return input.parse_err("Expected `{ ... }` to start an expression block.");
            }
        };
        if inner.eof() {
            return delim_span.open().parse_err("An empty object literal is written `%{}` with a `%` prefix. If you intend to use an empty block here, instead use `{ None }`.");
        }
        if let Some((_, next)) = inner.ident() {
            if next.punct_matching(':').is_some() || next.punct_matching(',').is_some() {
                return delim_span.open().parse_err("An object literal must be prefixed with %, e.g. `%{ field: 1 }`. Without such a prefix, { .. } defines a block.");
            }
        }

        let scoped_block = input.parse()?;
        Ok(Self {
            label: label.map(|l| (l, CatchLocationId::new_placeholder())),
            scoped_block,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        if let Some((label, location_id)) = &mut self.label {
            *location_id = context.register_catch_location(CatchLocationData::LabeledBlock {
                label: label.ident_string(),
            });
            context.enter_catch(*location_id);
            self.scoped_block.control_flow_pass(context)?;
            context.exit_catch(*location_id);
            Ok(())
        } else {
            self.scoped_block.control_flow_pass(context)
        }
    }
}

impl HasSpan for ExpressionBlock {
    fn span(&self) -> Span {
        // We ignore the label, because it's not really part of the span of the resultant value
        self.scoped_block.span()
    }
}

impl Evaluate for ExpressionBlock {
    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let scope = interpreter.current_scope_id();

        // If this block has a label, catch breaks targeting this specific catch location
        let output = if let Some((_, catch_location)) = &self.label {
            let output_result = self.scoped_block.evaluate_spanned(interpreter, ownership);
            match interpreter.catch_control_flow(output_result, *catch_location, scope)? {
                ExecutionOutcome::Value(Spanned(value, _)) => value,
                ExecutionOutcome::ControlFlow(ControlFlowInterrupt::Break(break_interrupt)) => {
                    break_interrupt.into_requested_value(self.span_range(), ownership)?
                }
                ExecutionOutcome::ControlFlow(_) => {
                    unreachable!("Only break control flow should be catchable by labeled blocks")
                }
            }
        } else {
            // No label, just evaluate the block normally
            self.scoped_block.evaluate(interpreter, ownership)?
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

impl Evaluate for ScopedBlock {
    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        interpreter.enter_child_scope(self.scope)?;
        let output = self
            .content
            .evaluate_spanned(interpreter, self.span().into(), ownership)?;
        interpreter.exit_scope(self.scope);
        Ok(output.0)
    }
}

impl ScopedBlock {
    pub(crate) fn evaluate_owned(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Spanned<AnyValue>> {
        let span_range = self.span().span_range();
        self.evaluate(interpreter, RequestedOwnership::owned())
            .map(|value| Spanned(value.expect_owned(), span_range))
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

impl Evaluate for UnscopedBlock {
    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        self.content
            .evaluate_spanned(interpreter, self.span().into(), ownership)
            .map(|v| v.0)
    }
}

impl UnscopedBlock {
    pub(crate) fn evaluate_owned(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Spanned<AnyValue>> {
        let span_range = self.span().span_range();
        self.evaluate(interpreter, RequestedOwnership::owned())
            .map(|value| Spanned(value.expect_owned(), span_range))
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
    pub(crate) fn evaluate_spanned(
        &self,
        interpreter: &mut Interpreter,
        output_span: SpanRange,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        for (i, (statement, semicolon)) in self.statements.iter().enumerate() {
            let is_last = i == self.statements.len() - 1;
            if is_last && semicolon.is_none() {
                let value = statement.evaluate_as_returning_expression(interpreter, ownership)?;
                return Ok(value.spanned(output_span));
            } else {
                statement.evaluate_as_statement(interpreter)?;
            }
        }
        Ok(ownership.map_none(output_span)?)
    }
}
