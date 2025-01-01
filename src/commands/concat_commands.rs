use crate::internal_prelude::*;

//========
// Helpers
//========

fn string_literal(value: &str, span: Span) -> Literal {
    let mut literal = Literal::string(value);
    literal.set_span(span);
    literal
}

fn parse_literal(value: &str, span: Span) -> Result<Literal> {
    let mut literal = Literal::from_str(value)
        .map_err(|err| span.error(format!("`{}` is not a valid literal: {:?}", value, err,)))?;
    literal.set_span(span);
    Ok(literal)
}

fn parse_ident(value: &str, span: Span) -> Result<Ident> {
    let mut ident = parse_str::<Ident>(value)
        .map_err(|err| span.error(format!("`{}` is not a valid ident: {:?}", value, err,)))?;
    ident.set_span(span);
    Ok(ident)
}

fn concat_into_string(
    interpreter: &mut Interpreter,
    mut command: Command,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<TokenStream> {
    let interpreted = command.interpret_remaining_arguments(interpreter, SubstitutionMode::token_stream())?;
    let concatenated = concat_recursive(interpreted);
    let string_literal = string_literal(&conversion_fn(&concatenated), command.span());
    Ok(TokenStream::from(TokenTree::Literal(string_literal)))
}

fn concat_into_ident(
    interpreter: &mut Interpreter,
    mut command: Command,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<TokenStream> {
    let interpreted = command.interpret_remaining_arguments(interpreter, SubstitutionMode::token_stream())?;
    let concatenated = concat_recursive(interpreted);
    let ident = parse_ident(&conversion_fn(&concatenated), command.span())?;
    Ok(TokenStream::from(TokenTree::Ident(ident)))
}

fn concat_into_literal(
    interpreter: &mut Interpreter,
    mut command: Command,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<TokenStream> {
    let interpreted = command.interpret_remaining_arguments(interpreter, SubstitutionMode::token_stream())?;
    let concatenated = concat_recursive(interpreted);
    let literal = parse_literal(&conversion_fn(&concatenated), command.span())?;
    Ok(TokenStream::from(TokenTree::Literal(literal)))
}

//=======================================
// Concatenating type-conversion commands
//=======================================

pub(crate) struct StringCommand;

impl CommandDefinition for StringCommand {
    const COMMAND_NAME: &'static str = "string";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, |s| s.to_string())
    }
}

pub(crate) struct IdentCommand;

impl CommandDefinition for IdentCommand {
    const COMMAND_NAME: &'static str = "ident";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_ident(interpreter, command, |s| s.to_string())
    }
}

pub(crate) struct IdentCamelCommand;

impl CommandDefinition for IdentCamelCommand {
    const COMMAND_NAME: &'static str = "ident_camel";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_ident(interpreter, command, to_upper_camel_case)
    }
}

pub(crate) struct IdentSnakeCommand;

impl CommandDefinition for IdentSnakeCommand {
    const COMMAND_NAME: &'static str = "ident_snake";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_ident(interpreter, command, to_lower_snake_case)
    }
}

pub(crate) struct IdentUpperSnakeCommand;

impl CommandDefinition for IdentUpperSnakeCommand {
    const COMMAND_NAME: &'static str = "ident_upper_snake";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_ident(interpreter, command, to_upper_snake_case)
    }
}

pub(crate) struct LiteralCommand;

impl CommandDefinition for LiteralCommand {
    const COMMAND_NAME: &'static str = "literal";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_literal(interpreter, command, |s| s.to_string())
    }
}

//===========================
// String conversion commands
//===========================

pub(crate) struct UpperCommand;

impl CommandDefinition for UpperCommand {
    const COMMAND_NAME: &'static str = "upper";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, to_uppercase)
    }
}

pub(crate) struct LowerCommand;

impl CommandDefinition for LowerCommand {
    const COMMAND_NAME: &'static str = "lower";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, to_lowercase)
    }
}

pub(crate) struct SnakeCommand;

impl CommandDefinition for SnakeCommand {
    const COMMAND_NAME: &'static str = "snake";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        // Lower snake case is the more common casing in Rust, so default to that
        LowerSnakeCommand::execute(interpreter, command)
    }
}

pub(crate) struct LowerSnakeCommand;

impl CommandDefinition for LowerSnakeCommand {
    const COMMAND_NAME: &'static str = "lower_snake";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, to_lower_snake_case)
    }
}

pub(crate) struct UpperSnakeCommand;

impl CommandDefinition for UpperSnakeCommand {
    const COMMAND_NAME: &'static str = "upper_snake";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, to_upper_snake_case)
    }
}

pub(crate) struct KebabCommand;

impl CommandDefinition for KebabCommand {
    const COMMAND_NAME: &'static str = "kebab";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        // Kebab case is normally lower case (including in Rust where it's used - e.g. crate names)
        // It can always be combined with other casing to get other versions
        concat_into_string(interpreter, command, to_lower_kebab_case)
    }
}

pub(crate) struct CamelCommand;

impl CommandDefinition for CamelCommand {
    const COMMAND_NAME: &'static str = "camel";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        // Upper camel case is the more common casing in Rust, so default to that
        UpperCamelCommand::execute(interpreter, command)
    }
}

pub(crate) struct LowerCamelCommand;

impl CommandDefinition for LowerCamelCommand {
    const COMMAND_NAME: &'static str = "lower_camel";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, to_lower_camel_case)
    }
}

pub(crate) struct UpperCamelCommand;

impl CommandDefinition for UpperCamelCommand {
    const COMMAND_NAME: &'static str = "upper_camel";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, to_upper_camel_case)
    }
}

pub(crate) struct CapitalizeCommand;

impl CommandDefinition for CapitalizeCommand {
    const COMMAND_NAME: &'static str = "capitalize";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, capitalize)
    }
}

pub(crate) struct DecapitalizeCommand;

impl CommandDefinition for DecapitalizeCommand {
    const COMMAND_NAME: &'static str = "decapitalize";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, decapitalize)
    }
}

pub(crate) struct TitleCommand;

impl CommandDefinition for TitleCommand {
    const COMMAND_NAME: &'static str = "title";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(interpreter, command, title_case)
    }
}

pub(crate) struct InsertSpacesCommand;

impl CommandDefinition for InsertSpacesCommand {
    const COMMAND_NAME: &'static str = "insert_spaces";

    fn execute(
        interpreter: &mut Interpreter,
        command: Command,
    ) -> Result<TokenStream> {
        concat_into_string(
            interpreter,
            command,
            insert_spaces_between_words,
        )
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
