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
        Ok(CommandOutput::AppendStream(InterpretedStream::raw(
            self.arguments_span_range,
            self.token_stream,
        )))
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
            error_span_stream: arguments
                .next_as_kinded_group(Delimiter::Bracket, ERROR)?
                .into_inner_stream(),
        };
        arguments.assert_end(ERROR)?;
        Ok(parsed)
    }
}

impl CommandInvocation for ErrorCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        static ERROR: &str = "Expected a single string literal as the error message";

        let message_span_range = self.message.span_range();
        let message = self
            .message
            .interpret_as_tokens(interpreter)?
            .flatten_transparent_groups()
            .into_singleton(ERROR)?
            .to_literal(ERROR)?
            .content_if_string()
            .ok_or_else(|| message_span_range.error(ERROR))?;

        // Consider the case where preinterpret embeds in a declarative macro, and we have
        // an error like this:
        // [!error! [!string! "Expected 100, got " $input] [$input]]
        //
        // In cases like this, rustc wraps $input in a transparent group, which means that
        // the span of that group is the span of the tokens "$input" in the definition of the
        // declarative macro. This is not what we want. We want the span of the tokens which
        // were fed into $input in the declarative macro.
        //
        // The simplest solution here is to get rid of all transparent groups, to get back to the
        // source spans.
        //
        // Once this workstream with macro diagnostics is stabilised:
        // https://github.com/rust-lang/rust/issues/54140#issuecomment-802701867
        //
        // Then we can revisit this and do something better, and include all spans as separate spans
        // in the error message, which will allow a user to trace an error through N different layers
        // of macros.
        //
        // (Possibly we can try to join spans together, and if they don't join, they become separate
        // spans which get printed to the error message).
        //
        // Coincidentally, rust analyzer currently does not properly support
        // transparent groups (as of Jan 2025), so gets it right without this flattening:
        // https://github.com/rust-lang/rust-analyzer/issues/18211
        let error_span_stream = self
            .error_span_stream
            .interpret_as_tokens(interpreter)?
            .flatten_transparent_groups();

        if error_span_stream.is_empty() {
            Span::call_site().err(message)
        } else {
            error_span_stream
                .into_token_stream()
                .span_range()
                .err(message)
        }
    }
}
