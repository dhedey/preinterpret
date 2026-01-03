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
                }
                interface_items {
                    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                        Some(match operation {
                            UnaryOperation::Neg { .. } => unary_definitions::neg(),
                            UnaryOperation::Cast { target, .. } => match target {
                                CastTarget::Integer(IntegerLeafKind::Untyped(_)) => unary_definitions::cast_to_untyped_integer(),
                                CastTarget::Integer(IntegerLeafKind::I8(_)) => unary_definitions::cast_to_i8(),
                                CastTarget::Integer(IntegerLeafKind::I16(_)) => unary_definitions::cast_to_i16(),
                                CastTarget::Integer(IntegerLeafKind::I32(_)) => unary_definitions::cast_to_i32(),
                                CastTarget::Integer(IntegerLeafKind::I64(_)) => unary_definitions::cast_to_i64(),
                                CastTarget::Integer(IntegerLeafKind::I128(_)) => unary_definitions::cast_to_i128(),
                                CastTarget::Integer(IntegerLeafKind::Isize(_)) => unary_definitions::cast_to_isize(),
                                CastTarget::Integer(IntegerLeafKind::U8(_)) => unary_definitions::cast_to_u8(),
                                CastTarget::Integer(IntegerLeafKind::U16(_)) => unary_definitions::cast_to_u16(),
                                CastTarget::Integer(IntegerLeafKind::U32(_)) => unary_definitions::cast_to_u32(),
                                CastTarget::Integer(IntegerLeafKind::U64(_)) => unary_definitions::cast_to_u64(),
                                CastTarget::Integer(IntegerLeafKind::U128(_)) => unary_definitions::cast_to_u128(),
                                CastTarget::Integer(IntegerLeafKind::Usize(_)) => unary_definitions::cast_to_usize(),
                                CastTarget::Float(FloatLeafKind::Untyped(_)) => unary_definitions::cast_to_untyped_float(),
                                CastTarget::Float(FloatLeafKind::F32(_)) => unary_definitions::cast_to_f32(),
                                CastTarget::Float(FloatLeafKind::F64(_)) => unary_definitions::cast_to_f64(),
                                CastTarget::String => unary_definitions::cast_to_string(),
                                _ => return None,
                            }
                            _ => return None,
                        })
                    }

                    fn resolve_own_binary_operation(
                        _operation: &BinaryOperation,
                    ) -> Option<BinaryOperationInterface> {
                        // All operations are defined on the parent FloatValue type
                        None
                    }

                    fn resolve_type_property(
                        property_name: &str,
                    ) -> Option<Value> {
                        match property_name {
                            "MAX" => Some($float_type::MAX.into_value()),
                            "MIN" => Some($float_type::MIN.into_value()),
                            "MIN_POSITIVE" => Some($float_type::MIN_POSITIVE.into_value()),
                            "INFINITY" => Some($float_type::INFINITY.into_value()),
                            "NEG_INFINITY" => Some($float_type::NEG_INFINITY.into_value()),
                            "NAN" => Some($float_type::NAN.into_value()),
                            "EPSILON" => Some($float_type::EPSILON.into_value()),
                            _ => None,
                        }
                    }
                }
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
    ($type_def:ident, $kind:ident, $value_type:ident, $type:ty, $variant:ident, $type_name:literal, $articled_display_name:expr) => {
        define_leaf_type! {
            pub(crate) $type_def => FloatType(FloatContent::$variant) => ValueType,
            content: $type,
            kind: pub(crate) $kind,
            type_name: $type_name,
            articled_display_name: $articled_display_name,
            temp_type_data: $value_type,
        }

        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl From<$type> for FloatValue {
            fn from(value: $type) -> Self {
                FloatValue::$variant(value)
            }
        }

        impl ResolvableOwned<FloatValue> for $type {
            fn resolve_from_value(
                value: FloatValue,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    FloatValue::Untyped(x) => Ok(x.into_fallback() as $type),
                    FloatValue::$variant(x) => Ok(x),
                    other => context.err($articled_display_name, other),
                }
            }
        }

        impl ResolvableOwned<Value> for $type {
            fn resolve_from_value(
                value: Value,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    Value::Float(x) => <$type>::resolve_from_value(x, context),
                    other => context.err($articled_display_name, other),
                }
            }
        }

        impl ResolvableShared<Value> for $type {
            fn resolve_from_ref<'a>(
                value: &'a Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                match value {
                    Value::Float(FloatValue::$variant(x)) => Ok(x),
                    other => context.err($articled_display_name, other),
                }
            }
        }

        impl ResolvableMutable<Value> for $type {
            fn resolve_from_mut<'a>(
                value: &'a mut Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                match value {
                    Value::Float(FloatValue::$variant(x)) => Ok(x),
                    other => context.err($articled_display_name, other),
                }
            }
        }
    };
}

impl_resolvable_float_subtype!(F32Type, F32Kind, F32TypeData, f32, F32, "f32", "an f32");
impl_resolvable_float_subtype!(F64Type, F64Kind, F64TypeData, f64, F64, "f64", "an f64");
