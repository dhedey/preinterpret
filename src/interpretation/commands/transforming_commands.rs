use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct ParseCommand {
    input: SourceStreamInput,
    #[allow(unused)]
    as_token: Token![as],
    transformer: ExplicitTransformStream,
}

impl CommandType for ParseCommand {
    type OutputKind = OutputKindStreaming;
}

impl StreamingCommandDefinition for ParseCommand {
    const COMMAND_NAME: &'static str = "parse";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    input: input.parse()?,
                    as_token: input.parse()?,
                    transformer: input.parse()?,
                })
            },
            "Expected [!parse! [...] as @(...)] or [!parse! #x as @(...)] where the latter is a transform stream",
        )
    }

    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let input = self.input.interpret_to_new_stream(interpreter)?;
        self.transformer
            .handle_transform_from_stream(input, interpreter, output)
    }
}

#[derive(Clone)]
pub(crate) struct LetCommand {
    destructuring: TransformStreamUntilToken<Token![=]>,
    #[allow(unused)]
    equals: Token![=],
    arguments: SourceStream,
}

impl CommandType for LetCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for LetCommand {
    const COMMAND_NAME: &'static str = "let";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    destructuring: input.parse()?,
                    equals: input.parse()?,
                    arguments: input.parse_with_context(arguments.command_span())?,
                })
            },
            "Expected [!let! <destructuring> = ...]",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let result_tokens = self.arguments.interpret_to_new_stream(interpreter)?;
        let mut ignored_transformer_output = OutputStream::new();
        self.destructuring.handle_transform_from_stream(
            result_tokens,
            interpreter,
            &mut ignored_transformer_output,
        )
    }
}
