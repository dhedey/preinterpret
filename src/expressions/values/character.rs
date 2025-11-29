use super::*;

#[derive(Clone)]
pub(crate) struct CharExpression {
    pub(super) value: char,
}

impl ToExpressionValue for CharExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Char(self)
    }
}

impl CharExpression {
    pub(super) fn for_litchar(lit: &syn::LitChar) -> Owned<Self> {
        Self { value: lit.value() }.into_owned(lit.span())
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: IntegerExpression,
        operation: WrappedOp<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: WrappedOp<PairedBinaryOperation>,
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

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        Literal::character(self.value).with_span(span)
    }
}

impl HasValueType for CharExpression {
    fn value_type(&self) -> &'static str {
        "char"
    }
}

impl ToExpressionValue for char {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Char(CharExpression { value: self })
    }
}

define_interface! {
    struct CharTypeData,
    parent: ValueTypeData,
    pub(crate) mod char_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
            fn cast_to_untyped_integer(input: char) -> UntypedInteger {
                UntypedInteger::from_fallback(input as FallbackInteger)
            }

            fn cast_to_i8(input: char) -> i8 {
                input as i8
            }

            fn cast_to_i16(input: char) -> i16 {
                input as i16
            }

            fn cast_to_i32(input: char) -> i32 {
                input as i32
            }

            fn cast_to_i64(input: char) -> i64 {
                input as i64
            }

            fn cast_to_i128(input: char) -> i128 {
                input as i128
            }

            fn cast_to_isize(input: char) -> isize {
                input as isize
            }

            fn cast_to_u8(input: char) -> u8 {
                input as u8
            }

            fn cast_to_u16(input: char) -> u16 {
                input as u16
            }

            fn cast_to_u32(input: char) -> u32 {
                input as u32
            }

            fn cast_to_u64(input: char) -> u64 {
                input as u64
            }

            fn cast_to_u128(input: char) -> u128 {
                input as u128
            }

            fn cast_to_usize(input: char) -> usize {
                input as usize
            }

            fn cast_to_char(input: char) -> char {
                input
            }

            fn cast_to_string(input: char) -> String {
                input.to_string()
            }
        }
        pub(crate) mod binary_operations {}
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::Integer(IntegerKind::Untyped) => unary_definitions::cast_to_untyped_integer(),
                        CastTarget::Integer(IntegerKind::I8) => unary_definitions::cast_to_i8(),
                        CastTarget::Integer(IntegerKind::I16) => unary_definitions::cast_to_i16(),
                        CastTarget::Integer(IntegerKind::I32) => unary_definitions::cast_to_i32(),
                        CastTarget::Integer(IntegerKind::I64) => unary_definitions::cast_to_i64(),
                        CastTarget::Integer(IntegerKind::I128) => unary_definitions::cast_to_i128(),
                        CastTarget::Integer(IntegerKind::Isize) => unary_definitions::cast_to_isize(),
                        CastTarget::Integer(IntegerKind::U8) => unary_definitions::cast_to_u8(),
                        CastTarget::Integer(IntegerKind::U16) => unary_definitions::cast_to_u16(),
                        CastTarget::Integer(IntegerKind::U32) => unary_definitions::cast_to_u32(),
                        CastTarget::Integer(IntegerKind::U64) => unary_definitions::cast_to_u64(),
                        CastTarget::Integer(IntegerKind::U128) => unary_definitions::cast_to_u128(),
                        CastTarget::Integer(IntegerKind::Usize) => unary_definitions::cast_to_usize(),
                        CastTarget::Char => unary_definitions::cast_to_char(),
                        CastTarget::String => unary_definitions::cast_to_string(),
                        _ => return None,
                    },
                    _ => return None,
                })
            }
        }
    }
}

impl_resolvable_argument_for! {
    CharTypeData,
    (value, context) -> CharExpression {
        match value {
            ExpressionValue::Char(value) => Ok(value),
            _ => context.err("char", value),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    CharTypeData,
    (value: CharExpression) -> char { value.value }
);
