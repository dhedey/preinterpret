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

#[derive(Clone, Copy)]
pub(crate) struct CharTypeData;

impl MethodResolutionTarget for CharTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    wrap_unary!((input: char) -> UntypedInteger {
                        UntypedInteger::from_fallback(input as FallbackInteger)
                    })
                }
                CastTarget::Integer(IntegerKind::I8) => {
                    wrap_unary!((input: char) -> i8 {
                        input as i8
                    })
                }
                CastTarget::Integer(IntegerKind::I16) => {
                    wrap_unary!((input: char) -> i16 {
                        input as i16
                    })
                }
                CastTarget::Integer(IntegerKind::I32) => {
                    wrap_unary!((input: char) -> i32 {
                        input as i32
                    })
                }
                CastTarget::Integer(IntegerKind::I64) => {
                    wrap_unary!((input: char) -> i64 {
                        input as i64
                    })
                }
                CastTarget::Integer(IntegerKind::I128) => {
                    wrap_unary!((input: char) -> i128 {
                        input as i128
                    })
                }
                CastTarget::Integer(IntegerKind::Isize) => {
                    wrap_unary!((input: char) -> isize {
                        input as isize
                    })
                }
                CastTarget::Integer(IntegerKind::U8) => {
                    wrap_unary!((input: char) -> u8 {
                        input as u8
                    })
                }
                CastTarget::Integer(IntegerKind::U16) => {
                    wrap_unary!((input: char) -> u16 {
                        input as u16
                    })
                }
                CastTarget::Integer(IntegerKind::U32) => {
                    wrap_unary!((input: char) -> u32 {
                        input as u32
                    })
                }
                CastTarget::Integer(IntegerKind::U64) => {
                    wrap_unary!((input: char) -> u64 {
                        input as u64
                    })
                }
                CastTarget::Integer(IntegerKind::U128) => {
                    wrap_unary!((input: char) -> u128 {
                        input as u128
                    })
                }
                CastTarget::Integer(IntegerKind::Usize) => {
                    wrap_unary!((input: char) -> usize {
                        input as usize
                    })
                }
                CastTarget::Char => {
                    wrap_unary!((input: char) -> char {
                        input
                    })
                }
                CastTarget::String => {
                    wrap_unary!((input: char) -> String {
                        input.to_string()
                    })
                }
                _ => return None,
            },
            _ => return None,
        })
    }
}
