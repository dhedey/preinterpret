use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct LetCommand {
    destructuring: DestructureUntil<Token![=]>,
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

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let result_tokens = self.arguments.interpret_to_new_stream(interpreter)?;
        self.destructuring
            .handle_destructure_from_stream(result_tokens, interpreter)
    }
}
