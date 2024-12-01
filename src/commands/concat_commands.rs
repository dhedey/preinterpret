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

pub(crate) struct UpperCaseCommand;

impl CommandDefinition for UpperCaseCommand {
    const COMMAND_NAME: &'static str = "upper_case";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_uppercase)
    }
}

pub(crate) struct LowerCaseCommand;

impl CommandDefinition for LowerCaseCommand {
    const COMMAND_NAME: &'static str = "lower_case";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_lowercase)
    }
}

pub(crate) struct SnakeCaseCommand;

impl CommandDefinition for SnakeCaseCommand {
    const COMMAND_NAME: &'static str = "snake_case";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        // Lower snake case is the more common casing in Rust, so default to that
        LowerSnakeCaseCommand::execute(interpreter, argument, command_span)
    }
}

pub(crate) struct LowerSnakeCaseCommand;

impl CommandDefinition for LowerSnakeCaseCommand {
    const COMMAND_NAME: &'static str = "lower_snake_case";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_lower_snake_case)
    }
}

pub(crate) struct UpperSnakeCaseCommand;

impl CommandDefinition for UpperSnakeCaseCommand {
    const COMMAND_NAME: &'static str = "upper_snake_case";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_upper_snake_case)
    }
}

pub(crate) struct CamelCaseCommand;

impl CommandDefinition for CamelCaseCommand {
    const COMMAND_NAME: &'static str = "camel_case";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        // Upper camel case is the more common casing in Rust, so default to that
        UpperCamelCaseCommand::execute(interpreter, argument, command_span)
    }
}

pub(crate) struct LowerCamelCaseCommand;

impl CommandDefinition for LowerCamelCaseCommand {
    const COMMAND_NAME: &'static str = "lower_camel_case";

    fn execute(
        interpreter: &mut Interpreter,
        argument: CommandArgumentStream,
        command_span: Span,
    ) -> Result<TokenStream> {
        concat_string_and_convert(interpreter, argument, command_span, to_lower_camel_case)
    }
}

pub(crate) struct UpperCamelCaseCommand;

impl CommandDefinition for UpperCamelCaseCommand {
    const COMMAND_NAME: &'static str = "upper_camel_case";

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
