#![allow(clippy::bool_comparison)]

use super::*;

#[derive(Clone)]
pub(crate) struct BooleanValue {
    pub(crate) value: bool,
}

impl IntoValue for BooleanValue {
    fn into_value(self) -> Value {
        Value::Boolean(self)
    }
}

impl Debug for BooleanValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl BooleanValue {
    pub(crate) fn for_litbool(lit: &syn::LitBool) -> Owned<Self> {
        Self { value: lit.value }.into_owned()
    }

    pub(super) fn to_ident(&self, span: Span) -> Ident {
        Ident::new_bool(self.value, span)
    }
}

impl HasValueKind for BooleanValue {
    type SpecificKind = ValueKind;

    fn kind(&self) -> ValueKind {
        ValueKind::Boolean
    }
}

impl ValuesEqual for BooleanValue {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.value == other.value {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Boolean(BooleanValue { value: self })
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
        pub(crate) mod binary_operations {
            fn and(lhs: bool, rhs: bool) -> bool {
                lhs && rhs
            }

            fn or(lhs: bool, rhs: bool) -> bool {
                lhs || rhs
            }

            fn bitxor(lhs: bool, rhs: bool) -> bool {
                lhs ^ rhs
            }

            fn bitand(lhs: bool, rhs: bool) -> bool {
                lhs & rhs
            }

            fn bitor(lhs: bool, rhs: bool) -> bool {
                lhs | rhs
            }

            fn eq(lhs: bool, rhs: bool) -> bool {
                lhs == rhs
            }

            fn ne(lhs: bool, rhs: bool) -> bool {
                lhs != rhs
            }

            fn lt(lhs: bool, rhs: bool) -> bool {
                lhs < rhs
            }

            fn le(lhs: bool, rhs: bool) -> bool {
                lhs <= rhs
            }

            fn ge(lhs: bool, rhs: bool) -> bool {
                lhs >= rhs
            }

            fn gt(lhs: bool, rhs: bool) -> bool {
                lhs > rhs
            }
        }
        interface_items {
            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    BinaryOperation::LogicalAnd { .. } => binary_definitions::and(),
                    BinaryOperation::LogicalOr { .. } => binary_definitions::or(),
                    BinaryOperation::BitXor { .. } => binary_definitions::bitxor(),
                    BinaryOperation::BitAnd { .. } => binary_definitions::bitand(),
                    BinaryOperation::BitOr { .. } => binary_definitions::bitor(),
                    BinaryOperation::Equal { .. } => binary_definitions::eq(),
                    BinaryOperation::NotEqual { .. } => binary_definitions::ne(),
                    BinaryOperation::LessThan { .. } => binary_definitions::lt(),
                    BinaryOperation::LessThanOrEqual { .. } => binary_definitions::le(),
                    BinaryOperation::GreaterThanOrEqual { .. } => binary_definitions::ge(),
                    BinaryOperation::GreaterThan { .. } => binary_definitions::gt(),
                    _ => return None,
                })
            }
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
    (value, context) -> BooleanValue {
        match value {
            Value::Boolean(value) => Ok(value),
            other => context.err("a boolean", other),
        }
    }
}

impl_delegated_resolvable_argument_for! {
    (value: BooleanValue) -> bool { value.value }
}
