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
    input: InterpretationStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<InterpretedStream> {
    let output_span = input.span();
    let concatenated = concat_recursive(input.interpret_as_tokens(interpreter)?);
    let string_literal = string_literal(&conversion_fn(&concatenated), output_span);
    Ok(InterpretedStream::of_literal(string_literal))
}

fn concat_into_ident(
    input: InterpretationStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<InterpretedStream> {
    let output_span = input.span();
    let concatenated = concat_recursive(input.interpret_as_tokens(interpreter)?);
    let ident = parse_ident(&conversion_fn(&concatenated), output_span)?;
    Ok(InterpretedStream::of_ident(ident))
}

fn concat_into_literal(
    input: InterpretationStream,
    interpreter: &mut Interpreter,
    conversion_fn: impl Fn(&str) -> String,
) -> Result<InterpretedStream> {
    let output_span = input.span();
    let concatenated = concat_recursive(input.interpret_as_tokens(interpreter)?);
    let literal = parse_literal(&conversion_fn(&concatenated), output_span)?;
    Ok(InterpretedStream::of_literal(literal))
}


fn concat_recursive(arguments: InterpretedStream) -> String {
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
    concat_recursive_internal(&mut output, arguments.into_token_stream());
    output
}

macro_rules! define_concat_command {
    (
        $command_name:literal => $command:ident: $output_fn:ident($conversion_fn:expr)
    ) => {
        #[derive(Clone)]
        pub(crate) struct $command {
            arguments: InterpretationStream,
        }

        impl CommandDefinition for $command {
            const COMMAND_NAME: &'static str = $command_name;
        
            const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::SingleToken;
        
            fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
                Ok(Self {
                    arguments: arguments.parse_all_for_interpretation()?,
                })
            }
        }

        impl CommandInvocation for $command {
            fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
                $output_fn(self.arguments, interpreter, $conversion_fn)
            }
        }
    }
}

//=======================================
// Concatenating type-conversion commands
//=======================================

define_concat_command!("string" => StringCommand: concat_into_string(|s| s.to_string()));
define_concat_command!("ident" => IdentCommand: concat_into_ident(|s| s.to_string()));
define_concat_command!("ident_camel" => IdentCamelCommand: concat_into_ident(to_upper_camel_case));
define_concat_command!("ident_snake" => IdentSnakeCommand: concat_into_ident(to_lower_snake_case));
define_concat_command!("ident_upper_snake" => IdentUpperSnakeCommand: concat_into_ident(to_upper_snake_case));
define_concat_command!("literal" => LiteralCommand: concat_into_literal(|s| s.to_string()));

//===========================
// String conversion commands
//===========================

define_concat_command!("upper" => UpperCommand: concat_into_string(to_uppercase));
define_concat_command!("lower" => LowerCommand: concat_into_string(to_lowercase));
// Snake case is typically lower snake case in Rust, so default to that
define_concat_command!("snake" => SnakeCommand: concat_into_string(to_lower_snake_case));
define_concat_command!("lower_snake" => LowerSnakeCommand: concat_into_string(to_lower_snake_case));
define_concat_command!("upper_snake" => UpperSnakeCommand: concat_into_string(to_upper_snake_case));
// Kebab case is normally lower case (including in Rust where it's used - e.g. crate names)
// It can always be combined with other casing to get other versions
define_concat_command!("kebab" => KebabCommand: concat_into_string(to_lower_kebab_case));
// Upper camel case is the more common casing in Rust, so default to that
define_concat_command!("camel" => CamelCommand: concat_into_string(to_upper_camel_case));
define_concat_command!("lower_camel" => LowerCamelCommand: concat_into_string(to_lower_camel_case));
define_concat_command!("upper_camel" => UpperCamelCommand: concat_into_string(to_upper_camel_case));
define_concat_command!("capitalize" => CapitalizeCommand: concat_into_string(capitalize));
define_concat_command!("decapitalize" => DecapitalizeCommand: concat_into_string(decapitalize));
define_concat_command!("title" => TitleCommand: concat_into_string(title_case));
define_concat_command!("insert_spaces" => InsertSpacesCommand: concat_into_string(insert_spaces_between_words));
