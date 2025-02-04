use crate::internal_prelude::*;

/// For use as a stream input to a command, when the whole command isn't the stream.
///
/// It accepts any of the following:
/// * A `[..]` group - the input stream is the interpreted contents of the brackets
/// * A [!command! ...] - the input stream is the command's output
/// * A `#variable` - the input stream is the content of the variable
/// * A flattened `#..variable` - the variable must contain a single `[..]` or transparent group, the input stream are the group contents
///
/// We don't support transparent groups, because they are not used consistently and it would expose this
/// inconsistency to the user, and be a potentially breaking change for other tooling to add/remove them.
/// For example, as of Jan 2025, in declarative macro substitutions a $literal gets a wrapping group,
/// but a $tt or $($tt)* does not.
#[derive(Clone)]
pub(crate) enum CommandStreamInput {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    Code(SourceCodeBlock),
    ExplicitStream(SourceGroup),
}

impl Parse<Source> for CommandStreamInput {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            GrammarPeekMatch::Command(_) => Self::Command(input.parse()?),
            GrammarPeekMatch::GroupedVariable => Self::GroupedVariable(input.parse()?),
            GrammarPeekMatch::FlattenedVariable => Self::FlattenedVariable(input.parse()?),
            GrammarPeekMatch::Group(Delimiter::Bracket) => Self::ExplicitStream(input.parse()?),
            GrammarPeekMatch::Group(Delimiter::Brace) => Self::Code(input.parse()?),
            _ => input.span()
                .parse_err("Expected [ ..input stream.. ], { [..input stream..] } or a [!command! ..], #variable or #..variable.\nMacro substitutions such as $x should be placed inside square brackets.")?,
        })
    }
}

impl HasSpanRange for CommandStreamInput {
    fn span_range(&self) -> SpanRange {
        match self {
            CommandStreamInput::Command(command) => command.span_range(),
            CommandStreamInput::GroupedVariable(variable) => variable.span_range(),
            CommandStreamInput::FlattenedVariable(variable) => variable.span_range(),
            CommandStreamInput::Code(code) => code.span_range(),
            CommandStreamInput::ExplicitStream(group) => group.span_range(),
        }
    }
}

impl Interpret for CommandStreamInput {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            CommandStreamInput::Command(mut command) => {
                match command.output_kind() {
                    CommandOutputKind::None
                    | CommandOutputKind::Value
                    | CommandOutputKind::Ident => {
                        command.execution_err("The command does not output a stream")
                    }
                    CommandOutputKind::FlattenedStream => parse_as_stream_input(
                        command,
                        interpreter,
                        || {
                            "Expected output of flattened command to contain a single [...] group or transparent group from a #variable or stream-output command such as [!group! ...]. Perhaps you want to remove the .., to use the command output as-is.".to_string()
                        },
                        output,
                    ),
                    CommandOutputKind::GroupedStream => {
                        unsafe {
                            // SAFETY: The kind change GroupedStream <=> FlattenedStream is valid
                            command.set_output_kind(CommandOutputKind::FlattenedStream);
                        }
                        command.interpret_into(interpreter, output)
                    }
                    CommandOutputKind::ControlFlowCodeStream => parse_as_stream_input(
                        command,
                        interpreter,
                        || {
                            "Expected output of control flow command to contain a single [...] group or transparent group from a #variable or stream-output command such as [!group! ...].".to_string()
                        },
                        output,
                    ),
                }
            }
            CommandStreamInput::FlattenedVariable(variable) => parse_as_stream_input(
                &variable,
                interpreter,
                || {
                    format!(
                        "Expected variable to contain a single [...] group or transparent group from a #variable or stream-output command such as [!group! ...]. Perhaps you want to use {} instead, to use the content of the variable as the stream.",
                        variable.display_grouped_variable_token(),
                    )
                },
                output,
            ),
            CommandStreamInput::GroupedVariable(variable) => {
                variable.substitute_ungrouped_contents_into(interpreter, output)
            }
            CommandStreamInput::Code(code) => parse_as_stream_input(
                code,
                interpreter,
                || {
                    "Expected the { ... } block to output a single [...] group or transparent group from a #variable or stream-output command such as [!group! ...]. You may wish to replace the outer `{ ... }` block with a `[ ... ]` block, which outputs all its contents as a stream.".to_string()
                },
                output,
            ),
            CommandStreamInput::ExplicitStream(group) => {
                group.into_content().interpret_into(interpreter, output)
            }
        }
    }
}

/// From a code neatness perspective, this would be better as `OutputCommandStream`...
/// However this approach avoids a round-trip to TokenStream, which causes bugs in RustAnalyzer.
/// This can be removed when we fork syn and can parse the OutputStream directly.
fn parse_as_stream_input(
    input: impl Interpret + HasSpanRange,
    interpreter: &mut Interpreter,
    error_message: impl FnOnce() -> String,
    output: &mut OutputStream,
) -> ExecutionResult<()> {
    let span = input.span_range();
    input
        .interpret_to_new_stream(interpreter)?
        .unwrap_singleton_group(
            |delimiter| matches!(delimiter, Delimiter::Bracket | Delimiter::None),
            || span.error(error_message()),
        )?
        .append_into(output);
    Ok(())
}

impl InterpretValue for CommandStreamInput {
    type OutputValue = OutputCommandStream;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        Ok(OutputCommandStream {
            stream: self.interpret_to_new_stream(interpreter)?,
        })
    }
}

pub(crate) struct OutputCommandStream {
    pub(crate) stream: OutputStream,
}

impl Parse<Output> for OutputCommandStream {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        // We replicate parse_as_stream_input
        let (_, inner) = input.parse_group_matching(
            |delimiter| matches!(delimiter, Delimiter::Bracket | Delimiter::None),
            || "Expected [...] or a transparent group from a #variable or stream-output command such as [!group! ...]".to_string(),
        )?;
        Ok(Self {
            stream: OutputStream::raw(inner.parse()?),
        })
    }
}
