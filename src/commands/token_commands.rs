use crate::internal_prelude::*;

pub(crate) struct EmptyCommand;

impl CommandDefinition for EmptyCommand {
    const COMMAND_NAME: &'static str = "empty";

    fn execute(_interpreter: &mut Interpreter, mut command: Command) -> Result<InterpretedStream> {
        command
            .arguments()
            .assert_end("The !empty! command does not take any arguments")?;
        Ok(InterpretedStream::new())
    }
}

pub(crate) struct IsEmptyCommand;

impl CommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<InterpretedStream> {
        let interpreted = command.arguments().interpret_as_tokens(interpreter)?;
        Ok(TokenTree::bool(interpreted.is_empty(), command.span()).into())
    }
}

pub(crate) struct LengthCommand;

impl CommandDefinition for LengthCommand {
    const COMMAND_NAME: &'static str = "length";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<InterpretedStream> {
        let interpreted = command.arguments().interpret_as_tokens(interpreter)?;
        let stream_length = interpreted.into_token_stream().into_iter().count();
        let length_literal = Literal::usize_unsuffixed(stream_length).with_span(command.span());
        Ok(InterpretedStream::of_literal(length_literal))
    }
}

pub(crate) struct GroupCommand;

impl CommandDefinition for GroupCommand {
    const COMMAND_NAME: &'static str = "group";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<InterpretedStream> {
        let mut output = InterpretedStream::new();
        output.push_new_group(
            command.arguments().interpret_as_tokens(interpreter)?,
            Delimiter::None,
            command.span_range(),
        );
        Ok(output)
    }
}
