use super::*;

#[derive(Clone)]
pub(crate) struct EvaluationChar {
    pub(super) value: char,
    /// The span of the source code that generated this boolean value.
    /// It may not have a value if generated from a complex expression.
    pub(super) source_span: Option<Span>,
}

impl EvaluationChar {
    pub(super) fn for_litchar(lit: syn::LitChar) -> Self {
        Self {
            value: lit.value(),
            source_span: Some(lit.span()),
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        let char = self.value;
        match operation {
            UnaryOperation::GroupedNoOp { .. } => operation.output(char),
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => {
                operation.unsupported_for_value_type_err("char")
            }
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    operation.output(UntypedInteger::from_fallback(char as FallbackInteger))
                }
                CastTarget::Integer(IntegerKind::I8) => operation.output(char as i8),
                CastTarget::Integer(IntegerKind::I16) => operation.output(char as i16),
                CastTarget::Integer(IntegerKind::I32) => operation.output(char as i32),
                CastTarget::Integer(IntegerKind::I64) => operation.output(char as i64),
                CastTarget::Integer(IntegerKind::I128) => operation.output(char as i128),
                CastTarget::Integer(IntegerKind::Isize) => operation.output(char as isize),
                CastTarget::Integer(IntegerKind::U8) => operation.output(char as u8),
                CastTarget::Integer(IntegerKind::U16) => operation.output(char as u16),
                CastTarget::Integer(IntegerKind::U32) => operation.output(char as u32),
                CastTarget::Integer(IntegerKind::U64) => operation.output(char as u64),
                CastTarget::Integer(IntegerKind::U128) => operation.output(char as u128),
                CastTarget::Integer(IntegerKind::Usize) => operation.output(char as usize),
                CastTarget::Char => operation.output(char),
                CastTarget::Boolean | CastTarget::Float(_) => {
                    operation.unsupported_for_value_type_err("char")
                }
            },
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: EvaluationInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        operation.unsupported_for_value_type_err("char")
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
                operation.unsupported_for_value_type_err("char")
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
        Literal::character(self.value).with_span(self.source_span.unwrap_or(fallback_span))
    }
}

impl ToEvaluationValue for char {
    fn to_value(self, source_span: Option<Span>) -> EvaluationValue {
        EvaluationValue::Char(EvaluationChar {
            value: self,
            source_span,
        })
    }
}
