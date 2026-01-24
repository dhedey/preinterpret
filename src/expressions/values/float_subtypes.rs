use super::*;

macro_rules! impl_float_operations {
    (
        $($type_data:ident mod $mod_name:ident: $float_enum_variant:ident($float_type:ident)),* $(,)?
    ) => {$(
        define_type_features! {
            impl $type_data,
            pub(crate) mod $mod_name {
                unary_operations {
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
                interface_items {
                    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                        Some(match operation {
                            UnaryOperation::Neg { .. } => unary_definitions::neg(),
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
                        // All operations are defined on the parent FloatValue type
                        None
                    }

                    fn resolve_type_property(
                        property_name: &str,
                    ) -> Option<AnyValue> {
                        match property_name {
                            "MAX" => Some($float_type::MAX.into_any_value()),
                            "MIN" => Some($float_type::MIN.into_any_value()),
                            "MIN_POSITIVE" => Some($float_type::MIN_POSITIVE.into_any_value()),
                            "INFINITY" => Some($float_type::INFINITY.into_any_value()),
                            "NEG_INFINITY" => Some($float_type::NEG_INFINITY.into_any_value()),
                            "NAN" => Some($float_type::NAN.into_any_value()),
                            "EPSILON" => Some($float_type::EPSILON.into_any_value()),
                            _ => None,
                        }
                    }
                }
            }
        }

        impl HandleBinaryOperation for $float_type {
            fn type_name() -> &'static str {
                stringify!($integer_type)
            }
        }
    )*};
}

impl_float_operations!(F32Type mod f32_interface: F32(f32), F64Type mod f64_interface: F64(f64));

macro_rules! impl_resolvable_float_subtype {
    ($type_def:ident, $kind:ident, $type:ty, $variant:ident, $type_name:literal, $articled_display_name:expr) => {
        define_leaf_type! {
            pub(crate) $type_def => FloatType(FloatContent::$variant) => AnyType,
            content: $type,
            kind: pub(crate) $kind,
            type_name: $type_name,
            articled_display_name: $articled_display_name,
            dyn_impls: {},
        }

        impl IsArgument for OptionalSuffix<$type> {
            type ValueType = FloatType;
            const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

            fn from_argument(argument: Spanned<ArgumentValue>) -> ExecutionResult<Self> {
                argument.expect_owned().resolve_as("This argument")
            }
        }

        impl ResolveAs<OptionalSuffix<$type>> for Spanned<AnyValue> {
            fn resolve_as(self, resolution_target: &str) -> ExecutionResult<OptionalSuffix<$type>> {
                let span = self.span_range();
                let float_value: FloatValue = self.resolve_as(resolution_target)?;
                Spanned(float_value, span).resolve_as(resolution_target)
            }
        }

        impl ResolveAs<OptionalSuffix<$type>> for Spanned<FloatValue> {
            fn resolve_as(self, resolution_target: &str) -> ExecutionResult<OptionalSuffix<$type>> {
                let Spanned(value, span) = self;
                match value {
                    FloatContent::Untyped(v) => Ok(OptionalSuffix(v.into_fallback() as $type)),
                    FloatContent::$variant(v) => Ok(OptionalSuffix(v)),
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

        impl From<$type> for FloatValue {
            fn from(value: $type) -> Self {
                FloatContent::$variant(value)
            }
        }

        impl ResolvableOwned<FloatValue> for $type {
            fn resolve_from_value(
                value: FloatValue,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    FloatContent::Untyped(x) => Ok(x.into_fallback() as $type),
                    FloatContent::$variant(x) => Ok(x),
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
                    AnyValue::Float(x) => <$type>::resolve_from_value(x, context),
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
                    AnyValueContent::Float(FloatContent::$variant(x)) => Ok(x),
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
                    AnyValueContent::Float(FloatContent::$variant(x)) => Ok(x),
                    other => context.err($articled_display_name, other),
                }
            }
        }
    };
}

impl_resolvable_float_subtype!(F32Type, F32Kind, f32, F32, "f32", "an f32");
impl_resolvable_float_subtype!(F64Type, F64Kind, f64, F64, "f64", "an f64");
