use super::*;

#[derive(Clone)]
pub(crate) struct BooleanExpression {
    pub(crate) value: bool,
}

impl ToExpressionValue for BooleanExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Boolean(self)
    }
}

impl BooleanExpression {
    pub(crate) fn for_litbool(lit: &syn::LitBool) -> Owned<Self> {
        Self { value: lit.value }.into_owned(lit.span)
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: IntegerExpression,
        operation: WrappedOp<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match operation.operation {
            IntegerBinaryOperation::ShiftLeft { .. }
            | IntegerBinaryOperation::ShiftRight { .. } => operation.unsupported(self),
        }
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

    pub(super) fn to_ident(&self, span: Span) -> Ident {
        Ident::new_bool(self.value, span)
    }
}

impl HasValueType for BooleanExpression {
    fn value_type(&self) -> &'static str {
        "bool"
    }
}

impl ToExpressionValue for bool {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Boolean(BooleanExpression { value: self })
    }
}

define_interface! {
    struct BooleanTypeData,
    parent: ValueTypeData,
    pub(crate) mod boolean_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
            fn not(this: bool) -> bool {
                !this
            }

            fn cast_to_untyped_integer(input: bool) -> UntypedInteger {
                UntypedInteger::from_fallback(input as FallbackInteger)
            }

            fn cast_to_i8(input: bool) -> i8 {
                input as i8
            }

            fn cast_to_i16(input: bool) -> i16 {
                input as i16
            }

            fn cast_to_i32(input: bool) -> i32 {
                input as i32
            }

            fn cast_to_i64(input: bool) -> i64 {
                input as i64
            }

            fn cast_to_i128(input: bool) -> i128 {
                input as i128
            }

            fn cast_to_isize(input: bool) -> isize {
                input as isize
            }

            fn cast_to_u8(input: bool) -> u8 {
                input as u8
            }

            fn cast_to_u16(input: bool) -> u16 {
                input as u16
            }

            fn cast_to_u32(input: bool) -> u32 {
                input as u32
            }

            fn cast_to_u64(input: bool) -> u64 {
                input as u64
            }

            fn cast_to_u128(input: bool) -> u128 {
                input as u128
            }

            fn cast_to_usize(input: bool) -> usize {
                input as usize
            }

            fn cast_to_boolean(input: bool) -> bool {
                input
            }

            fn cast_to_string(input: bool) -> String {
                input.to_string()
            }
        }
        pub(crate) mod binary_operations {}
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Not { .. } => unary_definitions::not(),
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
                        CastTarget::Boolean => unary_definitions::cast_to_boolean(),
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
    BooleanTypeData,
    (value, context) -> BooleanExpression {
        match value {
            ExpressionValue::Boolean(value) => Ok(value),
            other => context.err("boolean", other),
        }
    }
}

impl_delegated_resolvable_argument_for! {
    BooleanTypeData,
    (value: BooleanExpression) -> bool { value.value }
}
