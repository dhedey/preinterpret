use super::*;

pub(crate) mod keywords {
    pub(crate) const REVERT: &str = "revert";
    pub(crate) const EMIT: &str = "emit";
    pub(crate) const ATTEMPT: &str = "attempt";
}

pub(crate) fn is_keyword(ident: &str) -> bool {
    matches!(ident, keywords::REVERT | keywords::EMIT | keywords::ATTEMPT)
}

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
                keywords::REVERT => Statement::RevertStatement(input.parse()?),
                keywords::EMIT => Statement::EmitStatement(input.parse()?),
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
    break_token: Ident,
    label: Option<syn::Lifetime>,
    value: Option<Expression>,
}

impl HasSpan for BreakStatement {
    fn span(&self) -> Span {
        self.break_token.span()
    }
}

impl ParseSource for BreakStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let break_token = input.parse_ident_matching("break")?;

        // Try to parse an optional label
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
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
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
            Some(expr.evaluate_owned(interpreter)?)
        } else {
            None
        };

        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::Break {
                label: self.label.clone(),
                value,
            },
            self.break_token.span(),
        ))
    }
}

pub(crate) struct ContinueStatement {
    continue_token: Ident,
    label: Option<syn::Lifetime>,
}

impl HasSpan for ContinueStatement {
    fn span(&self) -> Span {
        self.continue_token.span()
    }
}

impl ParseSource for ContinueStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let continue_token = input.parse_ident_matching("continue")?;

        // Try to parse an optional label
        let label = if input.cursor().lifetime().is_some() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            continue_token,
            label,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl ContinueStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::Continue {
                label: self.label.clone(),
            },
            self.continue_token.span(),
        ))
    }
}

pub(crate) struct RevertStatement {
    revert_token: Ident,
}

impl HasSpan for RevertStatement {
    fn span(&self) -> Span {
        self.revert_token.span()
    }
}

impl ParseSource for RevertStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let revert_token = input.parse_ident_matching(keywords::REVERT)?;
        Ok(Self { revert_token })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl RevertStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::Revert,
            self.revert_token.span(),
        ))
    }
}

pub(crate) struct EmitStatement {
    emit_token: Ident,
    expression: Expression,
}

impl HasSpan for EmitStatement {
    fn span(&self) -> Span {
        self.emit_token.span()
    }
}

impl ParseSource for EmitStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let emit_token = input.parse_ident_matching(keywords::EMIT)?;
        let expression = input.parse()?;
        Ok(Self {
            emit_token,
            expression,
        })
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
            &mut ToStreamContext::new(interpreter.output(&self.emit_token)?, value.span_range()),
        )
    }
}
