use super::*;

macro_rules! impl_float_operations {
    (
        $($type_data:ident mod $mod_name:ident: $float_enum_variant:ident($float_type:ident)),* $(,)?
    ) => {$(
        define_interface! {
            struct $type_data,
            parent: FloatTypeData,
            pub(crate) mod $mod_name {
                pub(crate) mod methods {
                }
                pub(crate) mod unary_operations {
                    fn neg(input: $float_type) -> $float_type {
                        -input
                    }

                    fn cast_to_untyped_integer(input: $float_type) -> UntypedInteger {
                        UntypedInteger::from_fallback(input as FallbackInteger)
                    }

                    fn cast_to_i8(input: $float_type) -> i8 {
                        input as i8
                    }

                    fn cast_to_i16(input: $float_type) -> i16 {
                        input as i16
                    }

                    fn cast_to_i32(input: $float_type) -> i32 {
                        input as i32
                    }

                    fn cast_to_i64(input: $float_type) -> i64 {
                        input as i64
                    }

                    fn cast_to_i128(input: $float_type) -> i128 {
                        input as i128
                    }

                    fn cast_to_isize(input: $float_type) -> isize {
                        input as isize
                    }

                    fn cast_to_u8(input: $float_type) -> u8 {
                        input as u8
                    }

                    fn cast_to_u16(input: $float_type) -> u16 {
                        input as u16
                    }

                    fn cast_to_u32(input: $float_type) -> u32 {
                        input as u32
                    }

                    fn cast_to_u64(input: $float_type) -> u64 {
                        input as u64
                    }

                    fn cast_to_u128(input: $float_type) -> u128 {
                        input as u128
                    }

                    fn cast_to_usize(input: $float_type) -> usize {
                        input as usize
                    }

                    fn cast_to_untyped_float(input: $float_type) -> UntypedFloat {
                        UntypedFloat::from_fallback(input as FallbackFloat)
                    }

                    fn cast_to_f32(input: $float_type) -> f32 {
                        input as f32
                    }

                    fn cast_to_f64(input: $float_type) -> f64 {
                        input as f64
                    }

                    fn cast_to_string(input: $float_type) -> String {
                        input.to_string()
                    }
                }
                pub(crate) mod binary_operations {
                    [context] fn add(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a + b))
                    }

                    [context] fn sub(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a - b))
                    }

                    [context] fn mul(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a * b))
                    }

                    [context] fn div(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a / b))
                    }

                    [context] fn rem(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a % b))
                    }

                    fn eq(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs == rhs
                    }

                    fn ne(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs != rhs
                    }

                    fn lt(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs < rhs
                    }

                    fn le(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs <= rhs
                    }

                    fn ge(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs >= rhs
                    }

                    fn gt(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs > rhs
                    }
                }
                interface_items {
                    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                        Some(match operation {
                            UnaryOperation::Neg { .. } => unary_definitions::neg(),
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
                                CastTarget::Float(FloatKind::Untyped) => unary_definitions::cast_to_untyped_float(),
                                CastTarget::Float(FloatKind::F32) => unary_definitions::cast_to_f32(),
                                CastTarget::Float(FloatKind::F64) => unary_definitions::cast_to_f64(),
                                CastTarget::String => unary_definitions::cast_to_string(),
                                _ => return None,
                            }
                            _ => return None,
                        })
                    }

                    fn resolve_paired_binary_operation(
                        operation: &PairedBinaryOperation,
                    ) -> Option<BinaryOperationInterface> {
                        Some(match operation {
                            PairedBinaryOperation::Addition { .. } => binary_definitions::add(),
                            PairedBinaryOperation::Subtraction { .. } => binary_definitions::sub(),
                            PairedBinaryOperation::Multiplication { .. } => binary_definitions::mul(),
                            PairedBinaryOperation::Division { .. } => binary_definitions::div(),
                            PairedBinaryOperation::Remainder { .. } => binary_definitions::rem(),
                            PairedBinaryOperation::Equal { .. } => binary_definitions::eq(),
                            PairedBinaryOperation::NotEqual { .. } => binary_definitions::ne(),
                            PairedBinaryOperation::LessThan { .. } => binary_definitions::lt(),
                            PairedBinaryOperation::LessThanOrEqual { .. } => binary_definitions::le(),
                            PairedBinaryOperation::GreaterThanOrEqual { .. } => binary_definitions::ge(),
                            PairedBinaryOperation::GreaterThan { .. } => binary_definitions::gt(),
                            _ => return None,
                        })
                    }
                }
            }
        }

        impl HasValueKind for $float_type {
            type SpecificKind = FloatKind;

            fn kind(&self) -> FloatKind {
                FloatKind::$float_enum_variant
            }
        }

        impl IntoValue for $float_type {
            fn into_value(self) -> Value {
                Value::Float(FloatValue::$float_enum_variant(self))
            }
        }


        impl HandleBinaryOperation for $float_type {
            fn type_name() -> &'static str {
                stringify!($integer_type)
            }
        }
    )*};
}

impl_float_operations!(F32TypeData mod f32_interface: F32(f32), F64TypeData mod f64_interface: F64(f64));

macro_rules! impl_resolvable_float_subtype {
    ($value_type:ty, $type:ty, $variant:ident, $expected_msg:expr) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_value(
                value: Value,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    Value::Float(FloatValue::Untyped(x)) => x.parse_as(),
                    Value::Float(FloatValue::$variant(x)) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref<'a>(
                value: &'a Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                match value {
                    Value::Float(FloatValue::$variant(x)) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut<'a>(
                value: &'a mut Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                match value {
                    Value::Float(FloatValue::$variant(x)) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }
    };
}

impl_resolvable_float_subtype!(F32TypeData, f32, F32, "f32");
impl_resolvable_float_subtype!(F64TypeData, f64, F64, "f64");
