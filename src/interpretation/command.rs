use crate::internal_prelude::*;

pub(crate) enum CommandOutputBehaviour {
    EmptyStream,
    #[allow(unused)] // Likely useful in future
    GroupedStream,
    AppendStream,
    SingleToken,
}

pub(crate) trait CommandDefinition: CommandInvocation + Clone {
    const COMMAND_NAME: &'static str;
    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour;

    fn parse(arguments: InterpreterParseStream) -> Result<Self>;
}

pub(crate) trait CommandInvocation {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream>;
}

pub(crate) trait ClonableCommandInvocation: CommandInvocation {
    fn clone_box(&self) -> Box<dyn ClonableCommandInvocation>;
}

impl<C: CommandDefinition + CommandInvocation + 'static> ClonableCommandInvocation for C {
    fn clone_box(&self) -> Box<dyn ClonableCommandInvocation> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ClonableCommandInvocation> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

macro_rules! define_command_kind {
    (
        $(
            $command:ident,
        )*
    ) => {
        #[allow(clippy::enum_variant_names)]
        #[derive(Clone, Copy)]
        pub(crate) enum CommandKind {
            $(
                $command,
            )*
        }

        impl CommandKind {
            pub(crate) fn parse_invocation(&self, arguments: InterpreterParseStream) -> Result<Box<dyn ClonableCommandInvocation>> {
                Ok(match self {
                    $(
                        Self::$command => Box::new(
                            <$command as CommandDefinition>::parse(arguments)?
                        ),
                    )*
                })
            }

            pub(crate) fn output_behaviour(&self) -> CommandOutputBehaviour {
                match self {
                    $(
                        Self::$command => <$command as CommandDefinition>::OUTPUT_BEHAVIOUR,
                    )*
                }
            }

            pub(crate) fn for_ident(ident: &Ident) -> Option<Self> {
                Some(match ident.to_string().as_ref() {
                    $(
                        <$command as CommandDefinition>::COMMAND_NAME => Self::$command,
                    )*
                    _ => return None,
                })
            }

            const ALL_KIND_NAMES: &'static [&'static str] = &[$(<$command as CommandDefinition>::COMMAND_NAME,)*];

            pub(crate) fn list_all() -> String {
                // TODO improve to add an "and" at the end
                Self::ALL_KIND_NAMES.join(", ")
            }
        }
    };
}
pub(crate) use define_command_kind;

#[derive(Clone)]
pub(crate) struct Command {
    command_kind: CommandKind,
    source_group_span_range: SpanRange,
    invocation: Box<dyn ClonableCommandInvocation>,
}

impl Command {
    pub(super) fn attempt_parse_from_group(group: &Group) -> Result<Option<Self>> {
        fn matches_command_start(group: &Group) -> Option<(Ident, InterpreterParseStream)> {
            if group.delimiter() != Delimiter::Bracket {
                return None;
            }
            let mut tokens = InterpreterParseStream::new(group.stream(), group.span_range());
            tokens.next_as_punct_matching('!', "").ok()?;
            let ident = tokens.next_as_ident("").ok()?;
            Some((ident, tokens))
        }

        fn extract_command_data(
            command_ident: &Ident,
            parse_stream: &mut InterpreterParseStream,
        ) -> Option<CommandKind> {
            let command_kind = CommandKind::for_ident(command_ident)?;
            parse_stream.next_as_punct_matching('!', "").ok()?;
            Some(command_kind)
        }

        // Attempt to match `[!ident`, if that doesn't match, we assume it's not a command invocation,
        // so return `Ok(None)`
        let (command_ident, mut parse_stream) = match matches_command_start(group) {
            Some(command_start) => command_start,
            None => return Ok(None),
        };

        // We have now checked enough that we're confident the user is pretty intentionally using
        // the call convention. Any issues we hit from this point will be a helpful compiler error.
        match extract_command_data(&command_ident, &mut parse_stream) {
            Some(command_kind) => {
                let invocation = command_kind.parse_invocation( parse_stream)?;
                Ok(Some(Self {
                    command_kind,
                    source_group_span_range: group.span_range(),
                    invocation,
                }))
            },
            None => Err(command_ident.span().error(
                format!(
                    "Expected `[!<command>! ..]`, for <command> one of: {}.\nIf this wasn't intended to be a preinterpret command, you can work around this with [!raw! [!{} ... ]]",
                    CommandKind::list_all(),
                    command_ident,
                ),
            )),
        }
    }

    fn execute_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let substitution = self.invocation.execute(interpreter)?;
        match self.command_kind.output_behaviour() {
            CommandOutputBehaviour::GroupedStream => {
                output.push_new_group(substitution, Delimiter::None, self.source_group_span_range);
            }
            CommandOutputBehaviour::EmptyStream
            | CommandOutputBehaviour::SingleToken
            | CommandOutputBehaviour::AppendStream => {
                output.extend(substitution);
            }
        }
        Ok(())
    }
}

impl HasSpanRange for Command {
    fn span_range(&self) -> SpanRange {
        self.source_group_span_range
    }
}

impl Interpret for Command {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.execute_into(interpreter, output)
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        let span_range = self.span_range();
        expression_stream
            .push_interpreted_group(self.interpret_as_tokens(interpreter)?, span_range);
        Ok(())
    }
}
