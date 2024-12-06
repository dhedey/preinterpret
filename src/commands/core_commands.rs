use crate::internal_prelude::*;

pub(crate) struct SetCommand;

impl CommandDefinition for SetCommand {
    const COMMAND_NAME: &'static str = "set";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        let mut argument_tokens = argument.tokens();
        let variable_name = match parse_variable_set(&mut argument_tokens) {
            Some(ident) => ident.to_string(),
            None => {
                return Err(command_span
                    .error("A set call is expected to start with `#variable_name = ..`"));
            }
        };

        let result_tokens = interpreter.interpret_tokens(argument_tokens)?;
        interpreter.set_variable(variable_name, result_tokens);

        Ok(TokenStream::new())
    }
}

pub(crate) struct RawCommand;

impl CommandDefinition for RawCommand {
    const COMMAND_NAME: &'static str = "raw";

    fn execute(
        _interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        _command_span: Span,
    ) -> Result<TokenStream> {
        Ok(argument.tokens().into_token_stream())
    }
}

pub(crate) struct IgnoreCommand;

impl CommandDefinition for IgnoreCommand {
    const COMMAND_NAME: &'static str = "ignore";

    fn execute(_: &mut Interpreter, _: CommandArgumentStream, _: Span) -> Result<TokenStream> {
        Ok(TokenStream::new())
    }
}
