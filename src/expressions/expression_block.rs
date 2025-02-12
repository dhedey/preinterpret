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
        self.evaluate(interpreter)?.output_to(grouping, output);
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
    Assignment(AssignmentStatement),
    Expression(SourceExpression),
}

impl Parse<Source> for Statement {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(match input.cursor().ident() {
            // let or some ident for a variable
            // It may be the start of an assignment.
            Some((ident, _)) if { let str = ident.to_string(); str != "true" && str != "false" } => {
                let forked = input.fork();
                match forked.call(|input| {
                    let destination = input.parse()?;
                    let equals = input.parse()?;
                    Ok((destination, equals))
                }) {
                    Ok((destination, equals)) => {
                        input.advance_to(&forked);
                        // We commit to the fork after successfully parsing the destination and equals.
                        // This gives better error messages, if there is an error in the expression itself.
                        Statement::Assignment(AssignmentStatement {
                            destination,
                            equals,
                            expression: input.parse()?,
                        })
                    },
                    Err(_) => Statement::Expression(input.parse()?),
                }
            }
            _ => Statement::Expression(input.parse()?),
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
            Statement::Assignment(assignment) => assignment.interpret_to_value(interpreter),
            Statement::Expression(expression) => expression.interpret_to_value(interpreter),
        }
    }
}

#[derive(Clone)]
pub(crate) struct AssignmentStatement {
    destination: Destination,
    #[allow(unused)]
    equals: Token![=],
    expression: SourceExpression,
}

impl Parse<Source> for AssignmentStatement {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            destination: input.parse()?,
            equals: input.parse()?,
            expression: input.parse()?,
        })
    }
}

impl InterpretToValue for &AssignmentStatement {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        match &self.destination {
            Destination::NewVariable { let_token, path } => {
                let value = self.expression.interpret_to_value(interpreter)?;
                let mut span_range = value.span_range();
                path.set_value(interpreter, value)?;
                span_range.set_start(let_token.span);
                Ok(ExpressionValue::None(span_range))
            }
            Destination::ExistingVariable { path, operation } => {
                let value = if let Some(operation) = operation {
                    let left = path.interpret_to_value(interpreter)?;
                    let right = self.expression.interpret_to_value(interpreter)?;
                    operation.evaluate(left, right)?
                } else {
                    self.expression.interpret_to_value(interpreter)?
                };
                let mut span_range = value.span_range();
                path.set_value(interpreter, value)?;
                span_range.set_start(path.span_range().start());
                Ok(ExpressionValue::None(span_range))
            }
            Destination::Discarded { let_token, .. } => {
                let value = self.expression.interpret_to_value(interpreter)?;
                let mut span_range = value.span_range();
                span_range.set_start(let_token.span);
                Ok(ExpressionValue::None(span_range))
            }
        }
    }
}

#[derive(Clone)]
enum Destination {
    NewVariable {
        let_token: Token![let],
        path: VariablePath,
    },
    ExistingVariable {
        path: VariablePath,
        operation: Option<BinaryOperation>,
    },
    Discarded {
        let_token: Token![let],
        #[allow(unused)]
        discarded_token: Token![_],
    },
}

impl Parse<Source> for Destination {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        if input.peek(Token![let]) {
            let let_token = input.parse()?;
            if input.peek(Token![_]) {
                Ok(Self::Discarded {
                    let_token,
                    discarded_token: input.parse()?,
                })
            } else {
                Ok(Self::NewVariable {
                    let_token,
                    path: input.parse()?,
                })
            }
        } else {
            Ok(Self::ExistingVariable {
                path: input.parse()?,
                operation: if input.peek(Token![=]) {
                    None
                } else {
                    let operator_char = match input.cursor().punct() {
                        Some((operator, _)) => operator.as_char(),
                        None => 'X',
                    };
                    match operator_char {
                        '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
                        _ => {
                            return input
                                .parse_err("Expected = or one of += -= *= /= %= &= |= or ^=")
                        }
                    }
                    Some(input.parse()?)
                },
            })
        }
    }
}
