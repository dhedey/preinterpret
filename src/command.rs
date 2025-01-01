use crate::internal_prelude::*;

pub(crate) trait CommandDefinition {
    const COMMAND_NAME: &'static str;

    fn execute(interpreter: &mut Interpreter, argument: Command) -> Result<TokenStream>;
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
        pub(crate) enum $enum_name {
            $(
                $command,
            )*
        }

        impl $enum_name {
            pub(crate) fn execute(self, interpreter: &mut Interpreter, command: Command) -> Result<TokenStream> {
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

    pub(crate) fn execute(self, interpreter: &mut Interpreter) -> Result<TokenStream> {
        self.command_kind.execute(interpreter, self.command)
    }
}

impl HasSpanRange for CommandInvocation {
    fn span_range(&self) -> SpanRange {
        self.command.span_range()
    }
}

pub(crate) struct Variable {
    marker: Punct, // #
    variable_name: Ident,
}

impl Variable {
    pub(crate) fn new(marker: Punct, variable_name: Ident) -> Self {
        Self {
            marker,
            variable_name,
        }
    }

    pub(crate) fn variable_name(&self) -> &Ident {
        &self.variable_name
    }

    pub(crate) fn execute_substitution(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<TokenStream> {
        let Variable { variable_name, .. } = self;
        match interpreter.get_variable(&variable_name.to_string()) {
            Some(variable_value) => Ok(variable_value.clone()),
            None => {
                self.span_range().err(
                    format!(
                        "The variable {} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {}]",
                        self,
                        self,
                    ),
                )
            }
        }
    }
}

impl HasSpanRange for Variable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span(), self.variable_name.span())
    }
}

impl core::fmt::Display for Variable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.marker.as_char(), self.variable_name)
    }
}

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

    pub(crate) fn err(&self, message: impl core::fmt::Display) -> Result<TokenStream> {
        Err(self.error(message))
    }

    /// Expects the remaining arguments to be non-empty
    pub(crate) fn interpret_remaining_arguments(
        &mut self,
        interpreter: &mut Interpreter,
        substitution_mode: SubstitutionMode,
    ) -> Result<TokenStream> {
        if self.argument_tokens.is_empty() {
            // This is simply for clarity / to make empty arguments explicit.
            return self.err(
                "Arguments were empty. Use [!empty!] if you want to use an empty token stream.",
            );
        }
        interpreter.interpret_tokens(&mut self.argument_tokens, substitution_mode)
    }

    pub(crate) fn argument_tokens(&mut self) -> &mut Tokens {
        &mut self.argument_tokens
    }

    pub(crate) fn into_argument_tokens(self) -> Tokens {
        self.argument_tokens
    }
}

impl HasSpanRange for Command {
    fn span_range(&self) -> SpanRange {
        self.span().span_range()
    }
}
