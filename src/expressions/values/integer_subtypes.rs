use super::*;

// We have to use a macro because we don't have checked xx traits :(
macro_rules! impl_int_operations {
    (
        $($integer_type_data:ident mod $mod_name:ident: [$(CharCast[$char_cast:ident],)?$(Signed[$signed:ident],)?] $integer_enum_variant:ident($integer_type:ident)),* $(,)?
    ) => {$(
        define_type_features! {
            impl $integer_type_data,
            pub(crate) mod $mod_name {
                pub(crate) mod methods {
                }
                pub(crate) mod unary_operations {
                    $(
                        fn neg(Spanned(value, span): Spanned<Owned<$integer_type>>) -> ExecutionResult<$integer_type> {
                            ignore_all!($signed); // Include only for signed types
                            let value = value.into_inner();
                            match value.checked_neg() {
                                Some(negated) => Ok(negated),
                                None => span.value_err("Negating this value would overflow"),
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
                            UnaryOperation::Cast { target: CastTarget(kind), .. } => match kind {
                                $(
                                    AnyValueLeafKind::Char(_) => {
                                        ignore_all!($char_cast); // Only include for types with CharCast
                                        unary_definitions::cast_to_char()
                                    }
                                )?
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
                                AnyValueLeafKind::Float(FloatLeafKind::Untyped(_)) => unary_definitions::cast_to_untyped_float(),
                                AnyValueLeafKind::Float(FloatLeafKind::F32(_)) => unary_definitions::cast_to_f32(),
                                AnyValueLeafKind::Float(FloatLeafKind::F64(_)) => unary_definitions::cast_to_f64(),
                                AnyValueLeafKind::String(_) => unary_definitions::cast_to_string(),
                                _ => return None,
                            }
                            _ => return None,
                        })
                    }

                    fn resolve_own_binary_operation(
                        _operation: &BinaryOperation,
                    ) -> Option<BinaryOperationInterface> {
                        // All operations are defined on the parent IntegerValue type
                        None
                    }

                    fn resolve_type_property(
                        property_name: &str,
                    ) -> Option<AnyValue> {
                        match property_name {
                            "MAX" => Some($integer_type::MAX.into_value()),
                            "MIN" => Some($integer_type::MIN.into_value()),
                            _ => None,
                        }
                    }
                }
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
    U8Type mod u8_interface: [CharCast[yes],] U8(u8),
    U16Type mod u16_interface: [] U16(u16),
    U32Type mod u32_interface: [] U32(u32),
    U64Type mod u64_interface: [] U64(u64),
    U128Type mod u128_interface: [] U128(u128),
    UsizeType mod usize_interface: [] Usize(usize),
    I8Type mod i8_interface: [Signed[yes],] I8(i8),
    I16Type mod i16_interface: [Signed[yes],] I16(i16),
    I32Type mod i32_interface: [Signed[yes],] I32(i32),
    I64Type mod i64_interface: [Signed[yes],] I64(i64),
    I128Type mod i128_interface: [Signed[yes],] I128(i128),
    IsizeType mod isize_interface: [Signed[yes],] Isize(isize),
);

macro_rules! impl_resolvable_integer_subtype {
    ($type_def:ident, $kind:ident, $type:ty, $variant:ident, $type_name:literal, $articled_display_name:expr) => {
        define_leaf_type! {
            pub(crate) $type_def => IntegerType(IntegerContent::$variant) => AnyType,
            content: $type,
            kind: pub(crate) $kind,
            type_name: $type_name,
            articled_display_name: $articled_display_name,
            dyn_impls: {},
        }

        impl IsArgument for OptionalSuffix<$type> {
            type ValueType = IntegerType;
            const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

            fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
                argument.expect_owned().resolve_as("This argument")
            }
        }

        impl ResolveAs<OptionalSuffix<$type>> for Spanned<OwnedValue> {
            fn resolve_as(self, resolution_target: &str) -> ExecutionResult<OptionalSuffix<$type>> {
                let span = self.span_range();
                let integer_value: IntegerValue = self.resolve_as(resolution_target)?;
                Spanned(integer_value, span).resolve_as(resolution_target)
            }
        }

        impl ResolveAs<OptionalSuffix<$type>> for Spanned<IntegerValue> {
            fn resolve_as(self, resolution_target: &str) -> ExecutionResult<OptionalSuffix<$type>> {
                let Spanned(value, span) = self;
                match value {
                    IntegerValue::Untyped(v) => Ok(OptionalSuffix(v.into_fallback() as $type)),
                    IntegerValue::$variant(v) => Ok(OptionalSuffix(v)),
                    v => span.type_err(format!(
                        "{} is expected to be {}, but it is {}",
                        resolution_target,
                        $kind.articled_display_name(),
                        v.articled_kind(),
                    )),
                }
            }
        }

        impl ResolvableArgumentTarget for $type {
            type ValueType = $type_def;
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
                    IntegerValue::Untyped(x) => Ok(x.into_fallback() as $type),
                    IntegerValue::$variant(x) => Ok(x),
                    other => context.err($articled_display_name, other),
                }
            }
        }

        impl ResolvableOwned<AnyValue> for $type {
            fn resolve_from_value(
                value: AnyValue,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    AnyValue::Integer(x) => <$type>::resolve_from_value(x, context),
                    other => context.err($articled_display_name, other),
                }
            }
        }

        impl ResolvableShared<AnyValue> for $type {
            fn resolve_from_ref<'a>(
                value: &'a AnyValue,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                match value {
                    AnyValue::Integer(IntegerValue::$variant(x)) => Ok(x),
                    other => context.err($articled_display_name, other),
                }
            }
        }

        impl ResolvableMutable<AnyValue> for $type {
            fn resolve_from_mut<'a>(
                value: &'a mut AnyValue,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                match value {
                    AnyValue::Integer(IntegerValue::$variant(x)) => Ok(x),
                    other => context.err($articled_display_name, other),
                }
            }
        }
    };
}

impl_resolvable_integer_subtype!(I8Type, I8Kind, i8, I8, "i8", "an i8");
impl_resolvable_integer_subtype!(I16Type, I16Kind, i16, I16, "i16", "an i16");
impl_resolvable_integer_subtype!(I32Type, I32Kind, i32, I32, "i32", "an i32");
impl_resolvable_integer_subtype!(I64Type, I64Kind, i64, I64, "i64", "an i64");
impl_resolvable_integer_subtype!(I128Type, I128Kind, i128, I128, "i128", "an i128");
impl_resolvable_integer_subtype!(IsizeType, IsizeKind, isize, Isize, "isize", "an isize");
impl_resolvable_integer_subtype!(U8Type, U8Kind, u8, U8, "u8", "a u8");
impl_resolvable_integer_subtype!(U16Type, U16Kind, u16, U16, "u16", "a u16");
impl_resolvable_integer_subtype!(U32Type, U32Kind, u32, U32, "u32", "a u32");
impl_resolvable_integer_subtype!(U64Type, U64Kind, u64, U64, "u64", "a u64");
impl_resolvable_integer_subtype!(U128Type, U128Kind, u128, U128, "u128", "a u128");
impl_resolvable_integer_subtype!(UsizeType, UsizeKind, usize, Usize, "usize", "a usize");
