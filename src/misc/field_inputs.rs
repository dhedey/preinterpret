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

macro_rules! if_exists {
    ({$($something:tt)+} {$($then:tt)*} {$($else:tt)*}) => { $($then)* };
    ({} {$($then:tt)*} {$($else:tt)*}) => { $($else)* };
}

pub(crate) use if_exists;

macro_rules! define_optional_object {
    (
        $vis:vis struct $model:ident {
            $(
                $optional_field:ident: $optional_field_type:ty $( = $optional_field_default:expr)? => ($optional_example:tt, $optional_description:literal)
            ),* $(,)?
        }
    ) => {
        define_typed_object! {
            $vis struct $model {
                required: {},
                optional: {
                    $(
                        $optional_field: $optional_field_type $( = $optional_field_default)? => ($optional_example, $optional_description)
                    ),*
                }
            }
        }
    }
}

pub(crate) use define_optional_object;

macro_rules! define_typed_object {
    (
        $vis:vis struct $model:ident {
            required: {
                $(
                    $required_field:ident: $required_field_type:ty => ($required_example:tt, $required_description:literal)
                ),* $(,)?
            }$(,)?
            optional: {
                $(
                    $optional_field:ident: $optional_field_type:ty $( = $optional_field_default:expr)? => ($optional_example:tt, $optional_description:literal)
                ),* $(,)?
            }$(,)?
        }
    ) => {
        $vis struct $model {
            $(
                $required_field: $required_field_type,
            )*
            $(
                $optional_field: if_exists!{
                    { $($optional_field_default)? }
                    { $optional_field_type }
                    { Option<$optional_field_type> }
                },
            )*
        }

        impl ResolvableArgumentTarget for $model {
            type ValueType = ObjectTypeData;
        }

        impl ResolvableArgumentOwned for $model {
            fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
                Self::try_from(ExpressionObject::resolve_from_owned(value)?)
            }
        }

        if_exists!{
            {$($required_field)*}
            {}
            {
                impl Default for $model {
                    fn default() -> Self {
                        Self {
                            $($optional_field: {
                                if_exists!{
                                    { $($optional_field_default)? }
                                    { $($optional_field_default)? }
                                    { None }
                                }
                            },)*
                        }
                    }
                }
            }
        }

        impl TryFrom<ExpressionObject> for $model {
            type Error = ExecutionInterrupt;

            fn try_from(mut object: ExpressionObject) -> Result<Self, Self::Error> {
                object.validate(&Self::validation())?;
                let span_range = object.span_range;
                Ok($model {
                    $(
                        $required_field: object.remove_or_none(stringify!($required_field), span_range),
                    )*
                    $(
                        $optional_field: {
                            let optional = object.remove_no_none(stringify!($optional_field), span_range);
                            if_exists!{
                                { $($optional_field_default)? }
                                {
                                    // Need to return the $optional_field_type
                                    match optional {
                                        Some(value) => <$optional_field_type as ResolvableArgumentOwned>::resolve_from_owned(value)?,
                                        None => $($optional_field_default)?,
                                    }
                                }
                                {
                                    // Need to return Option<$optional_field_type>
                                    match optional {
                                        Some(value) => Some(<$optional_field_type as ResolvableArgumentOwned>::resolve_from_owned(value)?),
                                        None => None,
                                    }
                                }
                            }
                        },
                    )*
                })
            }
        }

        impl $model {
            fn validation() -> &'static [(&'static str, FieldDefinition)] {
                static VALIDATION: [
                    (&'static str, FieldDefinition);
                    count_fields!($($required_field)* $($optional_field)*)
                ] = $model::VALIDATION;
                &VALIDATION
            }

            const VALIDATION: [
                (&'static str, FieldDefinition);
                count_fields!($($required_field)* $($optional_field)*)
            ] = [
                $(
                    (
                        stringify!($required_field),
                        FieldDefinition::new_full_static(true, $required_description, $required_example),
                    ),
                )*
                $(
                    (
                        stringify!($optional_field),
                        FieldDefinition::new_full_static(false, $optional_description, $optional_example),
                    ),
                )*
            ];
        }
    };
}
pub(crate) use define_typed_object;
