use super::*;

#[derive(Clone)]
pub(crate) struct EmbeddedExpression {
    marker: Token![#],
    flattening: Option<Token![..]>,
    parentheses: Parentheses,
    content: ExpressionBlockContent,
}

impl Parse<Source> for EmbeddedExpression {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let marker = input.parse()?;
        let flattening = if input.peek(Token![..]) {
            Some(input.parse()?)
        } else {
            None
        };
        let (parentheses, inner) = input.parse_parentheses()?;
        let content = inner.parse()?;
        Ok(Self {
            marker,
            flattening,
            parentheses,
            content,
        })
    }
}

impl HasSpanRange for EmbeddedExpression {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.parentheses.close())
    }
}

impl EmbeddedExpression {
    pub(crate) fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<OwnedValue> {
        self.content.evaluate(interpreter, self.span_range())
    }
}

impl Interpret for &EmbeddedExpression {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let grouping = match self.flattening {
            Some(_) => Grouping::Flattened,
            None => Grouping::Flattened,
        };
        self.evaluate(interpreter)?.output_to(grouping, output)?;
        Ok(())
    }
}

impl InterpretToValue for &EmbeddedExpression {
    type OutputValue = OwnedValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        if let Some(flattening) = &self.flattening {
            return flattening
                .execution_err("Flattening is not supported when outputting as a value");
        }
        self.evaluate(interpreter)
    }
}

#[derive(Clone)]
pub(crate) struct ExpressionBlockContent {
    standard_statements: Vec<(Statement, Token![;])>,
    return_statement: Option<Statement>,
}

impl Parse<Source> for ExpressionBlockContent {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let mut standard_statements = Vec::new();
        let return_statement = loop {
            if input.is_empty() {
                break None;
            }
            let statement = input.parse()?;
            if input.is_empty() {
                break Some(statement);
            } else if input.peek(Token![;]) {
                standard_statements.push((statement, input.parse()?));
            } else {
                return input.parse_err("Invalid statement continuation. Possibly the previous statement is missing a semicolon?");
            }
        };
        Ok(Self {
            standard_statements,
            return_statement,
        })
    }
}

impl ExpressionBlockContent {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        output_span_range: SpanRange,
    ) -> ExecutionResult<OwnedValue> {
        for (statement, ..) in &self.standard_statements {
            let (value, span) = statement.interpret_to_value(interpreter)?.deconstruct();
            match value {
                ExpressionValue::None => {},
                _ => return span.execution_err("A statement ending with ; must not return a value. If you wish to explicitly discard the expression's result, use `let _ = ...;`"),
            }
        }
        Ok(if let Some(return_statement) = &self.return_statement {
            return_statement
                .interpret_to_value(interpreter)?
                .into_inner()
        } else {
            ExpressionValue::None
        }
        .into_owned(output_span_range))
    }
}

#[derive(Clone)]
pub(crate) enum Statement {
    LetStatement(LetStatement),
    Expression(SourceExpression),
}

impl Parse<Source> for Statement {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(if input.peek(Token![let]) {
            Statement::LetStatement(input.parse()?)
        } else {
            Statement::Expression(input.parse()?)
        })
    }
}

impl InterpretToValue for &Statement {
    type OutputValue = OwnedValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        match self {
            Statement::LetStatement(assignment) => assignment.interpret_to_value(interpreter),
            Statement::Expression(expression) => expression.interpret_to_value(interpreter),
        }
    }
}

/// Note a `let x = ...;` is very different to `x = ...;` inside an expression.
/// In the former, `x` is a pattern, and any identifiers creates new variable/bindings.
/// In the latter, `x` is a place expression, and identifiers can be either place references or
/// values, e.g. `a.x[y[0]][3] = ...` has `y[0]` evaluated as a value.
#[derive(Clone)]
pub(crate) struct LetStatement {
    let_token: Token![let],
    pattern: Pattern,
    assignment: Option<LetStatementAssignment>,
}

#[derive(Clone)]
struct LetStatementAssignment {
    #[allow(unused)]
    equals: Token![=],
    expression: SourceExpression,
}

impl Parse<Source> for LetStatement {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let let_token = input.parse()?;
        let pattern = input.parse()?;
        if input.peek(Token![=]) {
            Ok(Self {
                let_token,
                pattern,
                assignment: Some(LetStatementAssignment {
                    equals: input.parse()?,
                    expression: input.parse()?,
                }),
            })
        } else if input.is_empty() || input.peek(Token![;]) {
            Ok(Self {
                let_token,
                pattern,
                assignment: None,
            })
        } else {
            input.parse_err("Expected = or ;")
        }
    }
}

impl InterpretToValue for &LetStatement {
    type OutputValue = OwnedValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let output_span_range = self.let_token.span;
        let LetStatement {
            let_token: _,
            pattern,
            assignment,
        } = self;
        let value = match assignment {
            Some(assignment) => assignment
                .expression
                .interpret_to_value(interpreter)?
                .into_inner(),
            None => ExpressionValue::None,
        };
        pattern.handle_destructure(interpreter, value)?;
        Ok(ExpressionValue::None.into_owned(output_span_range))
    }
}
