use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EmptyCommand;

impl CommandDefinition for EmptyCommand {
    const COMMAND_NAME: &'static str = "empty";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.assert_empty("The !empty! command does not take any arguments")?;
        Ok(Self)
    }
}

impl CommandInvocation for EmptyCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<CommandOutput> {
        Ok(CommandOutput::Empty)
    }
}

#[derive(Clone)]
pub(crate) struct IsEmptyCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for IsEmptyCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
        Ok(CommandOutput::Ident(Ident::new_bool(
            interpreted.is_empty(),
            output_span,
        )))
    }
}

#[derive(Clone)]
pub(crate) struct LengthCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for LengthCommand {
    const COMMAND_NAME: &'static str = "length";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for LengthCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
        let stream_length = interpreted.into_token_stream().into_iter().count();
        let length_literal = Literal::usize_unsuffixed(stream_length).with_span(output_span);
        Ok(CommandOutput::Literal(length_literal))
    }
}

#[derive(Clone)]
pub(crate) struct GroupCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for GroupCommand {
    const COMMAND_NAME: &'static str = "group";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for GroupCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let output = self.arguments.interpret_as_tokens(interpreter)?;
        Ok(CommandOutput::GroupedStream(output))
    }
}
