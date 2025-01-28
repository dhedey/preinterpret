use super::*;

pub(crate) struct EvaluationString {
    pub(super) value: String,
    /// The span of the source code that generated this boolean value.
    /// It may not have a value if generated from a complex expression.
    pub(super) source_span: Option<Span>,
}

impl EvaluationString {
    pub(super) fn for_litstr(lit: &syn::LitStr) -> Self {
        Self {
            value: lit.value(),
            source_span: Some(lit.span()),
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match operation.operator {
            UnaryOperator::NoOp => operation.output(self.value),
            UnaryOperator::Neg | UnaryOperator::Not | UnaryOperator::Cast(_) => {
                operation.unsupported_for_value_type_err("string")
            }
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: EvaluationInteger,
        operation: BinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        operation.unsupported_for_value_type_err("string")
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &BinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        match operation.paired_operator() {
            PairedBinaryOperator::Addition
            | PairedBinaryOperator::Subtraction
            | PairedBinaryOperator::Multiplication
            | PairedBinaryOperator::Division
            | PairedBinaryOperator::LogicalAnd
            | PairedBinaryOperator::LogicalOr
            | PairedBinaryOperator::Remainder
            | PairedBinaryOperator::BitXor
            | PairedBinaryOperator::BitAnd
            | PairedBinaryOperator::BitOr => operation.unsupported_for_value_type_err("string"),
            PairedBinaryOperator::Equal => operation.output(lhs == rhs),
            PairedBinaryOperator::LessThan => operation.output(lhs < rhs),
            PairedBinaryOperator::LessThanOrEqual => operation.output(lhs <= rhs),
            PairedBinaryOperator::NotEqual => operation.output(lhs != rhs),
            PairedBinaryOperator::GreaterThanOrEqual => operation.output(lhs >= rhs),
            PairedBinaryOperator::GreaterThan => operation.output(lhs > rhs),
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
