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
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        match self {
            Statement::Expression(expression) => expression.evaluate(interpreter, ownership),
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
            Some(assignment) => assignment
                .expression
                .evaluate_owned(interpreter)?
                .into_inner(),
            None => ExpressionValue::None,
        };
        pattern.handle_destructure(interpreter, value)?;
        Ok(())
    }
}

pub(crate) struct BreakStatement {
    break_token: Token![break],
    label: Option<syn::Lifetime>,
    value: Option<Expression>,
    target_catch_location: Option<CatchLocationId>,
}

impl HasSpan for BreakStatement {
    fn span(&self) -> Span {
        self.break_token.span
    }
}

impl ParseSource for BreakStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let break_token = input.parse()?;

        let label = if input.cursor().lifetime().is_some() {
            Some(input.parse()?)
        } else {
            None
        };

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
            target_catch_location: None,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        if let Some(value) = &mut self.value {
            value.control_flow_pass(context)?;
        }
        // Resolve label to catch location if present
        if let Some(label) = &self.label {
            self.target_catch_location =
                context.resolve_label_to_catch_location(&label.ident.to_string());
            // If label doesn't resolve, it's an error
            if self.target_catch_location.is_none() {
                return self
                    .break_token
                    .span
                    .parse_err(format!("label '{}' not found in scope", label.ident));
            }
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
            Some(expr.evaluate_owned(interpreter)?)
        } else {
            None
        };

        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::new_break(self.target_catch_location, value),
            self.break_token.span,
        ))
    }
}

pub(crate) struct ContinueStatement {
    continue_token: Token![continue],
    label: Option<syn::Lifetime>,
    target_catch_location: Option<CatchLocationId>,
}

impl HasSpan for ContinueStatement {
    fn span(&self) -> Span {
        self.continue_token.span
    }
}

impl ParseSource for ContinueStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let continue_token = input.parse()?;

        let label = if input.cursor().lifetime().is_some() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            continue_token,
            label,
            target_catch_location: None,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        // Resolve label to catch location if present
        if let Some(label) = &self.label {
            self.target_catch_location =
                context.resolve_label_to_catch_location(&label.ident.to_string());
            // If label doesn't resolve, it's an error
            if self.target_catch_location.is_none() {
                return self
                    .continue_token
                    .span
                    .parse_err(format!("label '{}' not found in scope", label.ident));
            }
        }
        Ok(())
    }
}

impl ContinueStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::new_continue(self.target_catch_location),
            self.continue_token.span,
        ))
    }
}

pub(crate) struct RevertStatement {
    revert: RevertKeyword,
    target_catch_location: Option<CatchLocationId>,
}

impl HasSpan for RevertStatement {
    fn span(&self) -> Span {
        self.revert.span()
    }
}

impl ParseSource for RevertStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let revert = input.parse()?;
        Ok(Self {
            revert,
            target_catch_location: None,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        // For now, revert doesn't resolve to a specific catch location during parse
        // It will be caught by the nearest attempt block at runtime
        Ok(())
    }
}

impl RevertStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::Revert,
            self.revert.span(),
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
        let value = self.expression.evaluate_owned(interpreter)?;
        value.output_to(
            Grouping::Flattened,
            &mut ToStreamContext::new(interpreter.output(&self.emit)?, value.span_range()),
        )
    }
}
