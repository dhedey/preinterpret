use crate::internal_prelude::*;

pub(crate) struct SetCommand;

impl CommandDefinition for SetCommand {
    const COMMAND_NAME: &'static str = "set";
    
    fn execute(interpreter: &mut Interpreter, argument: CommandArgumentStream, command_span: Span) -> Result<TokenStream> {
        let mut argument_tokens = argument.tokens();
        let Some(ident) = parse_variable_set(&mut argument_tokens) else {
            return Err(command_span.error("A set call is expected to start with `#VariableName = ..`."));
        };
    
        let result_tokens = interpreter.interpret_tokens(argument_tokens)?;
        interpreter.set_variable(ident.to_string(), result_tokens);
    
        return Ok(TokenStream::new());
    }
}

pub(crate) struct AsRawTokensCommand;

impl CommandDefinition for AsRawTokensCommand {
    const COMMAND_NAME: &'static str = "raw";

    fn execute(_interpreter: &mut Interpreter, argument: CommandArgumentStream, _command_span: Span) -> Result<TokenStream> {
        Ok(argument.tokens().to_token_stream())
    }
}

pub(crate) struct IgnoreCommand;

impl CommandDefinition for IgnoreCommand {
    const COMMAND_NAME: &'static str = "ignore";

    fn execute(_: &mut Interpreter, _: CommandArgumentStream, _: Span) -> Result<TokenStream> {
        Ok(TokenStream::new())
    }
}