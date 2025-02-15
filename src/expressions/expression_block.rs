use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionBlock {
    marker: Token![#],
    flattening: Option<Token![..]>,
    delim_span: DelimSpan,
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
        let (delim_span, inner) = input.parse_specific_group(Delimiter::Parenthesis)?;
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
            delim_span,
            standard_statements,
            return_statement,
        })
    }
}

impl HasSpanRange for ExpressionBlock {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.delim_span.close())
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

/// In a rust expression, assignments are allowed in the middle of an expression.
///
/// But the following is rather hard to parse in a streaming manner,
/// due to ambiguity and right-associativity of =
/// ```rust,ignore
/// let a;
/// let b;
/// // When the = (4,) is revealed, the `(b,)` changes from a value to a destructuring
/// let out = a = (b,) = (4,);
/// // When the += 2 is revealed, b changes from a value to a place
/// let out = b += 2;
/// ```
#[derive(Clone)]
pub(crate) struct LetStatement {
    let_token: Token![let],
    pattern: Pattern,
    #[allow(unused)]
    equals: Token![=],
    expression: SourceExpression,
}

impl Parse<Source> for LetStatement {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            let_token: input.parse()?,
            pattern: input.parse()?,
            equals: input.parse()?,
            expression: input.parse()?,
        })
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
            expression,
            ..
        } = self;
        let value = expression.interpret_to_value(interpreter)?;
        let mut span_range = value.span_range();
        pattern.handle_destructure(interpreter, value)?;
        span_range.set_start(let_token.span);
        Ok(ExpressionValue::None(span_range))
    }
}
