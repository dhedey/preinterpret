use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct SetCommand {
    variable: GroupedVariable,
    #[allow(unused)]
    equals: Token![=],
    arguments: InterpretationStream,
}

impl CommandType for SetCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for SetCommand {
    const COMMAND_NAME: &'static str = "set";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: input.parse()?,
                    equals: input.parse()?,
                    arguments: input.parse_with(arguments.full_span_range())?,
                })
            },
            "Expected [!set! #variable = ..]",
        )
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let result_tokens = self.arguments.interpret_to_new_stream(interpreter)?;
        self.variable.set(interpreter, result_tokens)?;
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ExtendCommand {
    variable: GroupedVariable,
    #[allow(unused)]
    plus_equals: Token![+=],
    arguments: InterpretationStream,
}

impl CommandType for ExtendCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for ExtendCommand {
    const COMMAND_NAME: &'static str = "extend";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: input.parse()?,
                    plus_equals: input.parse()?,
                    arguments: input.parse_all_for_interpretation(arguments.full_span_range())?,
                })
            },
            "Expected [!extend! #variable += ..]",
        )
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let variable_data = self.variable.get_existing_for_mutation(interpreter)?;
        self.arguments.interpret_into(
            interpreter,
            variable_data.get_mut(&self.variable)?.deref_mut(),
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct RawCommand {
    token_stream: TokenStream,
}

impl CommandType for RawCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for RawCommand {
    const COMMAND_NAME: &'static str = "raw";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            token_stream: arguments.read_all_as_raw_token_stream(),
        })
    }

    fn execute(
        self: Box<Self>,
        _interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        output.extend_raw_tokens(self.token_stream);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct IgnoreCommand;

impl CommandType for IgnoreCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for IgnoreCommand {
    const COMMAND_NAME: &'static str = "ignore";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        // Avoid a syn parse error by reading all the tokens
        let _ = arguments.read_all_as_raw_token_stream();
        Ok(Self)
    }

    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> ExecutionResult<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct VoidCommand {
    inner: InterpretationStream,
}

impl CommandType for VoidCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for VoidCommand {
    const COMMAND_NAME: &'static str = "void";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inner: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let _ = self.inner.interpret_to_new_stream(interpreter)?;
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct SettingsCommand {
    inputs: SettingsInputs,
}

impl CommandType for SettingsCommand {
    type OutputKind = OutputKindNone;
}

define_field_inputs! {
    SettingsInputs {
        required: {},
        optional: {
            iteration_limit: CommandValueInput<syn::LitInt> = DEFAULT_ITERATION_LIMIT ("The new iteration limit"),
        }
    }
}

impl NoOutputCommandDefinition for SettingsCommand {
    const COMMAND_NAME: &'static str = "settings";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inputs: arguments.fully_parse_as()?,
        })
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        if let Some(limit) = self.inputs.iteration_limit {
            let limit: usize = limit.interpret(interpreter)?.base10_parse()?;
            interpreter.set_iteration_limit(Some(limit));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ErrorCommand {
    inputs: EitherErrorInput,
}

impl CommandType for ErrorCommand {
    type OutputKind = OutputKindNone;
}

#[derive(Clone)]
enum EitherErrorInput {
    Fields(ErrorInputs),
    JustMessage(InterpretationStream),
}

define_field_inputs! {
    ErrorInputs {
        required: {
            message: CommandValueInput<syn::LitStr> = r#""...""# ("The error message to display"),
        },
        optional: {
            spans: CommandStreamInput = "[$abc]" ("An optional [token stream], to determine where to show the error message"),
        }
    }
}

impl NoOutputCommandDefinition for ErrorCommand {
    const COMMAND_NAME: &'static str = "error";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                if input.peek(syn::token::Brace) {
                    Ok(Self {
                        inputs: EitherErrorInput::Fields(input.parse()?),
                    })
                } else {
                    Ok(Self {
                        inputs: EitherErrorInput::JustMessage(
                            input.parse_with(arguments.full_span_range())?,
                        ),
                    })
                }
            },
            format!(
                "Expected [!error! \"Expected X, found: \" #world] or [!error! {}]",
                ErrorInputs::fields_description()
            ),
        )
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let fields = match self.inputs {
            EitherErrorInput::Fields(error_inputs) => error_inputs,
            EitherErrorInput::JustMessage(stream) => {
                let error_message = stream
                    .interpret_to_new_stream(interpreter)?
                    .concat_recursive(&ConcatBehaviour::standard());
                return Span::call_site().execution_err(error_message);
            }
        };

        let message = fields.message.interpret(interpreter)?.value();

        let error_span = match fields.spans {
            Some(spans) => {
                let error_span_stream = spans.interpret_to_new_stream(interpreter)?;

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

                let error_span_stream =
                    error_span_stream.into_token_stream_removing_any_transparent_groups();
                if error_span_stream.is_empty() {
                    Span::call_site().span_range()
                } else {
                    error_span_stream.span_range()
                }
            }
            None => Span::call_site().span_range(),
        };

        error_span.execution_err(message)
    }
}

#[derive(Clone)]
pub(crate) struct DebugCommand {
    inner: InterpretationStream,
}

impl CommandType for DebugCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for DebugCommand {
    const COMMAND_NAME: &'static str = "debug";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inner: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<TokenTree> {
        let span = self.inner.span();
        let debug_string = self
            .inner
            .interpret_to_new_stream(interpreter)?
            .concat_recursive(&ConcatBehaviour::debug());
        Ok(Literal::string(&debug_string).with_span(span).into())
    }
}
