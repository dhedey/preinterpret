use crate::internal_prelude::*;

pub(crate) trait CommandDefinition {
    const COMMAND_NAME: &'static str;

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream>;
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
            pub(crate) fn execute(self, interpreter: &mut Interpreter, argument_stream: CommandArgumentStream, command_span: Span) -> Result<TokenStream> {
                match self {
                    $(
                        Self::$command => $command::execute(interpreter, argument_stream, command_span),
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
    argument_stream: CommandArgumentStream,
    command_span: Span,
}

impl CommandInvocation {
    pub(crate) fn new(command_kind: CommandKind, group: &Group, argument_tokens: Tokens) -> Self {
        Self {
            command_kind,
            argument_stream: CommandArgumentStream::new(argument_tokens),
            command_span: group.span(),
        }
    }

    pub(crate) fn execute(self, interpreter: &mut Interpreter) -> Result<TokenStream> {
        self.command_kind
            .execute(interpreter, self.argument_stream, self.command_span)
    }
}

pub(crate) struct VariableSubstitution {
    marker: Punct, // #
    variable_name: Ident,
}

impl VariableSubstitution {
    pub(crate) fn new(marker: Punct, variable_name: Ident) -> Self {
        Self {
            marker,
            variable_name,
        }
    }

    pub(crate) fn execute(self, interpreter: &mut Interpreter) -> Result<TokenStream> {
        let VariableSubstitution {
            marker,
            variable_name,
        } = self;
        match interpreter.get_variable(&variable_name.to_string()) {
            Some(variable_value) => Ok(variable_value.clone()),
            None => {
                let marker = marker.as_char();
                let name_str = variable_name.to_string();
                let name_str = &name_str;
                Err(Error::new(
                    variable_name.span(),
                    format!(
                        "The variable {}{} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {}{}]",
                        marker,
                        name_str,
                        marker,
                        name_str,
                    ),
                ))
            }
        }
    }
}

pub(crate) struct CommandArgumentStream {
    tokens: Tokens,
}

impl CommandArgumentStream {
    fn new(tokens: Tokens) -> Self {
        Self { tokens }
    }

    pub(crate) fn interpret_and_concat_to_string(
        self,
        interpreter: &mut Interpreter,
    ) -> Result<String> {
        let interpreted = interpreter.interpret_tokens(self.tokens)?;
        Ok(concat_recursive(interpreted))
    }

    pub(crate) fn tokens(self) -> Tokens {
        self.tokens
    }
}

fn concat_recursive(arguments: TokenStream) -> String {
    fn concat_recursive_internal(output: &mut String, arguments: TokenStream) {
        for token_tree in arguments {
            match token_tree {
                TokenTree::Literal(literal) => {
                    let lit: Lit = parse_str(&literal.to_string()).expect(
                        "All proc_macro2::Literal values should be decodable as a syn::Lit",
                    );
                    match lit {
                        Lit::Str(lit_str) => output.push_str(&lit_str.value()),
                        Lit::Char(lit_char) => output.push(lit_char.value()),
                        _ => {
                            output.push_str(&literal.to_string());
                        }
                    }
                }
                TokenTree::Group(group) => match group.delimiter() {
                    Delimiter::Parenthesis => {
                        output.push('(');
                        concat_recursive_internal(output, group.stream());
                        output.push(')');
                    }
                    Delimiter::Brace => {
                        output.push('{');
                        concat_recursive_internal(output, group.stream());
                        output.push('}');
                    }
                    Delimiter::Bracket => {
                        output.push('[');
                        concat_recursive_internal(output, group.stream());
                        output.push(']');
                    }
                    Delimiter::None => {
                        concat_recursive_internal(output, group.stream());
                    }
                },
                TokenTree::Punct(punct) => {
                    output.push(punct.as_char());
                }
                TokenTree::Ident(ident) => output.push_str(&ident.to_string()),
            }
        }
    }

    let mut output = String::new();
    concat_recursive_internal(&mut output, arguments);
    output
}
