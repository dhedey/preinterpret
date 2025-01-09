use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EmptyCommand;

impl CommandDefinition for EmptyCommand {
    const COMMAND_NAME: &'static str = "empty";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::EmptyStream;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        arguments.assert_end("The !empty! command does not take any arguments")?;
        Ok(Self)
    }
}

impl CommandInvocation for EmptyCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        Ok(InterpretedStream::new())
    }
}

#[derive(Clone)]
pub(crate) struct IsEmptyCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::SingleToken;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for IsEmptyCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
        Ok(TokenTree::bool(interpreted.is_empty(), output_span).into())
    }
}

#[derive(Clone)]
pub(crate) struct LengthCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for LengthCommand {
    const COMMAND_NAME: &'static str = "length";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::SingleToken;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for LengthCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
        let stream_length = interpreted.into_token_stream().into_iter().count();
        let length_literal = Literal::usize_unsuffixed(stream_length).with_span(output_span);
        Ok(InterpretedStream::of_literal(length_literal))
    }
}

#[derive(Clone)]
pub(crate) struct GroupCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for GroupCommand {
    const COMMAND_NAME: &'static str = "group";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::AppendStream;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for GroupCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let mut output = InterpretedStream::new();
        let span_range = self.arguments.span_range();
        output.push_new_group(
            self.arguments.interpret_as_tokens(interpreter)?,
            Delimiter::None,
            span_range,
        );
        Ok(output)
    }
}
