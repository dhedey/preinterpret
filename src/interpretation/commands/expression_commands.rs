use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct RangeCommand {
    span: Span,
    left: SourceExpression,
    range_limits: syn::RangeLimits,
    right: SourceExpression,
}

impl CommandType for RangeCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for RangeCommand {
    const COMMAND_NAME: &'static str = "range";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    span: arguments.command_span(),
                    left: input.parse()?,
                    range_limits: input.parse()?,
                    right: input.parse()?,
                })
            },
            "Expected a rust range expression such as [!range! 1..4]",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let range_limits = self.range_limits;
        let left = self.left.interpret_to_value(interpreter)?;
        let right = self.right.interpret_to_value(interpreter)?;
        let range_iterator = left.create_range(right, &range_limits)?;
        Ok(range_iterator.to_value(self.span.span_range()))
    }
}
