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
    Code(CommandCodeInput),
    ExplicitStream(InterpretationGroup),
}

impl Parse for CommandStreamInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(match detect_preinterpret_grammar(input.cursor()) {
            PeekMatch::GroupedCommand => Self::Command(input.parse()?),
            PeekMatch::FlattenedCommand => Self::Command(input.parse()?),
            PeekMatch::GroupedVariable => Self::GroupedVariable(input.parse()?),
            PeekMatch::FlattenedVariable => Self::FlattenedVariable(input.parse()?),
            PeekMatch::InterpretationGroup(Delimiter::Bracket) => Self::ExplicitStream(input.parse()?),
            PeekMatch::InterpretationGroup(Delimiter::Brace) => Self::Code(input.parse()?),
            PeekMatch::InterpretationGroup(_) | PeekMatch::Other => input.span()
                .err("Expected [ ..input stream.. ], { [..input stream..] } or a [!command! ..], #variable or #..variable.\nMacro substitutions such as $x should be placed inside square brackets.")?,
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
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match self {
            CommandStreamInput::Command(mut command) => {
                match command.output_kind() {
                    CommandOutputKind::None
                    | CommandOutputKind::Value
                    | CommandOutputKind::Ident => {
                        command.err("The command does not output a stream")
                    }
                    CommandOutputKind::FlattenedStream => {
                        let span = command.span();
                        let tokens = parse_as_stream_input(
                            command.interpret_as_tokens(interpreter)?,
                            || {
                                span.error("Expected output of flattened command to contain a single [ ... ] or transparent group. Perhaps you want to remove the .., to use the command output as-is.")
                            },
                        )?;
                        output.extend_raw_tokens(tokens);
                        Ok(())
                    }
                    CommandOutputKind::GroupedStream => {
                        unsafe {
                            // SAFETY: The kind change GroupedStream <=> FlattenedStream is valid
                            command.set_output_kind(CommandOutputKind::FlattenedStream);
                        }
                        command.interpret_as_tokens_into(interpreter, output)
                    }
                    CommandOutputKind::ControlFlowCodeStream => {
                        let span = command.span();
                        let tokens = parse_as_stream_input(
                            command.interpret_as_tokens(interpreter)?,
                            || {
                                span.error("Expected output of control flow command to contain a single [ ... ] or transparent group.")
                            },
                        )?;
                        output.extend_raw_tokens(tokens);
                        Ok(())
                    }
                }
            }
            CommandStreamInput::FlattenedVariable(variable) => {
                let tokens = parse_as_stream_input(
                    variable.interpret_as_tokens(interpreter)?,
                    || {
                        variable.error(format!(
                        "Expected variable to contain a single [ ... ] or transparent group. Perhaps you want to use {} instead, to use the content of the variable as the stream.",
                        variable.display_grouped_variable_token(),
                    ))
                    },
                )?;
                output.extend_raw_tokens(tokens);
                Ok(())
            }
            CommandStreamInput::GroupedVariable(variable) => {
                let group_contents = variable.interpret_ungrouped_contents(interpreter)?;
                output.extend(group_contents);
                Ok(())
            }
            CommandStreamInput::Code(code) => {
                let span = code.span();
                let tokens = parse_as_stream_input(code.interpret_as_tokens(interpreter)?, || {
                    span.error("Expected the { ... } block to output a single [ ... ] group or transparent group. You may wish to replace the outer `{ ... }` block with a `[ ... ]` block, which outputs all its contents as a stream.".to_string())
                })?;
                output.extend_raw_tokens(tokens);
                Ok(())
            }
            CommandStreamInput::ExplicitStream(group) => group
                .into_content()
                .interpret_as_tokens_into(interpreter, output),
        }
    }
}

fn parse_as_stream_input(
    interpreted: InterpretedStream,
    on_error: impl FnOnce() -> Error,
) -> Result<TokenStream> {
    fn get_group(interpreted: InterpretedStream) -> Option<Group> {
        let mut token_iter = interpreted.into_token_stream().into_iter();
        let group = match token_iter.next()? {
            TokenTree::Group(group)
                if matches!(group.delimiter(), Delimiter::Bracket | Delimiter::None) =>
            {
                Some(group)
            }
            _ => return None,
        };
        if token_iter.next().is_some() {
            return None;
        }
        group
    }
    let group = get_group(interpreted).ok_or_else(on_error)?;
    Ok(group.stream())
}
