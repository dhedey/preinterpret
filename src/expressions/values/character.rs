use super::*;

define_leaf_type! {
    pub(crate) CharType => AnyType(AnyValueContent::Char),
    content: char,
    kind: pub(crate) CharKind,
    type_name: "char",
    articled_display_name: "a char",
    dyn_impls: {},
}

impl ValuesEqual for char {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self == other {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

define_type_features! {
    impl CharType,
    pub(crate) mod char_interface {
        unary_operations {
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
        binary_operations {
            fn eq(lhs: char, rhs: char) -> bool {
                lhs == rhs
            }

            fn ne(lhs: char, rhs: char) -> bool {
                lhs != rhs
            }

            fn lt(lhs: char, rhs: char) -> bool {
                lhs < rhs
            }

            fn le(lhs: char, rhs: char) -> bool {
                lhs <= rhs
            }

            fn ge(lhs: char, rhs: char) -> bool {
                lhs >= rhs
            }

            fn gt(lhs: char, rhs: char) -> bool {
                lhs > rhs
            }
        }
        interface_items {
            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
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
                    UnaryOperation::Cast { target: CastTarget(kind), .. } => match kind {
                        AnyValueLeafKind::Integer(IntegerLeafKind::Untyped(_)) => unary_definitions::cast_to_untyped_integer(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::I8(_)) => unary_definitions::cast_to_i8(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::I16(_)) => unary_definitions::cast_to_i16(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::I32(_)) => unary_definitions::cast_to_i32(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::I64(_)) => unary_definitions::cast_to_i64(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::I128(_)) => unary_definitions::cast_to_i128(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::Isize(_)) => unary_definitions::cast_to_isize(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::U8(_)) => unary_definitions::cast_to_u8(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::U16(_)) => unary_definitions::cast_to_u16(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::U32(_)) => unary_definitions::cast_to_u32(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::U64(_)) => unary_definitions::cast_to_u64(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::U128(_)) => unary_definitions::cast_to_u128(),
                        AnyValueLeafKind::Integer(IntegerLeafKind::Usize(_)) => unary_definitions::cast_to_usize(),
                        AnyValueLeafKind::Char(_) => unary_definitions::cast_to_char(),
                        AnyValueLeafKind::String(_) => unary_definitions::cast_to_string(),
                        _ => return None,
                    },
                    _ => return None,
                })
            }
        }
    }
}

impl_resolvable_argument_for! {
    CharType,
    (value, context) -> char {
        match value {
            AnyValueContent::Char(char) => Ok(char),
            _ => context.err("a char", value),
        }
    }
}
