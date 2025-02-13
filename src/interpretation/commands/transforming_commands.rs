use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct ParseCommand {
    input: SourceExpression,
    #[allow(unused)]
    with_token: Ident,
    transformer: ExplicitTransformStream,
}

impl CommandType for ParseCommand {
    type OutputKind = OutputKindStream;
}

impl StreamingCommandDefinition for ParseCommand {
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
            "Expected [!parse! <stream> with <parser>] where:\n* The <stream> is some stream-valued expression, such as `#x` or `[!stream! ...]`\n* The <parser> is some parser such as @(...)",
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
