use crate::internal_prelude::*;

pub(crate) struct ParseCommand {
    pub(crate) input: Expression,
    #[allow(unused)]
    with_token: Ident,
    pub(crate) transformer: StreamParser,
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
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let input: OutputStream = self
            .input
            .evaluate_owned(interpreter)?
            .resolve_as("Parse input")?;
        self.transformer
            .handle_transform_from_stream(input, interpreter, output)
    }
}
