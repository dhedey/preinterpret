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
    arguments: ErrorArguments,
}

impl CommandDefinition for ErrorCommand {
    const COMMAND_NAME: &'static str = "error";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.fully_parse_as()?,
        })
    }
}

#[derive(Clone)]
struct ErrorArguments {
    message: InterpretationValue<syn::LitStr>,
    spans: Option<CommandStreamInput>,
}

impl ArgumentsContent for ErrorArguments {
    fn error_message() -> String {
        r#"Expected: {
    // The error message to display
    message: "...",
    // An optional [token stream], to determine where to show the error message
    spans?: [$abc],
}"#
        .to_string()
    }
}

impl Parse for ErrorArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut message = None;
        let mut spans = None;

        let content;
        let brace = syn::braced!(content in input);
        while !content.is_empty() {
            let ident: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            match ident.to_string().as_str() {
                "message" => {
                    if message.is_some() {
                        return ident.err("duplicate field");
                    }
                    message = Some(content.parse()?);
                }
                "spans" => {
                    if spans.is_some() {
                        return ident.err("duplicate field");
                    }
                    spans = Some(content.parse()?);
                }
                _ => return ident.err("unexpected field"),
            }
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }
        let mut missing_fields: Vec<String> = vec![];

        if message.is_none() {
            missing_fields.push("message".to_string());
        }

        if !missing_fields.is_empty() {
            return brace.span.err(format!(
                "required fields are missing: {}",
                missing_fields.join(", ")
            ));
        }

        Ok(Self {
            message: message.unwrap(),
            spans,
        })
    }
}

impl CommandInvocation for ErrorCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let message = self.arguments.message.interpret(interpreter)?.value();

        let error_span = match self.arguments.spans {
            Some(spans) => {
                let error_span_stream = spans.interpret_as_tokens(interpreter)?;

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

                let error_span_stream = error_span_stream
                    .into_token_stream()
                    .flatten_transparent_groups();
                if error_span_stream.is_empty() {
                    Span::call_site().span_range()
                } else {
                    error_span_stream.span_range()
                }
            }
            None => Span::call_site().span_range(),
        };

        error_span.err(message)
    }
}
