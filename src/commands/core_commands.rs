use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct SetCommand {
    variable: Variable,
    #[allow(unused)]
    equals: Token![=],
    arguments: InterpretationStream,
}

impl CommandDefinition for SetCommand {
    const COMMAND_NAME: &'static str = "set";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: input.parse()?,
                    equals: input.parse()?,
                    arguments: input.parse_with(arguments.full_span_range())?,
                })
            },
            "Expected [!set! #variable = ... ]",
        )
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

    fn parse(arguments: CommandArguments) -> Result<Self> {
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

    fn parse(arguments: CommandArguments) -> Result<Self> {
        // Avoid a syn parse error by reading all the tokens
        let _ = arguments.read_all_as_raw_token_stream();
        Ok(Self)
    }
}

impl CommandInvocation for IgnoreCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<CommandOutput> {
        Ok(CommandOutput::Empty)
    }
}

/// This serves as a no-op command to take a stream when the grammar only allows a single item
#[derive(Clone)]
pub(crate) struct StreamCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for StreamCommand {
    const COMMAND_NAME: &'static str = "stream";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for StreamCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        Ok(CommandOutput::AppendStream(
            self.arguments.interpret_as_tokens(interpreter)?,
        ))
    }
}

#[derive(Clone)]
pub(crate) struct ErrorCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for ErrorCommand {
    const COMMAND_NAME: &'static str = "error";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

#[derive(Default)]
struct ErrorCommandArguments {
    message: Option<syn::LitStr>,
    error_spans: Option<BracketedTokenStream>,
}

impl CommandInvocation for ErrorCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let fields_parser = FieldsParseDefinition::new(ErrorCommandArguments::default())
            .add_required_field(
                "message",
                "\"Error message to display\"",
                None,
                |params, val| params.message = Some(val),
            )
            .add_optional_field(
                "spans",
                "[$abc]",
                Some("An optional [token stream], to determine where to show the error message"),
                |params, val| params.error_spans = Some(val),
            );

        let arguments = self
            .arguments
            .interpret_as_tokens(interpreter)?
            .parse_into_fields(fields_parser)?;

        let message = arguments
            .message
            .unwrap() // Field was required
            .value();

        let error_span_stream = arguments
            .error_spans
            .map(|b| b.token_stream)
            .unwrap_or_default();

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
        let error_span_stream = error_span_stream.flatten_transparent_groups();

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
