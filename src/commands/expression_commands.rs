use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EvaluateCommand {
    expression: ExpressionInput,
}

impl CommandDefinition for EvaluateCommand {
    const COMMAND_NAME: &'static str = "evaluate";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    expression: input.parse()?,
                })
            },
            "Expected [!evaluate! ...] containing a valid preinterpret expression",
        )
    }
}

impl CommandInvocation for EvaluateCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let expression = self.expression.start_expression_builder(interpreter)?;
        Ok(CommandOutput::GroupedStream(
            expression.evaluate()?.into_interpreted_stream(),
        ))
    }
}

#[derive(Clone)]
pub(crate) struct AssignCommand {
    variable: GroupedVariable,
    operator: Punct,
    #[allow(unused)]
    equals: Token![=],
    expression: ExpressionInput,
}

impl CommandDefinition for AssignCommand {
    const COMMAND_NAME: &'static str = "assign";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: input.parse()?,
                    operator: {
                        let operator: Punct = input.parse()?;
                        match operator.as_char() {
                            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
                            _ => return operator.err("Expected one of + - * / % & | or ^"),
                        }
                        operator
                    },
                    equals: input.parse()?,
                    expression: input.parse()?,
                })
            },
            "Expected [!assign! #variable += ...] for + or some other operator supported in an expression",
        )
    }
}

impl CommandInvocation for AssignCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let Self {
            variable,
            operator,
            equals: _,
            expression,
        } = *self;

        let mut builder = ExpressionBuilder::new();
        variable.add_to_expression(interpreter, &mut builder)?;
        builder.push_punct(operator);
        builder.extend_with_interpreted_stream(
            expression.evaluate(interpreter)?.into_interpreted_stream(),
        );

        let output = builder.evaluate()?.into_interpreted_stream();
        variable.set(interpreter, output)?;

        Ok(CommandOutput::Empty)
    }
}

#[derive(Clone)]
pub(crate) struct RangeCommand {
    left: ExpressionInput,
    range_limits: RangeLimits,
    right: ExpressionInput,
}

impl CommandDefinition for RangeCommand {
    const COMMAND_NAME: &'static str = "range";

    fn parse(arguments: CommandArguments) -> Result<Self> {
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
}

impl CommandInvocation for RangeCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let range_span_range = self.range_limits.span_range();

        let left = self
            .left
            .evaluate(interpreter)?
            .expect_integer("The left side of the range must be an integer")?
            .try_into_i128()?;
        let right = self
            .right
            .evaluate(interpreter)?
            .expect_integer("The right side of the range must be an integer")?
            .try_into_i128()?;
        if left > right {
            return Ok(CommandOutput::Empty);
        }

        let length = self
            .range_limits
            .length_of_range(left, right)
            .ok_or_else(|| {
                range_span_range.error("The range is too large to be represented as a usize")
            })?;
        interpreter
            .config()
            .check_iteration_count(&range_span_range, length)?;

        let mut output = InterpretedStream::new(range_span_range);
        match self.range_limits {
            RangeLimits::HalfOpen(_) => {
                let iter =
                    (left..right).map(|value| TokenTree::Literal(Literal::i128_unsuffixed(value)));
                output.extend_raw_token_iter(iter)
            }
            RangeLimits::Closed(_) => {
                let iter =
                    (left..=right).map(|value| TokenTree::Literal(Literal::i128_unsuffixed(value)));
                output.extend_raw_token_iter(iter)
            }
        };
        Ok(CommandOutput::GroupedStream(output))
    }
}

// A copy of syn::RangeLimits to avoid needing a `full` dependency on syn
#[derive(Clone)]
enum RangeLimits {
    HalfOpen(Token![..]),
    Closed(Token![..=]),
}

impl Parse for RangeLimits {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![..=]) {
            Ok(RangeLimits::Closed(input.parse()?))
        } else {
            Ok(RangeLimits::HalfOpen(input.parse()?))
        }
    }
}

impl ToTokens for RangeLimits {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RangeLimits::HalfOpen(token) => token.to_tokens(tokens),
            RangeLimits::Closed(token) => token.to_tokens(tokens),
        }
    }
}

impl AutoSpanRange for RangeLimits {}

impl RangeLimits {
    fn length_of_range(&self, left: i128, right: i128) -> Option<usize> {
        match self {
            RangeLimits::HalfOpen(_) => usize::try_from(right.checked_sub(left)?).ok(),
            RangeLimits::Closed(_) => {
                usize::try_from(right.checked_sub(left)?.checked_add(1)?).ok()
            }
        }
    }
}
