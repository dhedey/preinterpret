use super::*;

#[derive(Clone)]
pub(crate) struct UntypedFloat(
    /// The span of the literal is ignored, and will be set when converted to an output.
    LitFloat,
);
pub(crate) type FallbackFloat = f64;

impl UntypedFloat {
    pub(super) fn new_from_lit_float(lit_float: LitFloat) -> Self {
        Self(lit_float)
    }

    fn new_from_known_float_literal(literal: Literal) -> Self {
        Self::new_from_lit_float(literal.into())
    }

    fn into_kind(self, kind: FloatKind) -> ExecutionResult<FloatExpressionValue> {
        Ok(match kind {
            FloatKind::Untyped => FloatExpressionValue::Untyped(self),
            FloatKind::F32 => FloatExpressionValue::F32(self.parse_as()?),
            FloatKind::F64 => FloatExpressionValue::F64(self.parse_as()?),
        })
    }

    fn paired_operation(
        lhs: Owned<UntypedFloat>,
        rhs: Owned<FloatExpression>,
        context: BinaryOperationCallContext,
        perform_fn: fn(FallbackFloat, FallbackFloat) -> Option<FallbackFloat>,
    ) -> ExecutionResult<ReturnedValue> {
        let (lhs, lhs_span_range) = lhs.deconstruct();
        let (rhs, rhs_span_range) = rhs.deconstruct();
        match rhs.value {
            FloatExpressionValue::Untyped(rhs) => {
                let lhs = lhs.parse_fallback()?;
                let rhs = rhs.parse_fallback()?;
                let output = perform_fn(lhs, rhs).ok_or_else(|| {
                    context.error(format!(
                        "The untyped integer operation {} {} {} overflowed in i128 space",
                        lhs,
                        context.operation.symbolic_description(),
                        rhs
                    ))
                })?;
                UntypedFloat::from_fallback(output).to_returned_value(context.output_span_range)
            }
            rhs => {
                let lhs = lhs.into_kind(rhs.kind())?;
                context.operation.evaluate(
                    lhs.into_owned_value(lhs_span_range),
                    rhs.into_owned_value(rhs_span_range),
                )
            }
        }
    }

    fn paired_comparison(
        lhs: Owned<UntypedFloat>,
        rhs: Owned<FloatExpression>,
        context: BinaryOperationCallContext,
        compare_fn: fn(FallbackFloat, FallbackFloat) -> bool,
    ) -> ExecutionResult<bool> {
        let (lhs, lhs_span_range) = lhs.deconstruct();
        let (rhs, rhs_span_range) = rhs.deconstruct();
        match rhs.value {
            FloatExpressionValue::Untyped(rhs) => {
                let lhs = lhs.parse_fallback()?;
                let rhs = rhs.parse_fallback()?;
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

    pub(super) fn from_fallback(value: FallbackFloat) -> Self {
        // TODO[untyped] - Have a way to store this more efficiently without going through a literal
        Self::new_from_known_float_literal(
            Literal::f64_unsuffixed(value).with_span(Span::call_site()),
        )
    }

    pub(crate) fn parse_fallback(&self) -> ExecutionResult<FallbackFloat> {
        self.0.base10_digits().parse().map_err(|err| {
            self.0.value_error(format!(
                "Could not parse as the default inferred type {}: {}",
                core::any::type_name::<FallbackFloat>(),
                err
            ))
        })
    }

    pub(crate) fn parse_as<N>(&self) -> ExecutionResult<N>
    where
        N: FromStr,
        N::Err: core::fmt::Display,
    {
        self.0.base10_digits().parse().map_err(|err| {
            self.0.value_error(format!(
                "Could not parse as {}: {}",
                core::any::type_name::<N>(),
                err
            ))
        })
    }

    pub(super) fn to_unspanned_literal(&self) -> Literal {
        self.0.token()
    }
}

impl HasValueType for UntypedFloat {
    fn value_type(&self) -> &'static str {
        "untyped float"
    }
}

impl ToExpressionValue for UntypedFloat {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Float(FloatExpression {
            value: FloatExpressionValue::Untyped(self),
        })
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
            [context] fn add(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ReturnedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a + b))
            }

            [context] fn sub(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ReturnedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a - b))
            }

            [context] fn mul(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ReturnedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a * b))
            }

            [context] fn div(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ReturnedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a / b))
            }

            [context] fn rem(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ReturnedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a % b))
            }

            [context] fn eq(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a == b)
            }

            [context] fn ne(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a != b)
            }

            [context] fn lt(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a < b)
            }

            [context] fn le(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a <= b)
            }

            [context] fn ge(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<bool> {
                UntypedFloat::paired_comparison(lhs, rhs, context, |a, b| a >= b)
            }

            [context] fn gt(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
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

pub(crate) struct UntypedFloatFallback(pub FallbackFloat);

impl ResolvableArgumentTarget for UntypedFloatFallback {
    type ValueType = UntypedFloatTypeData;
}

impl ResolvableArgumentOwned for UntypedFloatFallback {
    fn resolve_from_value(
        input_value: ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        let value = UntypedFloat::resolve_from_value(input_value, context)?;
        Ok(UntypedFloatFallback(value.parse_fallback()?))
    }
}

impl_resolvable_argument_for! {
    UntypedFloatTypeData,
    (value, context) -> UntypedFloat {
        match value {
            ExpressionValue::Float(FloatExpression { value: FloatExpressionValue::Untyped(x), ..}) => Ok(x),
            other => context.err("untyped float", other),
        }
    }
}
