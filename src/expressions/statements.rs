use super::*;

pub(crate) enum Statement {
    LetStatement(LetStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    RevertStatement(RevertStatement),
    EmitStatement(EmitStatement),
    Expression(Expression),
}

impl Statement {
    pub(crate) fn requires_semicolon(&self, last_line: bool) -> bool {
        match self {
            Statement::LetStatement(_) => true,
            Statement::BreakStatement(_) => true,
            Statement::ContinueStatement(_) => true,
            Statement::RevertStatement(_) => true,
            Statement::EmitStatement(_) => true,
            Statement::Expression(expression) => {
                !(expression.is_valid_as_statement_without_semicolon() || last_line)
            }
        }
    }
}

impl ParseSource for Statement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(if let Some((ident, _)) = input.cursor().ident() {
            match ident.to_string().as_str() {
                "let" => Statement::LetStatement(input.parse()?),
                "break" => Statement::BreakStatement(input.parse()?),
                "continue" => Statement::ContinueStatement(input.parse()?),
                keyword::REVERT => Statement::RevertStatement(input.parse()?),
                keyword::EMIT => Statement::EmitStatement(input.parse()?),
                _ => Statement::Expression(input.parse()?),
            }
        } else {
            Statement::Expression(input.parse()?)
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            Statement::LetStatement(statement) => statement.control_flow_pass(context),
            Statement::Expression(expression) => expression.control_flow_pass(context),
            Statement::BreakStatement(statement) => statement.control_flow_pass(context),
            Statement::ContinueStatement(statement) => statement.control_flow_pass(context),
            Statement::RevertStatement(statement) => statement.control_flow_pass(context),
            Statement::EmitStatement(statement) => statement.control_flow_pass(context),
        }
    }
}

impl Statement {
    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        match self {
            Statement::LetStatement(statement) => statement.evaluate_as_statement(interpreter),
            Statement::Expression(expression) => expression.evaluate_as_statement(interpreter),
            Statement::BreakStatement(statement) => statement.evaluate_as_statement(interpreter),
            Statement::ContinueStatement(statement) => statement.evaluate_as_statement(interpreter),
            Statement::RevertStatement(statement) => statement.evaluate_as_statement(interpreter),
            Statement::EmitStatement(statement) => statement.evaluate_as_statement(interpreter),
        }
    }

    pub(crate) fn evaluate_as_returning_expression(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        match self {
            Statement::Expression(expression) => {
                expression.evaluate(interpreter, ownership).map(|v| v.0)
            }
            Statement::LetStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::RevertStatement(_)
            | Statement::EmitStatement(_) => {
                panic!("Statements cannot be used as returning expressions")
            }
        }
    }
}

/// Note a `let x = ...;` is very different to `x = ...;` inside an expression.
/// In the former, `x` is a pattern, and any identifiers creates new variable/bindings.
/// In the latter, `x` is a place expression, and identifiers can be either place references or
/// values, e.g. `a.x[y[0]][3] = ...` has `y[0]` evaluated as a value.
pub(crate) struct LetStatement {
    _let_token: Token![let],
    pattern: Pattern,
    assignment: Option<LetStatementAssignment>,
}

struct LetStatementAssignment {
    #[allow(unused)]
    equals: Token![=],
    expression: Expression,
}

impl ParseSource for LetStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let let_token = input.parse()?;
        let pattern = input.parse()?;
        if input.peek(Token![=]) {
            Ok(Self {
                _let_token: let_token,
                pattern,
                assignment: Some(LetStatementAssignment {
                    equals: input.parse()?,
                    expression: input.parse()?,
                }),
            })
        } else if input.is_empty() || input.peek(Token![;]) {
            Ok(Self {
                _let_token: let_token,
                pattern,
                assignment: None,
            })
        } else {
            input.parse_err("Expected = or ;")
        }
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        if let Some(assignment) = self.assignment.as_mut() {
            assignment.expression.control_flow_pass(context)?;
        }
        self.pattern.control_flow_pass(context)?;
        Ok(())
    }
}

impl LetStatement {
    fn evaluate_as_statement(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let LetStatement {
            _let_token: _,
            pattern,
            assignment,
        } = self;
        let value = match assignment {
            Some(assignment) => assignment.expression.evaluate_owned(interpreter)?.0 .0,
            None => ().into_value(),
        };
        pattern.handle_destructure(interpreter, value)?;
        Ok(())
    }
}

