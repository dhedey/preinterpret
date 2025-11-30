use super::*;

#[derive(Clone)]
pub(crate) struct StringValue {
    pub(crate) value: String,
}

impl IntoValue for StringValue {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl StringValue {
    pub(super) fn for_litstr(lit: &syn::LitStr) -> Owned<Self> {
        Self { value: lit.value() }.into_owned(lit.span())
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        Literal::string(&self.value).with_span(span)
    }
}

impl HasValueType for StringValue {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

impl HasValueType for String {
    fn value_type(&self) -> &'static str {
        "string"
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

define_interface! {
    struct StringTypeData,
    parent: IterableTypeData,
    pub(crate) mod string_interface {
        pub(crate) mod methods {
            // ==================
            // CONVERSION METHODS
            // ==================
            [context] fn to_ident(this: SpannedAnyRef<str>) -> ExecutionResult<Ident> {
                let str: &str = &this;
                let ident = parse_str::<Ident>(str)
                    .map_err(|err| this.value_error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_ident_camel(this: SpannedAnyRef<str>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_camel_case(&this);
                let ident = parse_str::<Ident>(&str)
                    .map_err(|err| this.value_error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_ident_snake(this: SpannedAnyRef<str>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_lower_snake_case(&this);
                let ident = parse_str::<Ident>(&str)
                    .map_err(|err| this.value_error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_ident_upper_snake(this: SpannedAnyRef<str>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_snake_case(&this);
                let ident = parse_str::<Ident>(&str)
                    .map_err(|err| this.value_error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_literal(this: SpannedAnyRef<str>) -> ExecutionResult<Literal> {
                let str: &str = &this;
                let literal = Literal::from_str(str)
                    .map_err(|err| {
                        this.value_error(format!("`{}` is not a valid literal: {:?}", str, err))
                    })?
                    .with_span(context.span_from_join_else_start());
                Ok(literal)
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
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::String => unary_definitions::cast_to_string(),
                        _ => return None,
                    },
                })
            }

            fn resolve_paired_binary_operation(
                operation: &PairedBinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    PairedBinaryOperation::Addition { .. } => binary_definitions::add(),
                    PairedBinaryOperation::Equal { .. } => binary_definitions::eq(),
                    PairedBinaryOperation::NotEqual { .. } => binary_definitions::ne(),
                    PairedBinaryOperation::LessThan { .. } => binary_definitions::lt(),
                    PairedBinaryOperation::LessThanOrEqual { .. } => binary_definitions::le(),
                    PairedBinaryOperation::GreaterThanOrEqual { .. } => binary_definitions::ge(),
                    PairedBinaryOperation::GreaterThan { .. } => binary_definitions::gt(),
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
            _ => context.err("string", value),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    StringTypeData,
    (value: StringValue) -> String { value.value }
);

impl ResolvableArgumentTarget for str {
    type ValueType = StringTypeData;
}

impl ResolvableArgumentShared for str {
    fn resolve_from_ref<'a>(
        value: &'a Value,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a Self> {
        match value {
            Value::String(s) => Ok(s.value.as_str()),
            _ => context.err("string", value),
        }
    }
}
