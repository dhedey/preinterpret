macro_rules! define_field_inputs {
    (
        $inputs_type:ident {
            required: {
                $(
                    $required_field:ident: $required_type:ty = $required_example:literal $(($required_description:literal))?
                ),* $(,)?
            }$(,)?
            optional: {
                $(
                    $optional_field:ident: $optional_type:ty = $optional_example:literal $(($optional_description:literal))?
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
            fn parse(input: ParseStream) -> Result<Self> {
                $(
                    let mut $required_field: Option<$required_type> = None;
                )*
                $(
                    let mut $optional_field: Option<$optional_type> = None;
                )*

                let content;
                let brace = syn::braced!(content in input);

                while !content.is_empty() {
                    let ident = content.call(Ident::parse_any)?;
                    content.parse::<Token![:]>()?;
                    match ident.to_string().as_str() {
                        $(
                            stringify!($required_field) => {
                                if $required_field.is_some() {
                                    return ident.err("duplicate field");
                                }
                                $required_field = Some(content.parse()?);
                            }
                        )*
                        $(
                            stringify!($optional_field) => {
                                if $optional_field.is_some() {
                                    return ident.err("duplicate field");
                                }
                                $optional_field = Some(content.parse()?);
                            }
                        )*
                        _ => return ident.err("unexpected field"),
                    }
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }
                let mut missing_fields: Vec<String> = vec![];

                $(
                    if $required_field.is_none() {
                        missing_fields.push(stringify!($required_field).to_string());
                    }
                )*

                if !missing_fields.is_empty() {
                    return brace.span.err(format!(
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
                use std::fmt::Write;
                let mut message = "Expected: {\n".to_string();
                let buffer = &mut message;
                $(
                    $(writeln!(buffer, "    // {}", $required_description).unwrap();)?
                    writeln!(buffer, "    {}: {},", stringify!($required_field), $required_example).unwrap();
                )*
                $(
                    $(writeln!(buffer, "    // {}", $optional_description).unwrap();)?
                    writeln!(buffer, "    {}?: {},", stringify!($optional_field), $optional_example).unwrap();
                )*
                buffer.push('}');
                message
            }
        }
    };
}

pub(crate) use define_field_inputs;
