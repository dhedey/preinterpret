macro_rules! count_fields {
    ($head:tt $($tail:tt)*) => { 1 + count_fields!($($tail)*)};
    () => { 0 }
}

pub(crate) use count_fields;

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
            fn resolve_from_value(value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
                Self::try_from(ObjectValue::resolve_owned_from_value(value, context)?)
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

        impl TryFrom<Owned<ObjectValue>> for $model {
            type Error = ExecutionInterrupt;

            fn try_from(object: Owned<ObjectValue>) -> Result<Self, Self::Error> {
                let (mut object, span_range) = object.deconstruct();
                (&object).spanned(span_range).validate(&Self::validation())?;
                Ok($model {
                    $(
                        $required_field: object.remove_or_none(stringify!($required_field)),
                    )*
                    $(
                        $optional_field: {
                            let optional = object.remove_no_none(stringify!($optional_field));
                            if_exists!{
                                { $($optional_field_default)? }
                                {
                                    // Need to return the $optional_field_type
                                    match optional {
                                        Some(value) => ResolveAs::<$optional_field_type>::resolve_as(value.into_owned(span_range), stringify!($optional_field))?,
                                        None => $($optional_field_default)?,
                                    }
                                }
                                {
                                    // Need to return Option<$optional_field_type>
                                    match optional {
                                        Some(value) => Some(ResolveAs::<$optional_field_type>::resolve_as(value.into_owned(span_range), stringify!($optional_field))?),
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
