use crate::internal_prelude::*;

//========
// Helpers
//========

fn concat_into_string(
    input: SourceStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> ExecutionResult<ExpressionValue> {
    let output_span = input.span();
    let concatenated = input
        .interpret_to_new_stream(interpreter)?
        .concat_recursive(&ConcatBehaviour::standard());
    Ok(conversion_fn(&concatenated).to_value(output_span.span_range()))
}

fn concat_into_ident(
    input: SourceStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> ExecutionResult<Ident> {
    let output_span = input.span();
    let concatenated = input
        .interpret_to_new_stream(interpreter)?
        .concat_recursive(&ConcatBehaviour::standard());
    let value = conversion_fn(&concatenated);
    let ident = parse_str::<Ident>(&value)
        .map_err(|err| output_span.error(format!("`{}` is not a valid ident: {:?}", value, err,)))?
        .with_span(output_span);
    Ok(ident)
}

fn concat_into_literal(
    input: SourceStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> ExecutionResult<Literal> {
    let output_span = input.span();
    let concatenated = input
        .interpret_to_new_stream(interpreter)?
        .concat_recursive(&ConcatBehaviour::literal());
    let value = conversion_fn(&concatenated);
    let literal = Literal::from_str(&value)
        .map_err(|err| {
            output_span.error(format!("`{}` is not a valid literal: {:?}", value, err,))
        })?
        .with_span(output_span);
    Ok(literal)
}

macro_rules! define_string_concat_command {
    (
        $command_name:literal => $command:ident: $output_fn:ident($conversion_fn:expr)
    ) => {
        #[derive(Clone)]
        pub(crate) struct $command {
            arguments: SourceStream,
        }

        impl CommandType for $command {
            type OutputKind = OutputKindValue;
        }

        impl ValueCommandDefinition for $command {
            const COMMAND_NAME: &'static str = $command_name;

            fn parse(arguments: CommandArguments) -> ParseResult<Self> {
                Ok(Self {
                    arguments: arguments.parse_all_as_source()?,
                })
            }

            fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
                $output_fn(self.arguments, interpreter, $conversion_fn)
            }
        }
    };
}

macro_rules! define_ident_concat_command {
    (
        $command_name:literal => $command:ident: $output_fn:ident($conversion_fn:expr)
    ) => {
        #[derive(Clone)]
        pub(crate) struct $command {
            arguments: SourceStream,
        }

        impl CommandType for $command {
            type OutputKind = OutputKindIdent;
        }

        impl IdentCommandDefinition for $command {
            const COMMAND_NAME: &'static str = $command_name;

            fn parse(arguments: CommandArguments) -> ParseResult<Self> {
                Ok(Self {
                    arguments: arguments.parse_all_as_source()?,
                })
            }

            fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<Ident> {
                $output_fn(self.arguments, interpreter, $conversion_fn)
            }
        }
    };
}

macro_rules! define_literal_concat_command {
    (
        $command_name:literal => $command:ident: $output_fn:ident($conversion_fn:expr)
    ) => {
        #[derive(Clone)]
        pub(crate) struct $command {
            arguments: SourceStream,
        }

        impl CommandType for $command {
            type OutputKind = OutputKindLiteral;
        }

        impl LiteralCommandDefinition for $command {
            const COMMAND_NAME: &'static str = $command_name;

            fn parse(arguments: CommandArguments) -> ParseResult<Self> {
                Ok(Self {
                    arguments: arguments.parse_all_as_source()?,
                })
            }

            fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<Literal> {
                $output_fn(self.arguments, interpreter, $conversion_fn)
            }
        }
    };
}

//=======================================
// Concatenating type-conversion commands
//=======================================

define_string_concat_command!("string" => StringCommand: concat_into_string(|s| s.to_string()));
define_ident_concat_command!("ident" => IdentCommand: concat_into_ident(|s| s.to_string()));
define_ident_concat_command!("ident_camel" => IdentCamelCommand: concat_into_ident(to_upper_camel_case));
define_ident_concat_command!("ident_snake" => IdentSnakeCommand: concat_into_ident(to_lower_snake_case));
define_ident_concat_command!("ident_upper_snake" => IdentUpperSnakeCommand: concat_into_ident(to_upper_snake_case));
define_literal_concat_command!("literal" => LiteralCommand: concat_into_literal(|s| s.to_string()));

//===========================
// String conversion commands
//===========================

define_string_concat_command!("upper" => UpperCommand: concat_into_string(to_uppercase));
define_string_concat_command!("lower" => LowerCommand: concat_into_string(to_lowercase));
// Snake case is typically lower snake case in Rust, so default to that
define_string_concat_command!("snake" => SnakeCommand: concat_into_string(to_lower_snake_case));
define_string_concat_command!("lower_snake" => LowerSnakeCommand: concat_into_string(to_lower_snake_case));
define_string_concat_command!("upper_snake" => UpperSnakeCommand: concat_into_string(to_upper_snake_case));
// Kebab case is normally lower case (including in Rust where it's used - e.g. crate names)
// It can always be combined with other casing to get other versions
define_string_concat_command!("kebab" => KebabCommand: concat_into_string(to_lower_kebab_case));
// Upper camel case is the more common casing in Rust, so default to that
define_string_concat_command!("camel" => CamelCommand: concat_into_string(to_upper_camel_case));
define_string_concat_command!("lower_camel" => LowerCamelCommand: concat_into_string(to_lower_camel_case));
define_string_concat_command!("upper_camel" => UpperCamelCommand: concat_into_string(to_upper_camel_case));
define_string_concat_command!("capitalize" => CapitalizeCommand: concat_into_string(capitalize));
define_string_concat_command!("decapitalize" => DecapitalizeCommand: concat_into_string(decapitalize));
define_string_concat_command!("title" => TitleCommand: concat_into_string(title_case));
define_string_concat_command!("insert_spaces" => InsertSpacesCommand: concat_into_string(insert_spaces_between_words));
