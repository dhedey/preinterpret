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
        match operation.operator {
            UnaryOperator::Neg { .. } => operation.unsupported_for_value_type_err("boolean"),
            UnaryOperator::Not { .. } => operation.output(!input),
            UnaryOperator::GroupedNoOp { .. } => operation.output(input),
            UnaryOperator::Cast { target, .. } => match target {
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
                    operation.err("This cast is not supported")
                }
                CastTarget::Boolean => operation.output(self.value),
            },
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: EvaluationInteger,
        operation: BinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match operation.integer_operator() {
            IntegerBinaryOperator::ShiftLeft | IntegerBinaryOperator::ShiftRight => {
                operation.unsupported_for_value_type_err("boolean")
            }
        }
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
            | PairedBinaryOperator::Division => operation.unsupported_for_value_type_err("boolean"),
            PairedBinaryOperator::LogicalAnd => operation.output(lhs && rhs),
            PairedBinaryOperator::LogicalOr => operation.output(lhs || rhs),
            PairedBinaryOperator::Remainder => operation.unsupported_for_value_type_err("boolean"),
            PairedBinaryOperator::BitXor => operation.output(lhs ^ rhs),
            PairedBinaryOperator::BitAnd => operation.output(lhs & rhs),
            PairedBinaryOperator::BitOr => operation.output(lhs | rhs),
            PairedBinaryOperator::Equal => operation.output(lhs == rhs),
            PairedBinaryOperator::LessThan => operation.output(!lhs & rhs),
            PairedBinaryOperator::LessThanOrEqual => operation.output(lhs <= rhs),
            PairedBinaryOperator::NotEqual => operation.output(lhs != rhs),
            PairedBinaryOperator::GreaterThanOrEqual => operation.output(lhs >= rhs),
            PairedBinaryOperator::GreaterThan => operation.output(lhs & !rhs),
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
