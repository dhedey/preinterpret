use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EvaluateCommand {
    expression: ExpressionInput,
}

impl CommandType for EvaluateCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for EvaluateCommand {
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

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<TokenTree> {
        let expression = self.expression.start_expression_builder(interpreter)?;
        Ok(expression.evaluate()?.into_token_tree())
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

impl CommandType for AssignCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for AssignCommand {
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

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<()> {
        let Self {
            variable,
            operator,
            equals: _,
            expression,
        } = *self;

        let mut builder = ExpressionBuilder::new();
        variable.add_to_expression(interpreter, &mut builder)?;
        builder.push_punct(operator);
        builder.extend_with_evaluation_output(expression.evaluate(interpreter)?);

        let output = builder.evaluate()?.into_token_tree();
        variable.set(interpreter, output.into())?;

        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct RangeCommand {
    left: ExpressionInput,
    range_limits: RangeLimits,
    right: ExpressionInput,
}

impl CommandType for RangeCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for RangeCommand {
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

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let range_span_range = self.range_limits.span_range();
        let range_span = self.range_limits.span();

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
            return Ok(());
        }

        let length = self
            .range_limits
            .length_of_range(left, right)
            .ok_or_else(|| {
                range_span_range.error("The range is too large to be represented as a usize")
            })?;

        interpreter
            .start_iteration_counter(&range_span_range)
            .add_and_check(length)?;

        match self.range_limits {
            RangeLimits::HalfOpen(_) => {
                output_range(left..right, range_span, output);
            }
            RangeLimits::Closed(_) => {
                output_range(left..=right, range_span, output);
            }
        };

        Ok(())
    }
}

fn output_range(iter: impl Iterator<Item = i128>, span: Span, output: &mut InterpretedStream) {
    output.extend_raw_tokens(iter.map(|value| {
        let literal = Literal::i128_unsuffixed(value).with_span(span);
        TokenTree::Literal(literal)
            // We wrap it in a singleton group to ensure that negative
            // numbers are treated as single items in other stream commands
            .into_singleton_group(Delimiter::None)
    }))
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
