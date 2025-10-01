use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct ParseCommand {
    input: SourceExpression,
    #[allow(unused)]
    with_token: Ident,
    transformer: StreamParser,
}

impl CommandType for ParseCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for ParseCommand {
    const COMMAND_NAME: &'static str = "parse";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    input: input.parse()?,
                    with_token: input.parse_ident_matching("with")?,
                    transformer: input.parse()?,
                })
            },
            "Expected [!parse! <stream> with <parser>] where:\n* The <stream> is some stream-valued expression, such as `#x` or `%[...]`\n* The <parser> is some parser such as @(...)",
        )
    }

    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let input = self
            .input
            .interpret_to_value(interpreter)?
            .expect_stream("Parse input")?
            .value;
        self.transformer
            .handle_transform_from_stream(input, interpreter, output)
    }
}
