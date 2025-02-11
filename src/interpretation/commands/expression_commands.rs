use crate::internal_prelude::*;

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
        let left = self.left.interpret_to_value(interpreter)?;
        let right = self.right.interpret_to_value(interpreter)?;

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
            // It needs to be grouped so that e.g. -1 is interpreted as a single item, not two separate tokens.
            value.output_to(Grouping::Grouped, output)
        }

        Ok(())
    }
}
