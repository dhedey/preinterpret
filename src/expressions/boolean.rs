use super::*;

#[derive(Clone)]
pub(crate) struct EvaluationBoolean {
    pub(super) value: bool,
    /// The span of the source code that generated this boolean value.
    /// It may not have a value if generated from a complex expression.
    pub(super) source_span: Option<Span>,
}

impl EvaluationBoolean {
    pub(super) fn for_litbool(lit: syn::LitBool) -> Self {
        Self {
            source_span: Some(lit.span()),
            value: lit.value,
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        let input = self.value;
        match operation {
            UnaryOperation::Neg { .. } => operation.unsupported_for_value_type_err("boolean"),
            UnaryOperation::Not { .. } => operation.output(!input),
            UnaryOperation::GroupedNoOp { .. } => operation.output(input),
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    operation.output(UntypedInteger::from_fallback(input as FallbackInteger))
                }
                CastTarget::Integer(IntegerKind::I8) => operation.output(input as i8),
                CastTarget::Integer(IntegerKind::I16) => operation.output(input as i16),
                CastTarget::Integer(IntegerKind::I32) => operation.output(input as i32),
                CastTarget::Integer(IntegerKind::I64) => operation.output(input as i64),
                CastTarget::Integer(IntegerKind::I128) => operation.output(input as i128),
                CastTarget::Integer(IntegerKind::Isize) => operation.output(input as isize),
                CastTarget::Integer(IntegerKind::U8) => operation.output(input as u8),
                CastTarget::Integer(IntegerKind::U16) => operation.output(input as u16),
                CastTarget::Integer(IntegerKind::U32) => operation.output(input as u32),
                CastTarget::Integer(IntegerKind::U64) => operation.output(input as u64),
                CastTarget::Integer(IntegerKind::U128) => operation.output(input as u128),
                CastTarget::Integer(IntegerKind::Usize) => operation.output(input as usize),
                CastTarget::Float(_) | CastTarget::Char => {
                    operation.execution_err("This cast is not supported")
                }
                CastTarget::Boolean => operation.output(self.value),
            },
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: EvaluationInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match operation {
            IntegerBinaryOperation::ShiftLeft { .. }
            | IntegerBinaryOperation::ShiftRight { .. } => {
                operation.unsupported_for_value_type_err("boolean")
            }
        }
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
            | PairedBinaryOperation::Division { .. } => {
                operation.unsupported_for_value_type_err("boolean")
            }
            PairedBinaryOperation::LogicalAnd { .. } => operation.output(lhs && rhs),
            PairedBinaryOperation::LogicalOr { .. } => operation.output(lhs || rhs),
            PairedBinaryOperation::Remainder { .. } => {
                operation.unsupported_for_value_type_err("boolean")
            }
            PairedBinaryOperation::BitXor { .. } => operation.output(lhs ^ rhs),
            PairedBinaryOperation::BitAnd { .. } => operation.output(lhs & rhs),
            PairedBinaryOperation::BitOr { .. } => operation.output(lhs | rhs),
            PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
            PairedBinaryOperation::LessThan { .. } => operation.output(!lhs & rhs),
            PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
            PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
            PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
            PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs & !rhs),
        }
    }

    pub(super) fn to_ident(&self, fallback_span: Span) -> Ident {
        Ident::new_bool(self.value, self.source_span.unwrap_or(fallback_span))
    }
}

impl ToEvaluationValue for bool {
    fn to_value(self, source_span: Option<Span>) -> EvaluationValue {
        EvaluationValue::Boolean(EvaluationBoolean {
            value: self,
            source_span,
        })
    }
}
