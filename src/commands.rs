use crate::internal_prelude::*;

pub(crate) struct CommandInvocation {
    command_kind: CommandKind,
    argument_stream: CommandArgumentStream,
    command_span: Span,
}

pub(crate) struct CommandArgumentStream {
    tokens: PeekableTokenIter,
}

impl CommandArgumentStream {
    fn new(tokens: PeekableTokenIter) -> Self {
        Self {
            tokens,
        }
    }

    pub(crate) fn interpret_and_concat_to_string(self, interpreter: &mut Interpreter) -> Result<String> {
        let interpreted = interpreter.interpret_tokens(self.tokens)?;
        let mut output = String::new();
        concat_recursive_internal(&mut output, interpreted);
        Ok(output)
    }

    pub(crate) fn raw_tokens(self) -> PeekableTokenIter {
        self.tokens
    }
}

impl CommandInvocation {
    pub fn new(command_kind: CommandKind, group: &Group, argument_tokens: PeekableTokenIter) -> Self {
        Self {
            command_kind,
            argument_stream: CommandArgumentStream::new(argument_tokens),
            command_span: group.span(),
        }
    }

    pub fn execute(self, interpreter: &mut Interpreter) -> Result<TokenStream> {
        let Self {
            command_kind,
            argument_stream,
            command_span,
        } = self;

        match command_kind {
            CommandKind::Set => SetCommand::execute(interpreter, argument_stream, command_span),
            CommandKind::ToString => ToStringCommand::execute(interpreter, argument_stream, command_span),
            CommandKind::ToIdent => ToIdentCommand::execute(interpreter, argument_stream, command_span),
            CommandKind::ToLiteral => ToLiteralCommand::execute(interpreter, argument_stream, command_span),
            CommandKind::AsRawTokens => AsRawTokensCommand::execute(interpreter, argument_stream, command_span),
        }
    }
}

pub(crate) trait CommandDefinition {
    fn execute(interpreter: &mut Interpreter, argument: CommandArgumentStream, command_span: Span) -> Result<TokenStream>;
}

// TODO:
// * Create a macro to define the matching of the command names to commands
pub(crate) enum CommandKind {
    Set,
    ToString,
    ToIdent,
    ToLiteral,
    AsRawTokens,
}

pub(crate) fn parse_supported_command_kind(ident: &Ident) -> Option<CommandKind> {
    Some(match ident.to_string().as_ref() {
        "set" => CommandKind::Set,
        "string" => CommandKind::ToString,
        "ident" => CommandKind::ToIdent,
        "literal" => CommandKind::ToLiteral,
        "raw" => CommandKind::AsRawTokens,
        _ => return None,
    })
}

const ALL_COMMAND_KINDS: &[&str] = &["set", "string", "ident", "literal", "raw"];

impl CommandKind {
    pub(crate) fn list_all() -> String {
        ALL_COMMAND_KINDS.join(", ")
    }
}

pub(crate) struct SetCommand;

impl CommandDefinition for SetCommand {
    fn execute(interpreter: &mut Interpreter, argument: CommandArgumentStream, command_span: Span) -> Result<TokenStream> {
        let mut argument_tokens = argument.raw_tokens();
        let Some(ident) = consume_set_statement(&mut argument_tokens) else {
            return Err(command_span.error("A set call is expected to start with `#VariableName = ..`."));
        };
    
        let result_tokens = interpreter.interpret_tokens(argument_tokens)?;
        interpreter.set_variable(ident.to_string(), result_tokens);
    
        return Ok(TokenStream::new());
    }
}

pub(crate) struct ToStringCommand;

impl CommandDefinition for ToStringCommand {
    fn execute(interpreter: &mut Interpreter, argument: CommandArgumentStream, command_span: Span) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let literal = {
            let mut literal = Literal::string(&interpreted);
            literal.set_span(command_span);
            literal
        };
        Ok(TokenStream::from(TokenTree::Literal(literal)))
    }
}

pub(crate) struct ToIdentCommand;

impl CommandDefinition for ToIdentCommand {
    fn execute(interpreter: &mut Interpreter, argument: CommandArgumentStream, command_span: Span) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let ident = {
            let mut ident = parse_str::<Ident>(&interpreted)
                .map_err(|err| command_span.error(format!("`{interpreted}` is not a valid ident: {err:?}")))?;
            ident.set_span(command_span);
            ident
        };
        Ok(TokenStream::from(TokenTree::Ident(ident)))
    }
}

pub(crate) struct ToLiteralCommand;

impl CommandDefinition for ToLiteralCommand {
    fn execute(interpreter: &mut Interpreter, argument: CommandArgumentStream, command_span: Span) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let literal = {
            let mut literal = Literal::from_str(&interpreted)
                .map_err(|err| command_span.error(format!("`{interpreted}` is not a valid literal: {err:?}")))?;
            literal.set_span(command_span);
            literal
        };
        Ok(TokenTree::Literal(literal).into())
    }
}

pub(crate) struct AsRawTokensCommand;

impl CommandDefinition for AsRawTokensCommand {
    fn execute(_interpreter: &mut Interpreter, argument: CommandArgumentStream, _command_span: Span) -> Result<TokenStream> {
        Ok(argument.raw_tokens().collect())
    }
}

fn concat_recursive_internal(output: &mut String, arguments: TokenStream) {
    for token_tree in arguments {
        match token_tree {
            TokenTree::Literal(literal) => {
                let lit: Lit = parse_str(&literal.to_string()).expect("All proc_macro2::Literal values should be decodable as a syn::Lit");
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
