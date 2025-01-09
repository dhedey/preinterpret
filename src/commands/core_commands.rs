use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct SetCommand {
    variable: Variable,
    #[allow(unused)]
    equals: Punct,
    arguments: InterpretationStream,
}

impl CommandDefinition for SetCommand {
    const COMMAND_NAME: &'static str = "set";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::EmptyStream;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        static ERROR: &str = "Expected [!set! #variable = ... ]";
        Ok(Self {
            variable: arguments.next_as_variable(ERROR)?,
            equals: arguments.next_as_punct_matching('=', ERROR)?,
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for SetCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let result_tokens = self.arguments.interpret_as_tokens(interpreter)?;
        self.variable.set(interpreter, result_tokens);

        Ok(InterpretedStream::new())
    }
}

#[derive(Clone)]
pub(crate) struct RawCommand {
    token_stream: TokenStream,
}

impl CommandDefinition for RawCommand {
    const COMMAND_NAME: &'static str = "raw";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::AppendStream;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self {
            token_stream: arguments.read_all_as_raw_token_stream(),
        })
    }
}

impl CommandInvocation for RawCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        Ok(InterpretedStream::raw(self.token_stream))
    }
}

#[derive(Clone)]
pub(crate) struct IgnoreCommand;

impl CommandDefinition for IgnoreCommand {
    const COMMAND_NAME: &'static str = "ignore";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::EmptyStream;

    fn parse(_arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self)
    }
}

impl CommandInvocation for IgnoreCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        Ok(InterpretedStream::new())
    }
}
