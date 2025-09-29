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

#[derive(Clone, Copy)]
pub(crate) struct StringTypeData;

impl MethodResolutionTarget for StringTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => return None,
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::String => {
                    wrap_unary!((this: String) -> String {
                        this
                    })
                }
                _ => return None,
            },
        })
    }
}
