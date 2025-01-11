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
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let result_tokens = self.arguments.interpret_as_tokens(interpreter)?;
        self.variable.set(interpreter, result_tokens);

        Ok(CommandOutput::Empty)
    }
}

#[derive(Clone)]
pub(crate) struct RawCommand {
    arguments_span_range: SpanRange,
    token_stream: TokenStream,
}

impl CommandDefinition for RawCommand {
    const COMMAND_NAME: &'static str = "raw";

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self {
            arguments_span_range: arguments.full_span_range(),
            token_stream: arguments.read_all_as_raw_token_stream(),
        })
    }
}

impl CommandInvocation for RawCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<CommandOutput> {
        Ok(CommandOutput::AppendStream(InterpretedStream::raw(self.arguments_span_range, self.token_stream)))
    }
}

#[derive(Clone)]
pub(crate) struct IgnoreCommand;

impl CommandDefinition for IgnoreCommand {
    const COMMAND_NAME: &'static str = "ignore";

    fn parse(_arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self)
    }
}

impl CommandInvocation for IgnoreCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<CommandOutput> {
        Ok(CommandOutput::Empty)
    }
}

#[derive(Clone)]
pub(crate) struct ErrorCommand {
    message: NextItem,
    error_span_stream: InterpretationStream,
}

impl CommandDefinition for ErrorCommand {
    const COMMAND_NAME: &'static str = "error";

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        static ERROR: &str = "Expected [!error! \"Error message\" [tokens spanning error]], for example:\n* [!error! \"Compiler error message\" [$tokens_covering_error]]\n * [!error! [!string! \"My Error\" \"in bits\"] []]";
        let parsed = Self {
            message: arguments.next_item(ERROR)?,
            error_span_stream: arguments.next_as_kinded_group(Delimiter::Bracket, ERROR)?.into_inner_stream(),
        };
        arguments.assert_end(ERROR)?;
        Ok(parsed)
    }
}

impl CommandInvocation for ErrorCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        static ERROR: &str = "Expected a single string literal as the error message";
        let message_span_range = self.message.span_range();
        let message = self.message.interpret_as_tokens(interpreter)?
            .as_singleton(ERROR)?
            .to_literal(ERROR)?
            .content_if_string()
            .ok_or_else(|| message_span_range.error(ERROR))?;
        let error_span_stream = self.error_span_stream.interpret_as_tokens(interpreter)?;

        if error_span_stream.is_empty() {
            return Span::call_site().err(message);
        } else {
            error_span_stream.into_token_stream().span_range().err(message)
        }
    }
}