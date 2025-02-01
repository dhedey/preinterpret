use super::*;

#[derive(Clone)]
pub(crate) struct EvaluationString {
    pub(super) value: String,
    /// The span of the source code that generated this boolean value.
    /// It may not have a value if generated from a complex expression.
    pub(super) source_span: Option<Span>,
}

impl EvaluationString {
    pub(super) fn for_litstr(lit: syn::LitStr) -> Self {
        Self {
            value: lit.value(),
            source_span: Some(lit.span()),
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match operation {
            UnaryOperation::GroupedNoOp { .. } => operation.output(self.value),
            UnaryOperation::Neg { .. }
            | UnaryOperation::Not { .. }
            | UnaryOperation::Cast { .. } => operation.unsupported_for_value_type_err("string"),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: EvaluationInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        operation.unsupported_for_value_type_err("string")
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &PairedBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        match operation {
            PairedBinaryOperation::Addition { .. }
            | PairedBinaryOperation::Subtraction { .. }
            | PairedBinaryOperation::Multiplication { .. }
            | PairedBinaryOperation::Division { .. }
            | PairedBinaryOperation::LogicalAnd { .. }
            | PairedBinaryOperation::LogicalOr { .. }
            | PairedBinaryOperation::Remainder { .. }
            | PairedBinaryOperation::BitXor { .. }
            | PairedBinaryOperation::BitAnd { .. }
            | PairedBinaryOperation::BitOr { .. } => {
                operation.unsupported_for_value_type_err("string")
            }
            PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
            PairedBinaryOperation::LessThan { .. } => operation.output(lhs < rhs),
            PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
            PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
            PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
            PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs > rhs),
        }
    }

    pub(super) fn to_literal(&self, fallback_span: Span) -> Literal {
        Literal::string(&self.value).with_span(self.source_span.unwrap_or(fallback_span))
    }
}

impl ToEvaluationValue for String {
    fn to_value(self, source_span: Option<Span>) -> EvaluationValue {
        EvaluationValue::String(EvaluationString {
            value: self,
            source_span,
        })
    }
}

impl ToEvaluationValue for &str {
    fn to_value(self, source_span: Option<Span>) -> EvaluationValue {
        EvaluationValue::String(EvaluationString {
            value: self.to_string(),
            source_span,
        })
    }
}
