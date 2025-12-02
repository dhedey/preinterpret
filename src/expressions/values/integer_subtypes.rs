use super::*;

// We have to use a macro because we don't have checked xx traits :(
macro_rules! impl_int_operations {
    (
        $($integer_type_data:ident mod $mod_name:ident: [$(CharCast[$char_cast:ident],)?$(Signed[$signed:ident],)?] $integer_enum_variant:ident($integer_type:ident)),* $(,)?
    ) => {$(
        define_interface! {
            struct $integer_type_data,
            parent: IntegerTypeData,
            pub(crate) mod $mod_name {
                pub(crate) mod methods {
                }
                pub(crate) mod unary_operations {
                    $(
                        fn neg(this: Owned<$integer_type>) -> ExecutionResult<$integer_type> {
                            ignore_all!($signed); // Include only for signed types
                            let (value, span_range) = this.deconstruct();
                            match value.checked_neg() {
                                Some(negated) => Ok(negated),
                                None => span_range.value_err("Negating this value would overflow"),
                            }
                        }
                    )?

                    $(
                        fn cast_to_char(input: $integer_type) -> char {
                            ignore_all!($char_cast); // Include only for types with CharCast
                            input as char
                        }
                    )?

                    fn cast_to_untyped_integer(input: $integer_type) -> UntypedInteger {
                        UntypedInteger::from_fallback(input as FallbackInteger)
                    }

                    fn cast_to_i8(input: $integer_type) -> i8 {
                        input as i8
                    }

                    fn cast_to_i16(input: $integer_type) -> i16 {
                        input as i16
                    }

                    fn cast_to_i32(input: $integer_type) -> i32 {
                        input as i32
                    }

                    fn cast_to_i64(input: $integer_type) -> i64 {
                        input as i64
                    }

                    fn cast_to_i128(input: $integer_type) -> i128 {
                        input as i128
                    }

                    fn cast_to_isize(input: $integer_type) -> isize {
                        input as isize
                    }

                    fn cast_to_u8(input: $integer_type) -> u8 {
                        input as u8
                    }

                    fn cast_to_u16(input: $integer_type) -> u16 {
                        input as u16
                    }

                    fn cast_to_u32(input: $integer_type) -> u32 {
                        input as u32
                    }

                    fn cast_to_u64(input: $integer_type) -> u64 {
                        input as u64
                    }

                    fn cast_to_u128(input: $integer_type) -> u128 {
                        input as u128
                    }

                    fn cast_to_usize(input: $integer_type) -> usize {
                        input as usize
                    }

                    fn cast_to_untyped_float(input: $integer_type) -> UntypedFloat {
                        UntypedFloat::from_fallback(input as FallbackFloat)
                    }

                    fn cast_to_f32(input: $integer_type) -> f32 {
                        input as f32
                    }

                    fn cast_to_f64(input: $integer_type) -> f64 {
                        input as f64
                    }

                    fn cast_to_string(input: $integer_type) -> String {
                        input.to_string()
                    }
                }
                pub(crate) mod binary_operations {
                    fn eq(lhs: $integer_type, rhs: $integer_type) -> bool {
                        lhs == rhs
                    }

                    fn ne(lhs: $integer_type, rhs: $integer_type) -> bool {
                        lhs != rhs
                    }

                    fn lt(lhs: $integer_type, rhs: $integer_type) -> bool {
                        lhs < rhs
                    }

                    fn le(lhs: $integer_type, rhs: $integer_type) -> bool {
                        lhs <= rhs
                    }

                    fn ge(lhs: $integer_type, rhs: $integer_type) -> bool {
                        lhs >= rhs
                    }

                    fn gt(lhs: $integer_type, rhs: $integer_type) -> bool {
                        lhs > rhs
                    }
                }
                interface_items {
                    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                        Some(match operation {
                            $(
                                UnaryOperation::Neg { .. } => {
                                    ignore_all!($signed); // Only include for signed types
                                    unary_definitions::neg()
                                }
                            )?
                            UnaryOperation::Cast { target, .. } => match target {
                                $(
                                    CastTarget::Char => {
                                        ignore_all!($char_cast); // Only include for types with CharCast
                                        unary_definitions::cast_to_char()
                                    }
                                )?
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

                    fn resolve_own_binary_operation(
                        operation: &BinaryOperation,
                    ) -> Option<BinaryOperationInterface> {
                        Some(match operation {
                            // Most operations are defined on the integer value directly
                            BinaryOperation::Equal { .. } => binary_definitions::eq(),
                            BinaryOperation::NotEqual { .. } => binary_definitions::ne(),
                            BinaryOperation::LessThan { .. } => binary_definitions::lt(),
                            BinaryOperation::LessThanOrEqual { .. } => binary_definitions::le(),
                            BinaryOperation::GreaterThanOrEqual { .. } => binary_definitions::ge(),
                            BinaryOperation::GreaterThan { .. } => binary_definitions::gt(),
                            _ => return None,
                        })
                    }
                }
            }
        }

        impl HasValueKind for $integer_type {
            type SpecificKind = IntegerKind;

            fn kind(&self) -> IntegerKind {
                IntegerKind::$integer_enum_variant
            }
        }

        impl IntoValue for $integer_type {
            fn into_value(self) -> Value {
                Value::Integer(IntegerValue::$integer_enum_variant(self))
            }
        }

        impl HandleBinaryOperation for $integer_type {
            fn type_name() -> &'static str {
                stringify!($integer_type)
            }
        }
    )*};
}

impl_int_operations!(
    U8TypeData mod u8_interface: [CharCast[yes],] U8(u8),
    U16TypeData mod u16_interface: [] U16(u16),
    U32TypeData mod u32_interface: [] U32(u32),
    U64TypeData mod u64_interface: [] U64(u64),
    U128TypeData mod u128_interface: [] U128(u128),
    UsizeTypeData mod usize_interface: [] Usize(usize),
    I8TypeData mod i8_interface: [Signed[yes],] I8(i8),
    I16TypeData mod i16_interface: [Signed[yes],] I16(i16),
    I32TypeData mod i32_interface: [Signed[yes],] I32(i32),
    I64TypeData mod i64_interface: [Signed[yes],] I64(i64),
    I128TypeData mod i128_interface: [Signed[yes],] I128(i128),
    IsizeTypeData mod isize_interface: [Signed[yes],] Isize(isize),
);

macro_rules! impl_resolvable_integer_subtype {
    ($value_type:ty, $type:ty, $variant:ident, $expected_msg:expr) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl From<$type> for IntegerValue {
            fn from(value: $type) -> Self {
                IntegerValue::$variant(value)
            }
        }

        impl ResolvableOwned<IntegerValue> for $type {
            fn resolve_from_value(
                value: IntegerValue,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    IntegerValue::Untyped(x) => {
                        x.into_spanned_ref(context.error_span_range()).parse_as()
                    }
                    IntegerValue::$variant(x) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }

        impl ResolvableOwned<Value> for $type {
            fn resolve_from_value(
                value: Value,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    Value::Integer(x) => <$type>::resolve_from_value(x, context),
                    other => context.err($expected_msg, other),
                }
            }
        }

        impl ResolvableShared<Value> for $type {
            fn resolve_from_ref<'a>(
                value: &'a Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                match value {
                    Value::Integer(IntegerValue::$variant(x)) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }

        impl ResolvableMutable<Value> for $type {
            fn resolve_from_mut<'a>(
                value: &'a mut Value,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                match value {
                    Value::Integer(IntegerValue::$variant(x)) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }
    };
}

impl_resolvable_integer_subtype!(I8TypeData, i8, I8, "an i8");
impl_resolvable_integer_subtype!(I16TypeData, i16, I16, "an i16");
impl_resolvable_integer_subtype!(I32TypeData, i32, I32, "an i32");
impl_resolvable_integer_subtype!(I64TypeData, i64, I64, "an i64");
impl_resolvable_integer_subtype!(I128TypeData, i128, I128, "an i128");
impl_resolvable_integer_subtype!(IsizeTypeData, isize, Isize, "an isize");
impl_resolvable_integer_subtype!(U8TypeData, u8, U8, "a u8");
impl_resolvable_integer_subtype!(U16TypeData, u16, U16, "a u16");
impl_resolvable_integer_subtype!(U32TypeData, u32, U32, "a u32");
impl_resolvable_integer_subtype!(U64TypeData, u64, U64, "a u64");
impl_resolvable_integer_subtype!(U128TypeData, u128, U128, "a u128");
impl_resolvable_integer_subtype!(UsizeTypeData, usize, Usize, "a usize");
