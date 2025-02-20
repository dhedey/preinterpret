macro_rules! define_object_arguments {
    (
        $source:ident => $validated:ident {
            required: {
                $(
                    $required_field:ident: $required_example:tt $(($required_description:literal))?
                ),* $(,)?
            }$(,)?
            optional: {
                $(
                    $optional_field:ident: $optional_example:tt $(($optional_description:literal))?
                ),* $(,)?
            }$(,)?
        }
    ) => {
        #[derive(Clone)]
        struct $source {
            inner: SourceExpression,
        }

        impl ArgumentsContent for $source {
            fn error_message() -> String {
                format!("Expected: {}", Self::describe_object())
            }
        }

        impl Parse<Source> for $source {
            fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
                Ok(Self { inner: input.parse()? })
            }
        }

        impl InterpretToValue for &$source {
            type OutputValue = $validated;

            fn interpret_to_value(
                self,
                interpreter: &mut Interpreter,
            ) -> ExecutionResult<Self::OutputValue> {
                let mut object = self.inner.interpret_to_value(interpreter)?
                    .expect_object("The arguments")?;
                object.validate(&$source::validation())?;
                let span_range = object.span_range;
                Ok($validated {
                    $(
                        $required_field: object.remove_or_none(stringify!($required_field), span_range),
                    )*
                    $(
                        $optional_field: object.remove_no_none(stringify!($optional_field), span_range),
                    )*
                })
            }
        }

        struct $validated {
            $(
                $required_field: ExpressionValue,
            )*
            $(
                $optional_field: Option<ExpressionValue>,
            )*
        }

        impl $source {
            fn describe_object() -> String {
                Self::validation().describe_object()
            }

            fn validation() -> &'static [(&'static str, FieldDefinition)] {
                &Self::VALIDATION
            }

            const VALIDATION: [
                (&'static str, FieldDefinition);
                count_fields!($($required_field)* $($optional_field)*)
            ] = [
                $(
                    (
                        stringify!($required_field),
                        FieldDefinition {
                            required: true,
                            description: optional_else!{
                                { $(Some(std::borrow::Cow::Borrowed($required_description)))? }
                                { None }
                            },
                            example: std::borrow::Cow::Borrowed($required_example),
                        },
                    ),
                )*
                $(
                    (
                        stringify!($optional_field),
                        FieldDefinition {
                            required: false,
                            description: optional_else!{
                                { $(Some(std::borrow::Cow::Borrowed($optional_description)))? }
                                { None }
                            },
                            example: std::borrow::Cow::Borrowed($optional_example),
                        },
                    ),
                )*
            ];
        }
    };
}

pub(crate) use define_object_arguments;

macro_rules! count_fields {
    ($head:tt $($tail:tt)*) => { 1 + count_fields!($($tail)*)};
    () => { 0 }
}

pub(crate) use count_fields;

macro_rules! optional_else {
    ({$($content:tt)+} {$($else:tt)*}) => { $($content)+ };
    ({} {$($else:tt)*}) => { $($else)* }
}

pub(crate) use optional_else;
