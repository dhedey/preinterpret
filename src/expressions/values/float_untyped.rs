use super::*;

define_leaf_type! {
    pub(crate) UntypedFloatType => FloatType(FloatContent::Untyped) => ValueType,
    content: UntypedFloat,
    kind: pub(crate) UntypedFloatKind,
    type_name: "untyped_float",
    articled_display_name: "an untyped float",
    dyn_impls: {},
}

#[derive(Copy, Clone)]
pub(crate) struct UntypedFloat(FallbackFloat);
pub(crate) type FallbackFloat = f64;

impl UntypedFloat {
    pub(super) fn new_from_lit_float(lit_float: &LitFloat) -> ParseResult<Self> {
        Ok(Self(lit_float.base10_digits().parse().map_err(|err| {
            lit_float.parse_error(format!(
                "Untyped floats in preinterpret must fit inside a {}: {}",
                core::any::type_name::<FallbackFloat>(),
                err
            ))
        })?))
    }

    /// Converts an untyped float to a specific float kind.
    /// Unlike integers, float conversion never fails (may lose precision).
    pub(crate) fn into_kind(self, kind: FloatLeafKind) -> OwnedFloat {
        match kind {
            FloatLeafKind::Untyped(_) => FloatContent::Untyped(self),
            FloatLeafKind::F32(_) => FloatContent::F32(self.0 as f32),
            FloatLeafKind::F64(_) => FloatContent::F64(self.0),
        }
    }

    pub(crate) fn paired_operation(
        self,
        rhs: Spanned<OwnedFloat>,
        perform_fn: fn(FallbackFloat, FallbackFloat) -> FallbackFloat,
    ) -> ExecutionResult<OwnedFloat> {
        let lhs = self.0;
        let rhs: UntypedFloat = rhs.downcast_resolve("This operand")?;
        let rhs = rhs.0;
        let output = perform_fn(lhs, rhs);
        Ok(FloatContent::Untyped(UntypedFloat::from_fallback(output)))
    }

    pub(crate) fn paired_comparison(
        self,
        rhs: Spanned<OwnedFloat>,
        compare_fn: fn(FallbackFloat, FallbackFloat) -> bool,
    ) -> ExecutionResult<bool> {
        let lhs = self.0;
        let rhs: UntypedFloat = rhs.downcast_resolve("This operand")?;
        let rhs = rhs.0;
        Ok(compare_fn(lhs, rhs))
    }

    pub(crate) fn into_fallback(self) -> FallbackFloat {
        self.0
    }

    pub(crate) fn from_fallback(value: FallbackFloat) -> Self {
        Self(value)
    }

    pub(super) fn to_unspanned_literal(self) -> Literal {
        Literal::f64_unsuffixed(self.0)
    }
}

define_type_features! {
    impl UntypedFloatType,
    pub(crate) mod untyped_float_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
            fn neg(input: UntypedFloatFallback) -> UntypedFloat {
                UntypedFloat::from_fallback(-input.0)
            }

            fn cast_to_untyped_integer(input: UntypedFloatFallback) -> UntypedInteger {
                UntypedInteger::from_fallback(input.0 as FallbackInteger)
            }

            fn cast_to_i8(input: UntypedFloatFallback) -> i8 {
                input.0 as i8
            }

            fn cast_to_i16(input: UntypedFloatFallback) -> i16 {
                input.0 as i16
            }

            fn cast_to_i32(input: UntypedFloatFallback) -> i32 {
                input.0 as i32
            }

            fn cast_to_i64(input: UntypedFloatFallback) -> i64 {
                input.0 as i64
            }

            fn cast_to_i128(input: UntypedFloatFallback) -> i128 {
                input.0 as i128
            }

            fn cast_to_isize(input: UntypedFloatFallback) -> isize {
                input.0 as isize
            }

            fn cast_to_u8(input: UntypedFloatFallback) -> u8 {
                input.0 as u8
            }

            fn cast_to_u16(input: UntypedFloatFallback) -> u16 {
                input.0 as u16
            }

            fn cast_to_u32(input: UntypedFloatFallback) -> u32 {
                input.0 as u32
            }

            fn cast_to_u64(input: UntypedFloatFallback) -> u64 {
                input.0 as u64
            }

            fn cast_to_u128(input: UntypedFloatFallback) -> u128 {
                input.0 as u128
            }

            fn cast_to_usize(input: UntypedFloatFallback) -> usize {
                input.0 as usize
            }

            fn cast_to_untyped_float(input: UntypedFloatFallback) -> UntypedFloat {
                UntypedFloat::from_fallback(input.0)
            }

            fn cast_to_f32(input: UntypedFloatFallback) -> f32 {
                input.0 as f32
            }

            fn cast_to_f64(input: UntypedFloatFallback) -> f64 {
                input.0
            }

            fn cast_to_string(input: UntypedFloatFallback) -> String {
                input.0.to_string()
            }
        }
        pub(crate) mod binary_operations {
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Neg { .. } => unary_definitions::neg(),
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
                        ValueLeafKind::Float(FloatLeafKind::Untyped(_)) => unary_definitions::cast_to_untyped_float(),
                        ValueLeafKind::Float(FloatLeafKind::F32(_)) => unary_definitions::cast_to_f32(),
                        ValueLeafKind::Float(FloatLeafKind::F64(_)) => unary_definitions::cast_to_f64(),
                        ValueLeafKind::String(_) => unary_definitions::cast_to_string(),
                        _ => return None,
                    },
                    _ => return None,
                })
            }

            fn resolve_own_binary_operation(
                _operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                // All operations are defined on the parent FloatValue type
                None
            }
        }
    }
}

pub(crate) struct UntypedFloatFallback(pub FallbackFloat);

impl ResolvableArgumentTarget for UntypedFloatFallback {
    type ValueType = UntypedFloatType;
}

impl ResolvableOwned<Value> for UntypedFloatFallback {
    fn resolve_from_value(input_value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
        let value = UntypedFloat::resolve_from_value(input_value, context)?;
        Ok(UntypedFloatFallback(value.0))
    }
}

impl ResolvableOwned<OwnedFloat> for UntypedFloat {
    fn resolve_from_value(value: OwnedFloat, context: ResolutionContext) -> ExecutionResult<Self> {
        match value {
            FloatContent::Untyped(value) => Ok(value),
            _ => context.err("an untyped float", value),
        }
    }
}

impl_resolvable_argument_for! {
    UntypedFloatType,
    (value, context) -> UntypedFloat {
        match value {
            ValueContent::Float(FloatContent::Untyped(x)) => Ok(x),
            other => context.err("an untyped float", other),
        }
    }
}
