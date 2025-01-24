macro_rules! define_field_inputs {
    (
        $inputs_type:ident {
            required: {
                $(
                    $required_field:ident: $required_type:ty = $required_example:tt $(($required_description:literal))?
                ),* $(,)?
            }$(,)?
            optional: {
                $(
                    $optional_field:ident: $optional_type:ty = $optional_example:tt $(($optional_description:literal))?
                ),* $(,)?
            }$(,)?
        }
    ) => {
        #[derive(Clone)]
        struct $inputs_type {
            $(
                $required_field: $required_type,
            )*

            $(
                $optional_field: Option<$optional_type>,
            )*
        }

        impl Parse for $inputs_type {
            fn parse(input: ParseStream) -> ParseResult<Self> {
                $(
                    let mut $required_field: Option<$required_type> = None;
                )*
                $(
                    let mut $optional_field: Option<$optional_type> = None;
                )*

                let (delim_span, content) = input.parse_group_matching(Delimiter::Brace)?;

                while !content.is_empty() {
                    let ident = content.parse_any_ident()?;
                    content.parse::<Token![:]>()?;
                    match ident.to_string().as_str() {
                        $(
                            stringify!($required_field) => {
                                if $required_field.is_some() {
                                    return ident.parse_err("duplicate field");
                                }
                                $required_field = Some(content.parse_v2()?);
                            }
                        )*
                        $(
                            stringify!($optional_field) => {
                                if $optional_field.is_some() {
                                    return ident.parse_err("duplicate field");
                                }
                                $optional_field = Some(content.parse_v2()?);
                            }
                        )*
                        _ => return ident.parse_err("unexpected field"),
                    }
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }

                #[allow(unused_mut)]
                let mut missing_fields: Vec<String> = vec![];

                $(
                    if $required_field.is_none() {
                        missing_fields.push(stringify!($required_field).to_string());
                    }
                )*

                if !missing_fields.is_empty() {
                    return delim_span.join().parse_err(format!(
                        "required fields are missing: {}",
                        missing_fields.join(", ")
                    ));
                }

                $(
                    let $required_field = $required_field.unwrap();
                )*

                Ok(Self {
                    $(
                        $required_field,
                    )*
                    $(
                        $optional_field,
                    )*
                })
            }
        }

        impl ArgumentsContent for $inputs_type {
            fn error_message() -> String {
                format!("Expected: {}", Self::fields_description())
            }
        }

        impl $inputs_type {
            fn fields_description() -> String {
                use std::fmt::Write;
                let mut buffer = String::new();
                buffer.write_str("{\n").unwrap();
                $(
                    $(writeln!(buffer, "    // {}", $required_description).unwrap();)?
                    writeln!(buffer, "    {}: {},", stringify!($required_field), $required_example).unwrap();
                )*
                $(
                    $(writeln!(buffer, "    // {}", $optional_description).unwrap();)?
                    writeln!(buffer, "    {}?: {},", stringify!($optional_field), $optional_example).unwrap();
                )*
                buffer.write_str("}").unwrap();
                buffer
            }
        }
    };
}

pub(crate) use define_field_inputs;
