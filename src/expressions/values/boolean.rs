#![allow(clippy::bool_comparison)]

use super::*;

define_leaf_type! {
    pub(crate) BoolType => ValueType(ValueContent::Bool),
    content: bool,
    kind: pub(crate) BoolKind,
    type_name: "bool",
    articled_display_name: "a bool",
    dyn_impls: {},
}

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

impl HasLeafKind for BooleanValue {
    type LeafKind = BoolKind;

    fn kind(&self) -> Self::LeafKind {
        BoolKind
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

define_type_features! {
    impl BoolType,
    parent: ValueType,
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
                    UnaryOperation::Cast { target: CastTarget(kind), .. } => match kind {
                        ValueLeafKind::Integer(IntegerLeafKind::Untyped(_)) => unary_definitions::cast_to_untyped_integer(),
                        ValueLeafKind::Integer(IntegerLeafKind::I8(_)) => unary_definitions::cast_to_i8(),
                        ValueLeafKind::Integer(IntegerLeafKind::I16(_)) => unary_definitions::cast_to_i16(),
                        ValueLeafKind::Integer(IntegerLeafKind::I32(_)) => unary_definitions::cast_to_i32(),
                        ValueLeafKind::Integer(IntegerLeafKind::I64(_)) => unary_definitions::cast_to_i64(),
                        ValueLeafKind::Integer(IntegerLeafKind::I128(_)) => unary_definitions::cast_to_i128(),
                        ValueLeafKind::Integer(IntegerLeafKind::Isize(_)) => unary_definitions::cast_to_isize(),
                        ValueLeafKind::Integer(IntegerLeafKind::U8(_)) => unary_definitions::cast_to_u8(),
                        ValueLeafKind::Integer(IntegerLeafKind::U16(_)) => unary_definitions::cast_to_u16(),
                        ValueLeafKind::Integer(IntegerLeafKind::U32(_)) => unary_definitions::cast_to_u32(),
                        ValueLeafKind::Integer(IntegerLeafKind::U64(_)) => unary_definitions::cast_to_u64(),
                        ValueLeafKind::Integer(IntegerLeafKind::U128(_)) => unary_definitions::cast_to_u128(),
                        ValueLeafKind::Integer(IntegerLeafKind::Usize(_)) => unary_definitions::cast_to_usize(),
                        ValueLeafKind::Bool(_) => unary_definitions::cast_to_boolean(),
                        ValueLeafKind::String(_) => unary_definitions::cast_to_string(),
                        _ => return None,
                    },
                    _ => return None,
                })
            }
        }
    }
}

impl_resolvable_argument_for! {
    BoolType,
    (value, context) -> BooleanValue {
        match value {
            Value::Boolean(value) => Ok(value),
            other => context.err("a bool", other),
        }
    }
}

impl_delegated_resolvable_argument_for! {
    (value: BooleanValue) -> bool { value.value }
}
