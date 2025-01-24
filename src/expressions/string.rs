use super::*;

pub(crate) struct EvaluationString {
    pub(super) value: String,
    pub(super) source_span: SpanRange,
}

impl EvaluationString {
    pub(super) fn new(value: String, source_span: SpanRange) -> Self {
        Self { value, source_span }
    }

    pub(super) fn for_litstr(lit: &syn::LitStr) -> Self {
        Self {
            value: lit.value(),
            source_span: lit.span().span_range(),
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
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
    ) -> ExecutionResult<EvaluationOutput> {
        operation.unsupported_for_value_type_err("string")
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &BinaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
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

    pub(super) fn to_literal(&self) -> Literal {
        Literal::string(&self.value).with_span(self.source_span.start())
    }
}

impl ToEvaluationOutput for String {
    fn to_output(self, span: SpanRange) -> EvaluationOutput {
        EvaluationValue::String(EvaluationString::new(self, span)).into()
    }
}

impl ToEvaluationOutput for &str {
    fn to_output(self, span: SpanRange) -> EvaluationOutput {
        EvaluationValue::String(EvaluationString::new(self.to_string(), span)).into()
    }
}

impl quote::ToTokens for EvaluationString {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_literal().to_tokens(tokens)
    }
}
