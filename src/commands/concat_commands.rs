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

//=======================================
// Concatenating type-conversion commands
//=======================================

pub(crate) struct StringCommand;

impl CommandDefinition for StringCommand {
    const COMMAND_NAME: &'static str = "string";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let string_literal = string_literal(&interpreted, command_span);
        Ok(TokenStream::from(TokenTree::Literal(string_literal)))
    }
}

pub(crate) struct IdentCommand;

impl CommandDefinition for IdentCommand {
    const COMMAND_NAME: &'static str = "ident";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let parsed_ident = parse_ident(&interpreted, command_span)?;
        Ok(TokenStream::from(TokenTree::Ident(parsed_ident)))
    }
}

pub(crate) struct IdentCamelCommand;

impl CommandDefinition for IdentCamelCommand {
    const COMMAND_NAME: &'static str = "ident_camel";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let upper_camel_cased = to_upper_camel_case(&interpreted);
        let parsed_ident = parse_ident(&upper_camel_cased, command_span)?;
        Ok(TokenStream::from(TokenTree::Ident(parsed_ident)))
    }
}

pub(crate) struct IdentSnakeCommand;

impl CommandDefinition for IdentSnakeCommand {
    const COMMAND_NAME: &'static str = "ident_snake";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let lower_snake_cased = to_lower_snake_case(&interpreted);
        let parsed_ident = parse_ident(&lower_snake_cased, command_span)?;
        Ok(TokenStream::from(TokenTree::Ident(parsed_ident)))
    }
}

pub(crate) struct IdentUpperSnakeCommand;

impl CommandDefinition for IdentUpperSnakeCommand {
    const COMMAND_NAME: &'static str = "ident_upper_snake";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let upper_snake_cased = to_upper_snake_case(&interpreted);
        let parsed_ident = parse_ident(&upper_snake_cased, command_span)?;
        Ok(TokenStream::from(TokenTree::Ident(parsed_ident)))
    }
}

pub(crate) struct LiteralCommand;

impl CommandDefinition for LiteralCommand {
    const COMMAND_NAME: &'static str = "literal";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
        let parsed_literal = parse_literal(&interpreted, command_span)?;
        Ok(TokenStream::from(TokenTree::Literal(parsed_literal)))
    }
}

//===========================
// String conversion commands
//===========================

fn concat_string_and_convert(
    interpreter: &mut Interpreter,
    argument: CommandArgumentStream,
    command_span: Span,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<TokenStream> {
    let interpreted = argument.interpret_and_concat_to_string(interpreter)?;
    let string_literal = string_literal(&conversion_fn(&interpreted), command_span);
    Ok(TokenStream::from(TokenTree::Literal(string_literal)))
}

pub(crate) struct UpperCommand;

impl CommandDefinition for UpperCommand {
    const COMMAND_NAME: &'static str = "upper";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_uppercase)
    }
}

pub(crate) struct LowerCommand;

impl CommandDefinition for LowerCommand {
    const COMMAND_NAME: &'static str = "lower";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_lowercase)
    }
}

pub(crate) struct SnakeCommand;

impl CommandDefinition for SnakeCommand {
    const COMMAND_NAME: &'static str = "snake";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        // Lower snake case is the more common casing in Rust, so default to that
        LowerSnakeCommand::execute(interpreter, argument, command_span)
    }
}

pub(crate) struct LowerSnakeCommand;

impl CommandDefinition for LowerSnakeCommand {
    const COMMAND_NAME: &'static str = "lower_snake";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_lower_snake_case)
    }
}

pub(crate) struct UpperSnakeCommand;

impl CommandDefinition for UpperSnakeCommand {
    const COMMAND_NAME: &'static str = "upper_snake";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_upper_snake_case)
    }
}

pub(crate) struct KebabCommand;

impl CommandDefinition for KebabCommand {
    const COMMAND_NAME: &'static str = "kebab";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        // Kebab case is normally lower case (including in Rust where it's used - e.g. crate names)
        // It can always be combined with other casing to get other versions
        concat_string_and_convert(interpreter, argument, command_span, to_lower_kebab_case)
    }
}

pub(crate) struct CamelCommand;

impl CommandDefinition for CamelCommand {
    const COMMAND_NAME: &'static str = "camel";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        // Upper camel case is the more common casing in Rust, so default to that
        UpperCamelCommand::execute(interpreter, argument, command_span)
    }
}

pub(crate) struct LowerCamelCommand;

impl CommandDefinition for LowerCamelCommand {
    const COMMAND_NAME: &'static str = "lower_camel";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_lower_camel_case)
    }
}

pub(crate) struct UpperCamelCommand;

impl CommandDefinition for UpperCamelCommand {
    const COMMAND_NAME: &'static str = "upper_camel";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_upper_camel_case)
    }
}

pub(crate) struct CapitalizeCommand;

impl CommandDefinition for CapitalizeCommand {
    const COMMAND_NAME: &'static str = "capitalize";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, capitalize)
    }
}

pub(crate) struct DecapitalizeCommand;

impl CommandDefinition for DecapitalizeCommand {
    const COMMAND_NAME: &'static str = "decapitalize";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, decapitalize)
    }
}

pub(crate) struct TitleCommand;

impl CommandDefinition for TitleCommand {
    const COMMAND_NAME: &'static str = "title";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, title_case)
    }
}

pub(crate) struct InsertSpacesCommand;

impl CommandDefinition for InsertSpacesCommand {
    const COMMAND_NAME: &'static str = "insert_spaces";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(
            interpreter,
            argument,
            command_span,
            insert_spaces_between_words,
        )
    }
}
