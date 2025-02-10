use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EvaluateCommand {
    expression: SourceExpression,
    command_span: Span,
}

impl CommandType for EvaluateCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for EvaluateCommand {
    const COMMAND_NAME: &'static str = "evaluate";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    expression: input.parse()?,
                    command_span: arguments.command_span(),
                })
            },
            "Expected [!evaluate! ...] containing a valid preinterpret expression",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let value = self
            .expression
            .evaluate(interpreter)?
            .with_span(self.command_span);
        Ok(value)
    }
}

#[derive(Clone)]
pub(crate) struct AssignCommand {
    variable: GroupedVariable,
    operation: Option<BinaryOperation>,
    #[allow(unused)]
    equals: Token![=],
    expression: SourceExpression,
    command_span: Span,
}

impl CommandType for AssignCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for AssignCommand {
    const COMMAND_NAME: &'static str = "assign";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: input.parse()?,
                    operation: {
                        if input.peek(Token![=]) {
                            None
                        } else {
                            let operator_char = match input.cursor().punct() {
                                Some((operator, _)) => operator.as_char(),
                                None => 'X',
                            };
                            match operator_char {
                                '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
                                _ => return input.parse_err("Expected one of + - * / % & | or ^"),
                            }
                            Some(input.parse()?)
                        }
                    },
                    equals: input.parse()?,
                    expression: input.parse()?,
                    command_span: arguments.command_span(),
                })
            },
            "Expected [!assign! #variable = <expression>] or [!assign! #variable X= <expression>] for X one of + - * / % & | or ^",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let Self {
            variable,
            operation,
            equals: _,
            expression,
            command_span,
        } = self;

        let value = if let Some(operation) = operation {
            let left = variable.read_as_expression_value(interpreter)?;
            let right = expression.evaluate(interpreter)?;
            operation.evaluate(left, right)?
        } else {
            expression.evaluate(interpreter)?
        };

        variable.set_value(interpreter, value.with_span(command_span))?;

        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct RangeCommand {
    left: SourceExpression,
    range_limits: syn::RangeLimits,
    right: SourceExpression,
}

impl CommandType for RangeCommand {
    type OutputKind = OutputKindGroupedStream;
}

impl GroupedStreamCommandDefinition for RangeCommand {
    const COMMAND_NAME: &'static str = "range";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    left: input.parse()?,
                    range_limits: input.parse()?,
                    right: input.parse()?,
                })
            },
            "Expected a rust range expression such as [!range! 1..4]",
        )
    }

    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let range_limits = self.range_limits;
        let left = self.left.evaluate(interpreter)?;
        let right = self.right.evaluate(interpreter)?;

        let range_iterator = left.create_range(right, &range_limits)?;

        let (_, length) = range_iterator.size_hint();
        match length {
            Some(length) => {
                interpreter
                    .start_iteration_counter(&range_limits)
                    .add_and_check(length)?;
            }
            None => {
                return range_limits
                    .execution_err("The range must be between two integers or two characters");
            }
        }

        for value in range_iterator {
            value.output_to(output)
        }

        Ok(())
    }
}
