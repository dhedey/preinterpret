use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionChar {
    pub(super) value: char,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(super) span_range: SpanRange,
}

impl ExpressionChar {
    pub(super) fn for_litchar(lit: syn::LitChar) -> Self {
        Self {
            value: lit.value(),
            span_range: lit.span().span_range(),
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let char = self.value;
        Ok(match operation.operation {
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => {
                return operation.unsupported(self)
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
                CastTarget::Boolean | CastTarget::Float(_) => return operation.unsupported(self),
            },
        })
    }

    pub(super) fn create_range(
        self,
        right: Self,
        range_limits: OutputSpanned<syn::RangeLimits>,
    ) -> Box<dyn Iterator<Item = ExpressionValue> + '_> {
        let left = self.value;
        let right = right.value;
        match range_limits.operation {
            syn::RangeLimits::HalfOpen { .. } => {
                Box::new((left..right).map(move |x| range_limits.output(x)))
            }
            syn::RangeLimits::Closed { .. } => {
                Box::new((left..=right).map(move |x| range_limits.output(x)))
            }
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
            PairedBinaryOperation::Addition { .. }
            | PairedBinaryOperation::Subtraction { .. }
            | PairedBinaryOperation::Multiplication { .. }
            | PairedBinaryOperation::Division { .. }
            | PairedBinaryOperation::LogicalAnd { .. }
            | PairedBinaryOperation::LogicalOr { .. }
            | PairedBinaryOperation::Remainder { .. }
            | PairedBinaryOperation::BitXor { .. }
            | PairedBinaryOperation::BitAnd { .. }
            | PairedBinaryOperation::BitOr { .. } => return operation.unsupported(self),
            PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
            PairedBinaryOperation::LessThan { .. } => operation.output(lhs < rhs),
            PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
            PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
            PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
            PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs > rhs),
        })
    }

    pub(super) fn to_literal(&self) -> Literal {
        Literal::character(self.value).with_span(self.span_range.join_into_span_else_start())
    }
}

impl HasValueType for ExpressionChar {
    fn value_type(&self) -> &'static str {
        "char"
    }
}

impl ToExpressionValue for char {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Char(ExpressionChar {
            value: self,
            span_range,
        })
    }
}
