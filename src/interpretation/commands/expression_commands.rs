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

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<TokenTree> {
        Ok(self
            .expression
            .evaluate_with_span(interpreter, self.command_span)?
            .to_token_tree())
    }
}

#[derive(Clone)]
pub(crate) struct AssignCommand {
    variable: GroupedVariable,
    operator: Option<Punct>,
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
                    operator: {
                        if input.peek(Token![=]) {
                            None
                        } else {
                            let operator: Punct = input.parse()?;
                            match operator.as_char() {
                                '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
                                _ => return operator.parse_err("Expected one of + - * / % & | or ^"),
                            }
                            Some(operator)
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

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let Self {
            variable,
            operator,
            equals: _,
            expression,
            command_span,
        } = *self;

        let expression = if let Some(operator) = operator {
            let mut calculation = TokenStream::new();
            unsafe {
                // RUST-ANALYZER SAFETY: Hopefully it won't contain a none-delimited group
                variable
                    .interpret_to_new_stream(interpreter)?
                    .parse_as::<OutputExpression>()?
                    .evaluate()?
                    .to_tokens(&mut calculation);
            };
            operator.to_tokens(&mut calculation);
            expression
                .evaluate(interpreter)?
                .to_tokens(&mut calculation);
            calculation.source_parse_as()?
        } else {
            expression
        };

        let output = expression
            .evaluate_with_span(interpreter, command_span)?
            .to_token_tree();
        variable.set(interpreter, output.into())?;

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
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for RangeCommand {
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
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let range_limits = self.range_limits;
        let left = self.left.evaluate_to_value(interpreter)?;
        let right = self.right.evaluate_to_value(interpreter)?;

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

        let output_span = range_limits.span_range().start();
        output.extend_raw_tokens(range_iterator.map(|value| {
            value
                .to_token_tree(output_span)
                // We wrap it in a singleton group to ensure that negative
                // numbers are treated as single items in other stream commands
                .into_singleton_group(Delimiter::None)
        }));

        Ok(())
    }
}
