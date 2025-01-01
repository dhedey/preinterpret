use crate::internal_prelude::*;

pub(crate) struct SetCommand;

impl CommandDefinition for SetCommand {
    const COMMAND_NAME: &'static str = "set";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let variable_name = match parse_variable_set(command.argument_tokens()) {
            Some(variable) => variable.variable_name().to_string(),
            None => {
                return command.err("A set call is expected to start with `#variable_name = ..`");
            }
        };

        let result_tokens =
            command.interpret_remaining_arguments(interpreter, SubstitutionMode::token_stream())?;
        interpreter.set_variable(variable_name, result_tokens);

        Ok(TokenStream::new())
    }
}

pub(crate) fn parse_variable_set(tokens: &mut Tokens) -> Option<Variable> {
    let variable = tokens.next_item_as_variable("").ok()?;
    tokens.next_as_punct_matching('=')?;
    Some(variable)
}

pub(crate) struct RawCommand;

impl CommandDefinition for RawCommand {
    const COMMAND_NAME: &'static str = "raw";

    fn execute(_interpreter: &mut Interpreter, command: Command) -> Result<TokenStream> {
        Ok(command.into_argument_tokens().into_token_stream())
    }
}

pub(crate) struct IgnoreCommand;

impl CommandDefinition for IgnoreCommand {
    const COMMAND_NAME: &'static str = "ignore";

    fn execute(_: &mut Interpreter, _: Command) -> Result<TokenStream> {
        Ok(TokenStream::new())
    }
}
