use super::*;

pub(crate) struct EvaluationBoolean {
    pub(super) source_span: SpanRange,
    pub(super) value: bool,
}

impl EvaluationBoolean {
    pub(super) fn new(value: bool, source_span: SpanRange) -> Self {
        Self { value, source_span }
    }

    pub(crate) fn value(&self) -> bool {
        self.value
    }

    pub(super) fn for_litbool(lit: &syn::LitBool) -> Self {
        Self {
            source_span: lit.span().span_range(),
            value: lit.value,
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
        let input = self.value;
        match operation.operator {
            UnaryOperator::Neg => operation.unsupported_for_value_type_err("boolean"),
            UnaryOperator::Not => operation.output(!input),
            UnaryOperator::NoOp => operation.output(input),
            UnaryOperator::Cast(target) => match target {
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
    ) -> ExecutionResult<EvaluationOutput> {
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
    ) -> ExecutionResult<EvaluationOutput> {
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

    pub(super) fn to_ident(&self) -> Ident {
        Ident::new_bool(self.value, self.source_span.start())
    }

    #[allow(unused)]
    pub(super) fn to_lit_bool(&self) -> LitBool {
        LitBool::new(self.value, self.source_span.span())
    }
}

impl ToEvaluationOutput for bool {
    fn to_output(self, span: SpanRange) -> EvaluationOutput {
        EvaluationValue::Boolean(EvaluationBoolean::new(self, span)).into()
    }
}

impl quote::ToTokens for EvaluationBoolean {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_ident().to_tokens(tokens)
    }
}
