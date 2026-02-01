use super::*;

define_leaf_type! {
    pub(crate) StringType => AnyType(AnyValueContent::String),
    content: String,
    kind: pub(crate) StringKind,
    type_name: "string",
    articled_value_name: "a string",
    dyn_impls: {
        IterableType: impl IsIterable {
            fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue> {
                Ok(IteratorValue::new_for_string_over_chars(*self))
            }

            fn len(&self, _error_span_range: SpanRange) -> ExecutionResult<usize> {
                // The iterator is over chars, so this must count chars.
                // But contrast, string.len() counts bytes
                Ok(self.chars().count())
            }
        }
    },
}

impl IsValueContent for &str {
    type Type = StringType;
    type Form = BeRef;
}

impl<'a> FromValueContent<'a> for &'a str {
    fn from_content(value: &'a String) -> Self {
        value.as_str()
    }
}

impl ValuesEqual for String {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self == other {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

pub(crate) fn string_to_ident(
    str: &str,
    error_source: &impl HasSpanRange,
    span: Span,
) -> ExecutionResult<Ident> {
    let ident = parse_str::<Ident>(str).map_err(|err| {
        error_source.value_error::<ExecutionInterrupt>(format!("`{}` is not a valid ident: {:?}", str, err))
    })?;
    Ok(ident.with_span(span))
}

pub(crate) fn string_to_literal(
    str: &str,
    error_source: &impl HasSpanRange,
    span: Span,
) -> ExecutionResult<Literal> {
    let literal = Literal::from_str(str).map_err(|err| {
        error_source.value_error::<ExecutionInterrupt>(format!("`{}` is not a valid literal: {:?}", str, err))
    })?;
    Ok(literal.with_span(span))
}

define_type_features! {
    impl StringType,
    pub(crate) mod string_interface {
        methods {
            // ==================
            // CONVERSION METHODS
            // ==================
            [context] fn to_ident(this: Spanned<AnyRef<String>>) -> ExecutionResult<Ident> {
                string_to_ident(&this, &this, context.span_from_join_else_start())
            }

            [context] fn to_ident_camel(this: Spanned<AnyRef<String>>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_camel_case(&this);
                string_to_ident(&str, &this, context.span_from_join_else_start())
            }

            [context] fn to_ident_snake(this: Spanned<AnyRef<String>>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_lower_snake_case(&this);
                string_to_ident(&str, &this, context.span_from_join_else_start())
            }

            [context] fn to_ident_upper_snake(this: Spanned<AnyRef<String>>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_snake_case(&this);
                string_to_ident(&str, &this, context.span_from_join_else_start())
            }

            [context] fn to_literal(this: Spanned<AnyRef<String>>) -> ExecutionResult<Literal> {
                string_to_literal(&this, &this, context.span_from_join_else_start())
            }

            // ======================
            // STRING RESHAPE METHODS
            // ======================
            fn to_uppercase(this: AnyRef<String>) -> String {
                string_conversion::to_uppercase(&this)
            }

            fn to_lowercase(this: AnyRef<String>) -> String {
                string_conversion::to_lowercase(&this)
            }

            fn to_lower_snake_case(this: AnyRef<String>) -> String {
                string_conversion::to_lower_snake_case(&this)
            }

            fn to_upper_snake_case(this: AnyRef<String>) -> String {
                string_conversion::to_upper_snake_case(&this)
            }

            fn to_kebab_case(this: AnyRef<String>) -> String {
                string_conversion::to_lower_kebab_case(&this)
            }

            fn to_lower_camel_case(this: AnyRef<String>) -> String {
                string_conversion::to_lower_camel_case(&this)
            }

            fn to_upper_camel_case(this: AnyRef<String>) -> String {
                string_conversion::to_upper_camel_case(&this)
            }

            fn capitalize(this: AnyRef<String>) -> String {
                string_conversion::capitalize(&this)
            }

            fn decapitalize(this: AnyRef<String>) -> String {
                string_conversion::decapitalize(&this)
            }

            fn to_title_case(this: AnyRef<String>) -> String {
                string_conversion::title_case(&this)
            }

            fn insert_spaces(this: AnyRef<String>) -> String {
                string_conversion::insert_spaces_between_words(&this)
            }
        }
        unary_operations {
            fn cast_to_string(this: String) -> String {
                this
            }
        }
        binary_operations {
            fn add(mut lhs: String, rhs: AnyRef<String>) -> String {
                lhs.push_str(rhs.deref());
                lhs
            }

            fn add_assign(mut lhs: Assignee<String>, rhs: AnyRef<String>) {
                lhs.push_str(rhs.deref());
            }

            fn eq(lhs: AnyRef<String>, rhs: AnyRef<String>) -> bool {
                lhs.deref() == rhs.deref()
            }

            fn ne(lhs: AnyRef<String>, rhs: AnyRef<String>) -> bool {
                lhs.deref() != rhs.deref()
            }

            fn lt(lhs: AnyRef<String>, rhs: AnyRef<String>) -> bool {
                lhs.deref() < rhs.deref()
            }

            fn le(lhs: AnyRef<String>, rhs: AnyRef<String>) -> bool {
                lhs.deref() <= rhs.deref()
            }

            fn ge(lhs: AnyRef<String>, rhs: AnyRef<String>) -> bool {
                lhs.deref() >= rhs.deref()
            }

            fn gt(lhs: AnyRef<String>, rhs: AnyRef<String>) -> bool {
                lhs.deref() > rhs.deref()
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => return None,
                    UnaryOperation::Cast { target: CastTarget(kind), .. } => match kind {
                        AnyValueLeafKind::String(_) => unary_definitions::cast_to_string(),
                        _ => return None,
                    },
                })
            }

            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    BinaryOperation::Addition { .. } => binary_definitions::add(),
                    BinaryOperation::Equal { .. } => binary_definitions::eq(),
                    BinaryOperation::NotEqual { .. } => binary_definitions::ne(),
                    BinaryOperation::LessThan { .. } => binary_definitions::lt(),
                    BinaryOperation::LessThanOrEqual { .. } => binary_definitions::le(),
                    BinaryOperation::GreaterThanOrEqual { .. } => binary_definitions::ge(),
                    BinaryOperation::GreaterThan { .. } => binary_definitions::gt(),
                    BinaryOperation::AddAssign { .. } => binary_definitions::add_assign(),
                    _ => return None,
                })
            }
        }
    }
}

impl_resolvable_argument_for! {
    StringType,
    (value, context) -> String {
        match value {
            AnyValueContent::String(value) => Ok(value),
            _ => context.err("a string", value),
        }
    }
}

impl ResolvableArgumentTarget for str {
    type ValueType = StringType;
}

impl ResolvableShared<AnyValue> for str {
    fn resolve_from_ref<'a>(
        value: &'a AnyValue,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a Self> {
        match value {
            AnyValueContent::String(s) => Ok(s.as_str()),
            _ => context.err("a string", value),
        }
    }
}
