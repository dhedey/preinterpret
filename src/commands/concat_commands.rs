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

pub(crate) struct ToStringCommand;

impl CommandDefinition for ToStringCommand {
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

pub(crate) struct ToIdentCommand;

impl CommandDefinition for ToIdentCommand {
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

pub(crate) struct ToLiteralCommand;

impl CommandDefinition for ToLiteralCommand {
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
