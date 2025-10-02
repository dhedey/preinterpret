use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionString {
    pub(crate) value: String,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(super) span_range: SpanRange,
}

impl ExpressionString {
    pub(super) fn for_litstr(lit: syn::LitStr) -> Self {
        Self {
            value: lit.value(),
            span_range: lit.span().span_range(),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        Ok(match operation.operation {
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

    pub(super) fn to_literal(&self) -> Literal {
        Literal::string(&self.value).with_span(self.span_range.join_into_span_else_start())
    }
}

impl HasValueType for ExpressionString {
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
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::String(ExpressionString {
            value: self,
            span_range,
        })
    }
}

impl ToExpressionValue for &str {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::String(ExpressionString {
            value: self.to_string(),
            span_range,
        })
    }
}

define_interface! {
    struct StringTypeData,
    parent: ValueTypeData,
    pub(crate) mod string_interface {
        pub(crate) mod methods {
            // ==================
            // CONVERSION METHODS
            // ==================
            [context] fn to_ident(this: SpannedRef<str>) -> ExecutionResult<Ident> {
                let str = &*this;
                let ident = parse_str::<Ident>(str)
                    .map_err(|err| this.error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_ident_camel(this: SpannedRef<str>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_camel_case(&this);
                let ident = parse_str::<Ident>(&str)
                    .map_err(|err| this.error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_ident_snake(this: SpannedRef<str>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_lower_snake_case(&this);
                let ident = parse_str::<Ident>(&str)
                    .map_err(|err| this.error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_ident_upper_snake(this: SpannedRef<str>) -> ExecutionResult<Ident> {
                let str = string_conversion::to_upper_snake_case(&this);
                let ident = parse_str::<Ident>(&str)
                    .map_err(|err| this.error(format!("`{}` is not a valid ident: {:?}", str, err)))?
                    .with_span(context.span_from_join_else_start());
                Ok(ident)
            }

            [context] fn to_literal(this: SpannedRef<str>) -> ExecutionResult<Literal> {
                let str = &*this;
                let literal = Literal::from_str(str)
                    .map_err(|err| {
                        this.error(format!("`{}` is not a valid literal: {:?}", str, err))
                    })?
                    .with_span(context.span_from_join_else_start());
                Ok(literal)
            }

            // ======================
            // STRING RESHAPE METHODS
            // ======================
            fn to_uppercase(this: Ref<str>) -> String {
                string_conversion::to_uppercase(&this)
            }

            fn to_lowercase(this: Ref<str>) -> String {
                string_conversion::to_lowercase(&this)
            }

            fn to_lower_snake_case(this: Ref<str>) -> String {
                string_conversion::to_lower_snake_case(&this)
            }

            fn to_upper_snake_case(this: Ref<str>) -> String {
                string_conversion::to_upper_snake_case(&this)
            }

            fn to_kebab_case(this: Ref<str>) -> String {
                string_conversion::to_lower_kebab_case(&this)
            }

            fn to_lower_camel_case(this: Ref<str>) -> String {
                string_conversion::to_lower_camel_case(&this)
            }

            fn to_upper_camel_case(this: Ref<str>) -> String {
                string_conversion::to_upper_camel_case(&this)
            }

            fn capitalize(this: Ref<str>) -> String {
                string_conversion::capitalize(&this)
            }

            fn decapitalize(this: Ref<str>) -> String {
                string_conversion::decapitalize(&this)
            }

            fn to_title_case(this: Ref<str>) -> String {
                string_conversion::title_case(&this)
            }

            fn insert_spaces(this: Ref<str>) -> String {
                string_conversion::insert_spaces_between_words(&this)
            }
        }
        pub(crate) mod unary_operations {
            fn cast_to_string(this: String) -> String {
                this
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
        }
    }
}
