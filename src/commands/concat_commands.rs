use crate::internal_prelude::*;

//========
// Helpers
//========

fn concat_into_string(
    input: InterpretationStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<Literal> {
    let output_span = input.span();
    let concatenated = input
        .interpret_to_new_stream(interpreter)?
        .concat_recursive();
    let value = conversion_fn(&concatenated);
    Ok(Literal::string(&value).with_span(output_span))
}

fn concat_into_ident(
    input: InterpretationStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<Ident> {
    let output_span = input.span();
    let concatenated = input
        .interpret_to_new_stream(interpreter)?
        .concat_recursive();
    let value = conversion_fn(&concatenated);
    let ident = parse_str::<Ident>(&value)
        .map_err(|err| output_span.error(format!("`{}` is not a valid ident: {:?}", value, err,)))?
        .with_span(output_span);
    Ok(ident)
}

fn concat_into_literal(
    input: InterpretationStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<Literal> {
    let output_span = input.span();
    let concatenated = input
        .interpret_to_new_stream(interpreter)?
        .concat_recursive();
    let value = conversion_fn(&concatenated);
    let literal = Literal::from_str(&value)
        .map_err(|err| {
            output_span.error(format!("`{}` is not a valid literal: {:?}", value, err,))
        })?
        .with_span(output_span);
    Ok(literal)
}

macro_rules! define_literal_concat_command {
    (
        $command_name:literal => $command:ident: $output_fn:ident($conversion_fn:expr)
    ) => {
        #[derive(Clone)]
        pub(crate) struct $command {
            arguments: InterpretationStream,
        }

        impl CommandType for $command {
            type OutputKind = OutputKindValue;
        }

        impl ValueCommandDefinition for $command {
            const COMMAND_NAME: &'static str = $command_name;

            fn parse(arguments: CommandArguments) -> Result<Self> {
                Ok(Self {
                    arguments: arguments.parse_all_for_interpretation()?,
                })
            }

            fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<TokenTree> {
                Ok($output_fn(self.arguments, interpreter, $conversion_fn)?.into())
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
            arguments: InterpretationStream,
        }

        impl CommandType for $command {
            type OutputKind = OutputKindIdent;
        }

        impl IdentCommandDefinition for $command {
            const COMMAND_NAME: &'static str = $command_name;

            fn parse(arguments: CommandArguments) -> Result<Self> {
                Ok(Self {
                    arguments: arguments.parse_all_for_interpretation()?,
                })
            }

            fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<Ident> {
                $output_fn(self.arguments, interpreter, $conversion_fn).into()
            }
        }
    };
}

//=======================================
// Concatenating type-conversion commands
//=======================================

define_literal_concat_command!("string" => StringCommand: concat_into_string(|s| s.to_string()));
define_ident_concat_command!("ident" => IdentCommand: concat_into_ident(|s| s.to_string()));
define_ident_concat_command!("ident_camel" => IdentCamelCommand: concat_into_ident(to_upper_camel_case));
define_ident_concat_command!("ident_snake" => IdentSnakeCommand: concat_into_ident(to_lower_snake_case));
define_ident_concat_command!("ident_upper_snake" => IdentUpperSnakeCommand: concat_into_ident(to_upper_snake_case));
define_literal_concat_command!("literal" => LiteralCommand: concat_into_literal(|s| s.to_string()));

//===========================
// String conversion commands
//===========================

define_literal_concat_command!("upper" => UpperCommand: concat_into_string(to_uppercase));
define_literal_concat_command!("lower" => LowerCommand: concat_into_string(to_lowercase));
// Snake case is typically lower snake case in Rust, so default to that
define_literal_concat_command!("snake" => SnakeCommand: concat_into_string(to_lower_snake_case));
define_literal_concat_command!("lower_snake" => LowerSnakeCommand: concat_into_string(to_lower_snake_case));
define_literal_concat_command!("upper_snake" => UpperSnakeCommand: concat_into_string(to_upper_snake_case));
// Kebab case is normally lower case (including in Rust where it's used - e.g. crate names)
// It can always be combined with other casing to get other versions
define_literal_concat_command!("kebab" => KebabCommand: concat_into_string(to_lower_kebab_case));
// Upper camel case is the more common casing in Rust, so default to that
define_literal_concat_command!("camel" => CamelCommand: concat_into_string(to_upper_camel_case));
define_literal_concat_command!("lower_camel" => LowerCamelCommand: concat_into_string(to_lower_camel_case));
define_literal_concat_command!("upper_camel" => UpperCamelCommand: concat_into_string(to_upper_camel_case));
define_literal_concat_command!("capitalize" => CapitalizeCommand: concat_into_string(capitalize));
define_literal_concat_command!("decapitalize" => DecapitalizeCommand: concat_into_string(decapitalize));
define_literal_concat_command!("title" => TitleCommand: concat_into_string(title_case));
define_literal_concat_command!("insert_spaces" => InsertSpacesCommand: concat_into_string(insert_spaces_between_words));
