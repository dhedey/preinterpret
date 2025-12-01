use super::*;

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

    pub(crate) fn into_kind(self, kind: FloatKind) -> ExecutionResult<FloatValue> {
        Ok(match kind {
            FloatKind::Untyped => FloatValue::Untyped(self),
            FloatKind::F32 => FloatValue::F32(self.0 as f32),
            FloatKind::F64 => FloatValue::F64(self.0 as f64),
        })
    }

    pub(crate) fn paired_operation(
        self,
        rhs: Owned<FloatValue>,
        perform_fn: fn(FallbackFloat, FallbackFloat) -> FallbackFloat,
    ) -> ExecutionResult<FloatValue> {
        let lhs = self.0;
        let rhs: UntypedFloat = rhs.resolve_as("This operand")?;
        let rhs = rhs.0;
        let output = perform_fn(lhs, rhs);
        Ok(FloatValue::Untyped(UntypedFloat::from_fallback(output)))
    }

    fn paired_comparison(
        lhs: Owned<UntypedFloat>,
        rhs: Owned<FloatValue>,
        context: BinaryOperationCallContext,
        compare_fn: fn(FallbackFloat, FallbackFloat) -> bool,
    ) -> ExecutionResult<bool> {
        let (lhs, lhs_span_range) = lhs.deconstruct();
        let (rhs, rhs_span_range) = rhs.deconstruct();
        match rhs {
            FloatValue::Untyped(rhs) => {
                let lhs = lhs.0;
                let rhs = rhs.0;
                Ok(compare_fn(lhs, rhs))
            }
            rhs => {
                // Re-evaluate with lhs converted to the typed float
                let lhs = lhs.into_kind(rhs.kind())?;
                context
                    .operation
                    .evaluate(
                        lhs.into_owned_value(lhs_span_range),
                        rhs.into_owned_value(rhs_span_range),
                    )?
                    .expect_owned()
                    .resolve_as("The result of a comparison")
            }
        }
    }

    pub(super) fn into_fallback(self) -> FallbackFloat {
        self.0
    }

    pub(super) fn from_fallback(value: FallbackFloat) -> Self {
        Self(value)
    }

    pub(super) fn to_unspanned_literal(&self) -> Literal {
        Literal::f64_unsuffixed(self.0)
    }
}

impl HasValueKind for UntypedFloat {
    type SpecificKind = FloatKind;

    fn kind(&self) -> FloatKind {
        FloatKind::Untyped
    }
}

impl IntoValue for UntypedFloat {
    fn into_value(self) -> Value {
        Value::Float(FloatValue::Untyped(self))
    }
}

define_interface! {
    struct UntypedFloatTypeData,
    parent: FloatTypeData,
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
            [context] fn eq(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatValue>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a == b)
            }

            [context] fn ne(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatValue>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a != b)
            }

            [context] fn lt(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatValue>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a < b)
            }

            [context] fn le(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatValue>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a <= b)
            }

            [context] fn ge(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatValue>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a >= b)
            }

            [context] fn gt(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatValue>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a > b)
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
                    },
                    _ => return None,
                })
            }

            fn resolve_paired_binary_operation(
                operation: &PairedBinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    // Most operations are defined on the float value directly
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

pub(crate) struct UntypedFloatFallback(pub FallbackFloat);

impl ResolvableArgumentTarget for UntypedFloatFallback {
    type ValueType = UntypedFloatTypeData;
}

impl ResolvableOwned<Value> for UntypedFloatFallback {
    fn resolve_from_value(input_value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
        let value = UntypedFloat::resolve_from_value(input_value, context)?;
        Ok(UntypedFloatFallback(value.0))
    }
}

impl ResolvableOwned<FloatValue> for UntypedFloat {
    fn resolve_from_value(
        value: FloatValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        match value {
            FloatValue::Untyped(value) => Ok(value),
            _ => context.err("untyped float", value),
        }
    }
}

impl_resolvable_argument_for! {
    UntypedFloatTypeData,
    (value, context) -> UntypedFloat {
        match value {
            Value::Float(FloatValue::Untyped(x)) => Ok(x),
            other => context.err("untyped float", other),
        }
    }
}
