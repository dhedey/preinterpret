use super::*;

#[derive(Clone)]
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

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
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

#[derive(Clone)]
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

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
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

#[derive(Clone)]
pub(crate) struct ExpressionBlock {
    pub(super) braces: Braces,
    pub(super) scope: ScopeId,
    pub(super) content: ExpressionBlockContent,
}

impl ParseSource for ExpressionBlock {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (braces, inner) = input.parse_braces()?;
        let scope = input.register_scope();
        let content = inner.parse()?;
        Ok(Self {
            braces,
            scope,
            content,
        })
    }

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
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

#[derive(Clone)]
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

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        for (statement, _semicolon) in self.statements.iter() {
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

#[derive(Clone)]
pub(crate) enum Statement {
    LetStatement(LetStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    Expression(Expression),
}

impl Statement {
    fn requires_semicolon(&self, last_line: bool) -> bool {
        match self {
            Statement::LetStatement(_) => true,
            Statement::BreakStatement(_) => true,
            Statement::ContinueStatement(_) => true,
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
                _ => Statement::Expression(input.parse()?),
            }
        } else {
            Statement::Expression(input.parse()?)
        })
    }

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            Statement::LetStatement(statement) => statement.control_flow_pass(context),
            Statement::Expression(expression) => expression.control_flow_pass(context),
            Statement::BreakStatement(statement) => statement.control_flow_pass(context),
            Statement::ContinueStatement(statement) => statement.control_flow_pass(context),
        }
    }
}

impl Statement {
    fn evaluate_as_statement(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        match self {
            Statement::LetStatement(statement) => statement.evaluate_as_statement(interpreter),
            Statement::Expression(expression) => expression.evaluate_as_statement(interpreter),
            Statement::BreakStatement(statement) => statement.evaluate_as_statement(interpreter),
            Statement::ContinueStatement(statement) => statement.evaluate_as_statement(interpreter),
        }
    }

    fn evaluate_as_returning_expression(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        match self {
            Statement::Expression(expression) => expression.evaluate(interpreter, ownership),
            Statement::LetStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_) => {
                panic!("Statements cannot be used as returning expressions")
            }
        }
    }
}

/// Note a `let x = ...;` is very different to `x = ...;` inside an expression.
/// In the former, `x` is a pattern, and any identifiers creates new variable/bindings.
/// In the latter, `x` is a place expression, and identifiers can be either place references or
/// values, e.g. `a.x[y[0]][3] = ...` has `y[0]` evaluated as a value.
#[derive(Clone)]
pub(crate) struct LetStatement {
    _let_token: Token![let],
    pattern: Pattern,
    assignment: Option<LetStatementAssignment>,
}

#[derive(Clone)]
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

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        if let Some(assignment) = self.assignment.as_ref() {
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

#[derive(Clone)]
pub(crate) struct BreakStatement {
    break_token: Ident,
}

impl HasSpan for BreakStatement {
    fn span(&self) -> Span {
        self.break_token.span()
    }
}

impl ParseSource for BreakStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let break_token = input.parse_ident_matching("break")?;
        Ok(Self { break_token })
    }

    fn control_flow_pass(&self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl BreakStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::Break,
            self.break_token.span(),
        ))
    }
}

#[derive(Clone)]
pub(crate) struct ContinueStatement {
    continue_token: Ident,
}

impl HasSpan for ContinueStatement {
    fn span(&self) -> Span {
        self.continue_token.span()
    }
}

impl ParseSource for ContinueStatement {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let continue_token = input.parse_ident_matching("continue")?;
        Ok(Self { continue_token })
    }

    fn control_flow_pass(&self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl ContinueStatement {
    pub(crate) fn evaluate_as_statement(&self, _: &mut Interpreter) -> ExecutionResult<()> {
        Err(ExecutionInterrupt::control_flow(
            ControlFlowInterrupt::Continue,
            self.continue_token.span(),
        ))
    }
}
