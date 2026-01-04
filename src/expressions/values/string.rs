use super::*;

define_leaf_type! {
    pub(crate) StringType => ValueType(ValueContent::String),
    content: String,
    kind: pub(crate) StringKind,
    type_name: "string",
    articled_display_name: "a string",
    temp_type_data: StringTypeData,
}

#[derive(Clone)]
pub(crate) struct StringValue {
    pub(crate) value: String,
}

impl Debug for StringValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl IntoValue for StringValue {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl StringValue {
    pub(super) fn for_litstr(lit: &syn::LitStr) -> Owned<Self> {
        Self { value: lit.value() }.into_owned()
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        Literal::string(&self.value).with_span(span)
    }
}

impl HasLeafKind for StringValue {
    type LeafKind = StringKind;

    fn kind(&self) -> Self::LeafKind {
        StringKind
    }
}

impl ValuesEqual for StringValue {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.value == other.value {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(StringValue { value: self })
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Value {
        Value::String(StringValue {
            value: self.to_string(),
        })
    }
}

pub(crate) fn string_to_ident(
    str: &str,
    error_source: &impl HasSpanRange,
    span: Span,
) -> ExecutionResult<Ident> {
    let ident = parse_str::<Ident>(str).map_err(|err| {
        error_source.value_error(format!("`{}` is not a valid ident: {:?}", str, err))
    })?;
    Ok(ident.with_span(span))
}

pub(crate) fn string_to_literal(
    str: &str,
    error_source: &impl HasSpanRange,
    span: Span,
) -> ExecutionResult<Literal> {
    let literal = Literal::from_str(str).map_err(|err| {
        error_source.value_error(format!("`{}` is not a valid literal: {:?}", str, err))
    })?;
    Ok(literal.with_span(span))
}

define_interface! {
    struct StringTypeData,
    parent: IterableTypeData,
    pub(crate) mod string_interface {
        pub(crate) mod methods {
            // ==================
            // CONVERSION METHODS
            // ==================
            [context] fn to_ident(this: Spanned<AnyRef<str>>) -> ExecutionResult<Ident> {
                string_to_ident(&this, &this, context.span_from_join_else_start())
            }

            [context] fn to_ident_camel(this: Spanned<AnyRef<str>>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_camel_case(&this);
                string_to_ident(&str, &this, context.span_from_join_else_start())
            }

            [context] fn to_ident_snake(this: Spanned<AnyRef<str>>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_lower_snake_case(&this);
                string_to_ident(&str, &this, context.span_from_join_else_start())
            }

            [context] fn to_ident_upper_snake(this: Spanned<AnyRef<str>>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_snake_case(&this);
                string_to_ident(&str, &this, context.span_from_join_else_start())
            }

            [context] fn to_literal(this: Spanned<AnyRef<str>>) -> ExecutionResult<Literal> {
                string_to_literal(&this, &this, context.span_from_join_else_start())
            }

            // ======================
            // STRING RESHAPE METHODS
            // ======================
            fn to_uppercase(this: AnyRef<str>) -> String {
                string_conversion::to_uppercase(&this)
            }

            fn to_lowercase(this: AnyRef<str>) -> String {
                string_conversion::to_lowercase(&this)
            }

            fn to_lower_snake_case(this: AnyRef<str>) -> String {
                string_conversion::to_lower_snake_case(&this)
            }

            fn to_upper_snake_case(this: AnyRef<str>) -> String {
                string_conversion::to_upper_snake_case(&this)
            }

            fn to_kebab_case(this: AnyRef<str>) -> String {
                string_conversion::to_lower_kebab_case(&this)
            }

            fn to_lower_camel_case(this: AnyRef<str>) -> String {
                string_conversion::to_lower_camel_case(&this)
            }

            fn to_upper_camel_case(this: AnyRef<str>) -> String {
                string_conversion::to_upper_camel_case(&this)
            }

            fn capitalize(this: AnyRef<str>) -> String {
                string_conversion::capitalize(&this)
            }

            fn decapitalize(this: AnyRef<str>) -> String {
                string_conversion::decapitalize(&this)
            }

            fn to_title_case(this: AnyRef<str>) -> String {
                string_conversion::title_case(&this)
            }

            fn insert_spaces(this: AnyRef<str>) -> String {
                string_conversion::insert_spaces_between_words(&this)
            }
        }
        pub(crate) mod unary_operations {
            fn cast_to_string(this: String) -> String {
                this
            }
        }
        pub(crate) mod binary_operations {
            fn add(mut lhs: String, rhs: Shared<str>) -> String {
                lhs.push_str(rhs.deref());
                lhs
            }

            fn add_assign(mut lhs: Assignee<String>, rhs: Shared<str>) {
                lhs.push_str(rhs.deref());
            }

            fn eq(lhs: Shared<str>, rhs: Shared<str>) -> bool {
                lhs.deref() == rhs.deref()
            }

            fn ne(lhs: Shared<str>, rhs: Shared<str>) -> bool {
                lhs.deref() != rhs.deref()
            }

            fn lt(lhs: Shared<str>, rhs: Shared<str>) -> bool {
                lhs.deref() < rhs.deref()
            }

            fn le(lhs: Shared<str>, rhs: Shared<str>) -> bool {
                lhs.deref() <= rhs.deref()
            }

            fn ge(lhs: Shared<str>, rhs: Shared<str>) -> bool {
                lhs.deref() >= rhs.deref()
            }

            fn gt(lhs: Shared<str>, rhs: Shared<str>) -> bool {
                lhs.deref() > rhs.deref()
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => return None,
                    UnaryOperation::Cast { target: CastTarget(kind), .. } => match kind {
                        ValueLeafKind::String(_) => unary_definitions::cast_to_string(),
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
    StringTypeData,
    (value, context) -> StringValue {
        match value {
            Value::String(value) => Ok(value),
            _ => context.err("a string", value),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    (value: StringValue) -> String { value.value }
);

impl ResolvableArgumentTarget for str {
    type ValueType = StringTypeData;
}

impl ResolvableShared<Value> for str {
    fn resolve_from_ref<'a>(
        value: &'a Value,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a Self> {
        match value {
            Value::String(s) => Ok(s.value.as_str()),
            _ => context.err("a string", value),
        }
    }
}
