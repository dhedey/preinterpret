use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionBoolean {
    pub(crate) value: bool,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(super) span_range: SpanRange,
}

impl ExpressionBoolean {
    pub(super) fn for_litbool(lit: syn::LitBool) -> Self {
        Self {
            span_range: lit.span().span_range(),
            value: lit.value,
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let input = self.value;
        Ok(match operation.operation {
            UnaryOperation::Neg { .. } => return operation.unsupported(self),
            UnaryOperation::Not { .. } => operation.output(!input),
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
                    return operation.execution_err("This cast is not supported")
                }
                CastTarget::Boolean => operation.output(input),
                CastTarget::String => operation.output(input.to_string()),
                CastTarget::DebugString => operation.output(input.to_string()),
                CastTarget::Stream => operation.output(
                    operation
                        .output(input)
                        .into_new_output_stream(Grouping::Flattened, false)?,
                ),
                CastTarget::Group => operation.output(
                    operation
                        .output(input)
                        .into_new_output_stream(Grouping::Grouped, false)?,
                ),
            },
        })
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match operation.operation {
            IntegerBinaryOperation::ShiftLeft { .. }
            | IntegerBinaryOperation::ShiftRight { .. } => operation.unsupported(self),
        }
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        Ok(match operation.operation {
            PairedBinaryOperation::Addition { .. }
            | PairedBinaryOperation::Subtraction { .. }
            | PairedBinaryOperation::Multiplication { .. }
            | PairedBinaryOperation::Division { .. } => return operation.unsupported(self),
            PairedBinaryOperation::LogicalAnd { .. } => operation.output(lhs && rhs),
            PairedBinaryOperation::LogicalOr { .. } => operation.output(lhs || rhs),
            PairedBinaryOperation::Remainder { .. } => return operation.unsupported(self),
            PairedBinaryOperation::BitXor { .. } => operation.output(lhs ^ rhs),
            PairedBinaryOperation::BitAnd { .. } => operation.output(lhs & rhs),
            PairedBinaryOperation::BitOr { .. } => operation.output(lhs | rhs),
            PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
            PairedBinaryOperation::LessThan { .. } => operation.output(!lhs & rhs),
            PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
            PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
            PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
            PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs & !rhs),
        })
    }

    pub(super) fn to_ident(&self) -> Ident {
        Ident::new_bool(self.value, self.span_range.join_into_span_else_start())
    }
}

impl HasValueType for ExpressionBoolean {
    fn value_type(&self) -> &'static str {
        "bool"
    }
}

impl ToExpressionValue for bool {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Boolean(ExpressionBoolean {
            value: self,
            span_range,
        })
    }
}
