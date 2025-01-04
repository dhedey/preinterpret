use crate::internal_prelude::*;

pub(crate) trait CommandDefinition {
    const COMMAND_NAME: &'static str;

    fn execute(interpreter: &mut Interpreter, command: Command) -> Result<InterpretedStream>;
}

macro_rules! define_commands {
    (
        pub(crate) enum $enum_name:ident {
            $(
                $command:ident,
            )*
        }
    ) => {
        #[allow(clippy::enum_variant_names)]
        #[derive(Clone, Copy)]
        pub(crate) enum $enum_name {
            $(
                $command,
            )*
        }

        impl $enum_name {
            pub(crate) fn execute(self, interpreter: &mut Interpreter, command: Command) -> Result<InterpretedStream> {
                match self {
                    $(
                        Self::$command => $command::execute(interpreter, command),
                    )*
                }
            }

            pub(crate) fn attempt_parse(ident: &Ident) -> Option<Self> {
                Some(match ident.to_string().as_ref() {
                    $(
                        <$command as CommandDefinition>::COMMAND_NAME => Self::$command,
                    )*
                    _ => return None,
                })
            }

            const ALL_KIND_NAMES: &'static [&'static str] = &[$($command::COMMAND_NAME,)*];

            pub(crate) fn list_all() -> String {
                // TODO improve to add an "and" at the end
                Self::ALL_KIND_NAMES.join(", ")
            }
        }
    };
}
pub(crate) use define_commands;

#[derive(Clone)]
pub(crate) struct CommandInvocation {
    command_kind: CommandKind,
    command: Command,
}

impl CommandInvocation {
    pub(crate) fn new(
        command_ident: Ident,
        command_kind: CommandKind,
        group: &Group,
        argument_tokens: Tokens,
    ) -> Self {
        Self {
            command_kind,
            command: Command::new(command_ident, group.span(), argument_tokens),
        }
    }

    fn execute_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let substitution = self.command_kind.execute(interpreter, self.command)?;
        output.extend(substitution);
        Ok(())
    }
}

impl HasSpanRange for CommandInvocation {
    fn span_range(&self) -> SpanRange {
        self.command.span_range()
    }
}

impl Interpret for CommandInvocation {
    fn interpret_as_tokens_into(self, interpreter: &mut Interpreter, output: &mut InterpretedStream) -> Result<()> {
        self.execute_into(interpreter, output)
    }

    fn interpret_as_expression_into(self, interpreter: &mut Interpreter, expression_stream: &mut ExpressionStream) -> Result<()> {
        let span_range = self.span_range();
        expression_stream.push_interpreted_group(
            self.interpret_as_tokens(interpreter)?,
            span_range,
        );
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct Command {
    command_ident: Ident,
    command_span: Span,
    argument_tokens: Tokens,
}

impl Command {
    fn new(command_ident: Ident, command_span: Span, argument_tokens: Tokens) -> Self {
        Self {
            command_ident,
            command_span,
            argument_tokens,
        }
    }

    #[allow(unused)] // Likely useful in future
    pub(crate) fn ident_span(&self) -> Span {
        self.command_ident.span()
    }

    pub(crate) fn span(&self) -> Span {
        self.command_span
    }

    pub(crate) fn error(&self, message: impl core::fmt::Display) -> syn::Error {
        self.command_span.error(message)
    }

    pub(crate) fn err<T>(&self, message: impl core::fmt::Display) -> Result<T> {
        Err(self.error(message))
    }

    pub(crate) fn arguments(&mut self) -> &mut Tokens {
        &mut self.argument_tokens
    }
}

impl HasSpanRange for Command {
    fn span_range(&self) -> SpanRange {
        self.span().span_range()
    }
}
