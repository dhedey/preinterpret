use crate::internal_prelude::*;

/// For use as a stream input to a command, when the whole command isn't the stream.
///
/// It accepts any of the following:
/// * A `[..]` group - the input stream is the interpreted contents of the brackets
/// * A [!command! ...] - the input stream is the command's output
/// * A $macro_variable or other transparent group - the input stream is the *raw* contents of the group
/// * A `#variable` - the input stream is the content of the variable
/// * A flattened `#..variable` - the variable must contain a single `[..]` or transparent group, the input stream are the group contents
#[derive(Clone)]
pub(crate) enum CommandStreamInput {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    Bracketed {
        delim_span: DelimSpan,
        inner: InterpretationStream,
    },
    Raw {
        delim_span: DelimSpan,
        inner: TokenStream,
    },
}

impl Parse for CommandStreamInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        if let Ok(command) = fork.parse() {
            input.advance_to(&fork);
            return Ok(CommandStreamInput::Command(command));
        }
        let fork = input.fork();
        if let Ok(command) = fork.parse() {
            input.advance_to(&fork);
            return Ok(CommandStreamInput::GroupedVariable(command));
        }
        let fork = input.fork();
        if let Ok(command) = fork.parse() {
            input.advance_to(&fork);
            return Ok(CommandStreamInput::FlattenedVariable(command));
        }
        let error_span = input.span();
        match input.parse_any_delimiter() {
            Ok((Delimiter::Bracket, delim_span, content)) => Ok(CommandStreamInput::Bracketed {
                delim_span,
                inner: content.parse_with(delim_span.span_range())?,
            }),
            Ok((Delimiter::None, delim_span, content)) => Ok(CommandStreamInput::Raw {
                delim_span,
                inner: content.parse()?,
            }),
            _ => error_span
                .err("expected [ ..<input>.. ] or a [!command! ..], #variable, or #macro_variable"),
        }
    }
}

impl HasSpanRange for CommandStreamInput {
    fn span_range(&self) -> SpanRange {
        match self {
            CommandStreamInput::Command(command) => command.span_range(),
            CommandStreamInput::GroupedVariable(variable) => variable.span_range(),
            CommandStreamInput::FlattenedVariable(variable) => variable.span_range(),
            CommandStreamInput::Bracketed {
                delim_span: span, ..
            } => span.span_range(),
            CommandStreamInput::Raw {
                delim_span: span, ..
            } => span.span_range(),
        }
    }
}

impl Interpret for CommandStreamInput {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match self {
            CommandStreamInput::Command(command) => {
                command.interpret_as_tokens_into(interpreter, output)
            }
            CommandStreamInput::FlattenedVariable(variable) => {
                let tokens = variable.interpret_as_new_stream(interpreter)?
                    .syn_parse(|input: ParseStream| -> Result<TokenStream> {
                        let (delimiter, _, content) = input.parse_any_delimiter()?;
                        match delimiter {
                            Delimiter::Bracket | Delimiter::None if input.is_empty() => {
                                content.parse()
                            },
                            _ => {
                                variable.err(format!(
                                    "expected variable to contain a single [ .. ] or transparent group. Perhaps you want to use {} instead, to use the content of the variable as the stream.",
                                    variable.display_grouped_variable_token(),
                                ))
                            },
                        }
                    })?;

                output.extend_raw(tokens);
                Ok(())
            }
            CommandStreamInput::GroupedVariable(variable) => {
                variable.interpret_as_tokens_into(interpreter, output)
            }
            CommandStreamInput::Bracketed { inner, .. } => {
                inner.interpret_as_tokens_into(interpreter, output)
            }
            CommandStreamInput::Raw { inner, .. } => {
                output.extend_raw(inner);
                Ok(())
            }
        }
    }
}
