use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct SetCommand {
    arguments: SetArguments,
}

#[allow(unused)]
#[derive(Clone)]
enum SetArguments {
    SetVariable {
        variable: GroupedVariable,
        equals: Token![=],
        content: SourceStream,
    },
    ExtendVariable {
        variable: GroupedVariable,
        plus_equals: Token![+=],
        content: SourceStream,
    },
    SetVariablesEmpty {
        variables: Vec<GroupedVariable>,
    },
    Discard {
        discard: Token![_],
        equals: Token![=],
        content: SourceStream,
    },
}

impl CommandType for SetCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for SetCommand {
    const COMMAND_NAME: &'static str = "set";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                if input.peek(Token![_]) {
                    return Ok(SetCommand {
                        arguments: SetArguments::Discard {
                            discard: input.parse()?,
                            equals: input.parse()?,
                            content: input.parse_with_context(arguments.command_span())?,
                        },
                    });
                }
                let variable = input.parse()?;
                if input.peek(Token![+=]) {
                    return Ok(SetCommand {
                        arguments: SetArguments::ExtendVariable {
                            variable,
                            plus_equals: input.parse()?,
                            content: input.parse_with_context(arguments.command_span())?,
                        },
                    })
                }
                if input.peek(Token![=]) {
                    return Ok(SetCommand {
                        arguments: SetArguments::SetVariable {
                            variable,
                            equals: input.parse()?,
                            content: input.parse_with_context(arguments.command_span())?,
                        },
                    })
                }
                let mut variables = vec![variable];
                loop {
                    if !input.is_empty() {
                        input.parse::<Token![,]>()?;
                    }
                    if input.is_empty() {
                        return Ok(SetCommand {
                            arguments: SetArguments::SetVariablesEmpty { variables },
                        });
                    }
                    variables.push(input.parse()?);
                }
            },
            "Expected [!set! #var1 = ...] or [!set! #var1 += ...] or [!set! _ = ...] or [!set! #var1, #var2]",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        match self.arguments {
            SetArguments::SetVariable {
                variable, content, ..
            } => {
                let content = content.interpret_to_new_stream(interpreter)?;
                variable.define(interpreter, content);
            }
            SetArguments::ExtendVariable {
                variable, content, ..
            } => {
                let variable_data = variable.reference(interpreter)?;
                content.interpret_into(
                    interpreter,
                    variable_data.into_mut()?.into_stream()?.as_mut(),
                )?;
            }
            SetArguments::SetVariablesEmpty { variables } => {
                for variable in variables {
                    variable.define(interpreter, OutputStream::new());
                }
            }
            SetArguments::Discard { content, .. } => {
                let _ = content.interpret_to_new_stream(interpreter)?;
            }
        }
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
        self,
        _interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        output.extend_raw_tokens(self.token_stream);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct StreamCommand {
    inner: SourceStream,
}

impl CommandType for StreamCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for StreamCommand {
    const COMMAND_NAME: &'static str = "stream";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inner: arguments.parse_all_as_source()?,
        })
    }

    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.inner.interpret_into(interpreter, output)
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

    fn execute(self, _interpreter: &mut Interpreter) -> ExecutionResult<()> {
        Ok(())
    }
}

/// This is temporary until we have a proper implementation of #(...)
#[derive(Clone)]
pub(crate) struct ReinterpretCommand {
    content: SourceStream,
}

impl CommandType for ReinterpretCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for ReinterpretCommand {
    const COMMAND_NAME: &'static str = "reinterpret";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            content: arguments.parse_all_as_source()?,
        })
    }

    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let command_span = self.content.span();
        let interpreted = self.content.interpret_to_new_stream(interpreter)?;
        let source = unsafe {
            // RUST-ANALYZER-SAFETY - Can't do much better than this
            interpreted.into_token_stream()
        };
        let reparsed_source_stream =
            source.source_parse_with(|input| SourceStream::parse(input, command_span))?;
        reparsed_source_stream.interpret_into(interpreter, output)
    }
}

#[derive(Clone)]
pub(crate) struct SettingsCommand {
    inputs: SourceSettingsInputs,
}

impl CommandType for SettingsCommand {
    type OutputKind = OutputKindNone;
}

define_object_arguments! {
    SourceSettingsInputs => SettingsInputs {
        required: {},
        optional: {
            iteration_limit: DEFAULT_ITERATION_LIMIT_STR ("The new iteration limit"),
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

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let inputs = self.inputs.interpret_to_value(interpreter)?;
        if let Some(limit) = inputs.iteration_limit {
            let limit = limit
                .expect_integer("The iteration limit")?
                .expect_usize()?;
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
    Fields(SourceErrorInputs),
    JustMessage(SourceStream),
}

define_object_arguments! {
    SourceErrorInputs => ErrorInputs {
        required: {
            message: r#""...""# ("The error message to display"),
        },
        optional: {
            spans: "[!stream! $abc]" ("An optional [token stream], to determine where to show the error message"),
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
                            input.parse_with_context(arguments.command_span())?,
                        ),
                    })
                }
            },
            format!(
                "Expected [!error! \"Expected X, found: \" #world] or [!error! {}]",
                SourceErrorInputs::describe_object()
            ),
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let fields = match self.inputs {
            EitherErrorInput::Fields(error_inputs) => {
                error_inputs.interpret_to_value(interpreter)?
            }
            EitherErrorInput::JustMessage(stream) => {
                let error_message = stream
                    .interpret_to_new_stream(interpreter)?
                    .concat_recursive(&ConcatBehaviour::standard());
                return Span::call_site().execution_err(error_message);
            }
        };

        let error_message = fields.message.expect_string("Error message")?.value;

        let error_span = match fields.spans {
            Some(spans) => {
                let error_span_stream = spans.expect_stream("The error spans")?.value;

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
                    error_span_stream.span_range_from_iterating_over_all_tokens()
                }
            }
            None => Span::call_site().span_range(),
        };

        error_span.execution_err(error_message)
    }
}

#[derive(Clone)]
pub(crate) struct DebugCommand {
    span: Span,
    inner: SourceExpression,
}

impl CommandType for DebugCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for DebugCommand {
    const COMMAND_NAME: &'static str = "debug";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    span: arguments.command_span(),
                    inner: input.parse()?,
                })
            },
            "Expected [!debug! <expression>]. To provide a stream, use [!debug! [!stream! ...]]",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let value = self
            .inner
            .interpret_to_value(interpreter)?
            .with_span(self.span)
            .into_debug_string_value()?;
        Ok(value)
    }
}
