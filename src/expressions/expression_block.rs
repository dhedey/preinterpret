use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionBlock {
    marker: Token![#],
    flattening: Option<Token![..]>,
    parentheses: Parentheses,
    standard_statements: Vec<(Statement, Token![;])>,
    return_statement: Option<Statement>,
}

impl Parse<Source> for ExpressionBlock {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let marker = input.parse()?;
        let flattening = if input.peek(Token![..]) {
            Some(input.parse()?)
        } else {
            None
        };
        let (parentheses, inner) = input.parse_parentheses()?;
        let mut standard_statements = Vec::new();
        let return_statement = loop {
            if inner.is_empty() {
                break None;
            }
            let statement = inner.parse()?;
            if inner.is_empty() {
                break Some(statement);
            } else if inner.peek(Token![;]) {
                standard_statements.push((statement, inner.parse()?));
            } else {
                return inner.parse_err("Expected an operator to continue the expression, or ; to mark the end of the expression statement");
            }
        };
        Ok(Self {
            marker,
            flattening,
            parentheses,
            standard_statements,
            return_statement,
        })
    }
}

impl HasSpanRange for ExpressionBlock {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.parentheses.close())
    }
}

impl ExpressionBlock {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<ExpressionValue> {
        let output_span_range = self.span_range();
        for (statement, ..) in &self.standard_statements {
            let value = statement.interpret_to_value(interpreter)?;
            match value {
                ExpressionValue::None { .. } => {},
                other_value => return other_value.execution_err("A statement ending with ; must not return a value. If you wish to explicitly discard the expression's result, use `let _ = ...;`"),
            }
        }
        if let Some(return_statement) = &self.return_statement {
            return_statement.interpret_to_value(interpreter)
        } else {
            Ok(ExpressionValue::None(output_span_range))
        }
    }
}

impl Interpret for &ExpressionBlock {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let grouping = match self.flattening {
            Some(_) => Grouping::Flattened,
            None => Grouping::Grouped,
        };
        self.evaluate(interpreter)?
            .output_to(grouping, output, StreamOutputBehaviour::Standard)?;
        Ok(())
    }
}

impl InterpretToValue for &ExpressionBlock {
    type OutputValue = ExpressionValue;

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
    type OutputValue = ExpressionValue;

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
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let LetStatement {
            let_token,
            pattern,
            assignment,
        } = self;
        let value = match assignment {
            Some(assignment) => assignment.expression.interpret_to_value(interpreter)?,
            None => ExpressionValue::None(let_token.span.span_range()),
        };
        let mut span_range = value.span_range();
        pattern.handle_destructure(interpreter, value)?;
        span_range.set_start(let_token.span);
        Ok(ExpressionValue::None(span_range))
    }
}
