use crate::internal_prelude::*;

pub(crate) struct EmptyCommand;

impl CommandDefinition for EmptyCommand {
    const COMMAND_NAME: &'static str = "empty";

    fn execute(_interpreter: &mut Interpreter, command: Command) -> Result<TokenStream> {
        command
            .into_argument_tokens()
            .assert_end("The !empty! command does not take any arguments")?;
        Ok(TokenStream::new())
    }
}

pub(crate) struct IsEmptyCommand;

impl CommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let interpreted =
            command.interpret_remaining_arguments(interpreter, SubstitutionMode::token_stream())?;
        Ok(TokenTree::bool(interpreted.is_empty(), command.span()).into())
    }
}

pub(crate) struct LengthCommand;

impl CommandDefinition for LengthCommand {
    const COMMAND_NAME: &'static str = "length";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let interpreted =
            command.interpret_remaining_arguments(interpreter, SubstitutionMode::token_stream())?;
        let stream_length = interpreted.into_iter().count();
        Ok(
            TokenTree::Literal(Literal::usize_unsuffixed(stream_length).with_span(command.span()))
                .into(),
        )
    }
}

pub(crate) struct GroupCommand;

impl CommandDefinition for GroupCommand {
    const COMMAND_NAME: &'static str = "group";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let mut output = TokenStream::new();
        output.push_new_group(
            command.interpret_remaining_arguments(interpreter, SubstitutionMode::token_stream())?,
            Delimiter::None,
            command.span_range(),
        );
        Ok(output)
    }
}