pub(crate) struct InterruptLabel {
    label: syn::Lifetime,
}

impl ParseSource for InterruptLabel {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let label = input.parse()?;
        Ok(Self { label })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl ParseSourceOptional for InterruptLabel {}

impl HasSpanRange for InterruptLabel {
    fn span_range(&self) -> SpanRange {
        self.label.span_range()
    }
}

impl InterruptLabel {
    pub(crate) fn ident_string(&self) -> String {
        self.label.ident.to_string()
    }
}

pub(crate) struct BreakStatement {
    break_token: Token![break],
    label: Option<InterruptLabel>,
    value: Option<Expression>,
    target_catch_location: CatchLocationId,
}

impl HasSpanRange for BreakStatement {
    fn span_range(&self) -> SpanRange {
        let last = self
            .label
            .as_ref()
            .map(|l| l.end_span())
            .unwrap_or_else(|| self.break_token.span);
        SpanRange::new_between(self.break_token.span, last)
    }
}

impl ParseSource for BreakStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let break_token = input.parse()?;
        let label = input.parse_optional()?;

        // Try to parse an optional expression value
        let value = if !input.is_empty() && !input.peek(Token![;]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            break_token,
            label,
            value,
            target_catch_location: CatchLocationId::new_placeholder(),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.target_catch_location =
            context.resolve_catch_for_interrupt(InterruptDetails::Break {
                break_token: &self.break_token,
                label: self.label.as_ref(),
            })?;
        if let Some(value) = &mut self.value {
            value.control_flow_pass(context)?;
        }
        Ok(())
    }
}

impl BreakStatement {
    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        let value = if let Some(expr) = &self.value {
            Some(expr.evaluate_owned(interpreter)?.0)
        } else {
            None
        };

        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::new_break(self.target_catch_location, value),
        ))
    }
}

pub(crate) struct ContinueStatement {
    target_catch_location: CatchLocationId,
    continue_token: Token![continue],
    label: Option<InterruptLabel>,
}

impl HasSpanRange for ContinueStatement {
    fn span_range(&self) -> SpanRange {
        let last = self
            .label
            .as_ref()
            .map(|l| l.end_span())
            .unwrap_or_else(|| self.continue_token.span);
        SpanRange::new_between(self.continue_token.span, last)
    }
}

impl ParseSource for ContinueStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let continue_token = input.parse()?;
        let label = input.parse_optional()?;

        Ok(Self {
            target_catch_location: CatchLocationId::new_placeholder(),
            continue_token,
            label,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.target_catch_location =
            context.resolve_catch_for_interrupt(InterruptDetails::Continue {
                label: self.label.as_ref(),
                continue_token: &self.continue_token,
            })?;
        Ok(())
    }
}

impl ContinueStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::new_continue(self.target_catch_location),
        ))
    }
}

pub(crate) struct RevertStatement {
    revert: RevertKeyword,
    label: Option<InterruptLabel>,
    target_catch_location: CatchLocationId,
}

impl HasSpanRange for RevertStatement {
    fn span_range(&self) -> SpanRange {
        let last = self
            .label
            .as_ref()
            .map(|l| l.end_span())
            .unwrap_or_else(|| self.revert.span());
        SpanRange::new_between(self.revert.span(), last)
    }
}

impl ParseSource for RevertStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(Self {
            revert: input.parse()?,
            label: input.parse_optional()?,
            target_catch_location: CatchLocationId::new_placeholder(),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.target_catch_location =
            context.resolve_catch_for_interrupt(InterruptDetails::Revert {
                revert_token: &self.revert,
                label: self.label.as_ref(),
            })?;
        Ok(())
    }
}

impl RevertStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::new_revert(self.target_catch_location),
        ))
    }
}

pub(crate) struct EmitStatement {
    emit: EmitKeyword,
    expression: Expression,
}

impl HasSpan for EmitStatement {
    fn span(&self) -> Span {
        self.emit.span()
    }
}

impl ParseSource for EmitStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let emit = input.parse()?;
        let expression = input.parse()?;
        Ok(Self { emit, expression })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.expression.control_flow_pass(context)
    }
}

impl EmitStatement {
    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        let Spanned(value, span) = self.expression.evaluate_owned(interpreter)?;
        value.output_to(
            Grouping::Flattened,
            &mut ToStreamContext::new(interpreter.output(&self.emit)?, span),
        )
    }
}
