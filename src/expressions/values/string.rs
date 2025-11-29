use super::*;

#[derive(Clone)]
pub(crate) struct StringExpression {
    pub(crate) value: String,
}

impl ToExpressionValue for StringExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::String(self)
    }
}

impl StringExpression {
    pub(super) fn for_litstr(lit: &syn::LitStr) -> Owned<Self> {
        Self { value: lit.value() }.into_owned(lit.span())
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: IntegerExpression,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &PairedBinaryOperation,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        Ok(match operation {
            PairedBinaryOperation::Addition { .. } => operation.output(lhs + &rhs),
            PairedBinaryOperation::Subtraction { .. }
            | PairedBinaryOperation::Multiplication { .. }
            | PairedBinaryOperation::Division { .. }
            | PairedBinaryOperation::LogicalAnd { .. }
            | PairedBinaryOperation::LogicalOr { .. }
            | PairedBinaryOperation::Remainder { .. }
            | PairedBinaryOperation::BitXor { .. }
            | PairedBinaryOperation::BitAnd { .. }
            | PairedBinaryOperation::BitOr { .. } => return operation.unsupported(lhs),
            PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
            PairedBinaryOperation::LessThan { .. } => operation.output(lhs < rhs),
            PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
            PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
            PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
            PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs > rhs),
        })
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        Literal::string(&self.value).with_span(span)
    }
}

impl HasValueType for StringExpression {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

impl HasValueType for String {
    fn value_type(&self) -> &'static str {
        "string"
    }
}

impl ToExpressionValue for String {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::String(StringExpression { value: self })
    }
}

impl ToExpressionValue for &str {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::String(StringExpression {
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
        pub(crate) mod binary_operations {}
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
        }
    }
}

impl_resolvable_argument_for! {
    StringTypeData,
    (value, context) -> StringExpression {
        match value {
            ExpressionValue::String(value) => Ok(value),
            _ => context.err("string", value),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    StringTypeData,
    (value: StringExpression) -> String { value.value }
);

impl ResolvableArgumentTarget for str {
    type ValueType = StringTypeData;
}

impl ResolvableArgumentShared for str {
    fn resolve_from_ref<'a>(
        value: &'a ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<&'a Self> {
        match value {
            ExpressionValue::String(s) => Ok(s.value.as_str()),
            _ => context.err("string", value),
        }
    }
}
