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

#[derive(Clone, Copy)]
pub(crate) struct BooleanTypeData;

impl MethodResolutionTarget for BooleanTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Not { .. } => {
                wrap_unary!((this: Owned<bool>) -> ExecutionResult<bool> {
                    Ok(!this.into_inner())
                })
            }
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    wrap_unary!((input: bool) -> UntypedInteger {
                        UntypedInteger::from_fallback(input as FallbackInteger)
                    })
                }
                CastTarget::Integer(IntegerKind::I8) => {
                    wrap_unary!((input: bool) -> i8 {
                        input as i8
                    })
                }
                CastTarget::Integer(IntegerKind::I16) => {
                    wrap_unary!((input: bool) -> i16 {
                        input as i16
                    })
                }
                CastTarget::Integer(IntegerKind::I32) => {
                    wrap_unary!((input: bool) -> i32 {
                        input as i32
                    })
                }
                CastTarget::Integer(IntegerKind::I64) => {
                    wrap_unary!((input: bool) -> i64 {
                        input as i64
                    })
                }
                CastTarget::Integer(IntegerKind::I128) => {
                    wrap_unary!((input: bool) -> i128 {
                        input as i128
                    })
                }
                CastTarget::Integer(IntegerKind::Isize) => {
                    wrap_unary!((input: bool) -> isize {
                        input as isize
                    })
                }
                CastTarget::Integer(IntegerKind::U8) => {
                    wrap_unary!((input: bool) -> u8 {
                        input as u8
                    })
                }
                CastTarget::Integer(IntegerKind::U16) => {
                    wrap_unary!((input: bool) -> u16 {
                        input as u16
                    })
                }
                CastTarget::Integer(IntegerKind::U32) => {
                    wrap_unary!((input: bool) -> u32 {
                        input as u32
                    })
                }
                CastTarget::Integer(IntegerKind::U64) => {
                    wrap_unary!((input: bool) -> u64 {
                        input as u64
                    })
                }
                CastTarget::Integer(IntegerKind::U128) => {
                    wrap_unary!((input: bool) -> u128 {
                        input as u128
                    })
                }
                CastTarget::Integer(IntegerKind::Usize) => {
                    wrap_unary!((input: bool) -> usize {
                        input as usize
                    })
                }
                CastTarget::Boolean => {
                    wrap_unary!((input: bool) -> bool {
                        input
                    })
                }
                CastTarget::String => {
                    wrap_unary!((input: bool) -> String {
                        input.to_string()
                    })
                }
                _ => return None,
            },
            _ => return None,
        })
    }
}
