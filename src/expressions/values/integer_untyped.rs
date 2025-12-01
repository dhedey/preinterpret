use super::*;

#[derive(Clone)]
pub(crate) struct UntypedInteger(syn::LitInt);
pub(crate) type FallbackInteger = i128;

impl UntypedInteger {
    pub(super) fn new_from_lit_int(lit_int: LitInt) -> Self {
        Self(lit_int)
    }

    fn new_from_known_int_literal(literal: Literal) -> Self {
        Self::new_from_lit_int(literal.into())
    }

    fn binary_overflow_error(
        context: BinaryOperationCallContext,
        lhs: impl std::fmt::Display,
        rhs: impl std::fmt::Display,
    ) -> ExecutionInterrupt {
        context.error(format!(
            "The untyped integer operation {} {} {} overflowed in i128 space",
            lhs,
            context.operation.symbolic_description(),
            rhs
        ))
    }

    fn paired_comparison(
        lhs: Owned<UntypedInteger>,
        rhs: Owned<IntegerValue>,
        context: BinaryOperationCallContext,
        compare_fn: fn(FallbackInteger, FallbackInteger) -> bool,
    ) -> ExecutionResult<bool> {
        let (lhs, lhs_span_range) = lhs.deconstruct();
        let (rhs, rhs_span_range) = rhs.deconstruct();
        match rhs {
            IntegerValue::Untyped(rhs) => {
                let lhs = lhs.parse_fallback()?;
                let rhs = rhs.parse_fallback()?;
                Ok(compare_fn(lhs, rhs))
            }
            rhs => {
                // Re-evaluate with lhs converted to the typed integer
                let lhs = lhs.into_owned(lhs_span_range).into_kind(rhs.kind())?;
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

    pub(crate) fn from_fallback(value: FallbackInteger) -> Self {
        // TODO[untyped] - Have a way to store this more efficiently without going through a literal
        Self::new_from_known_int_literal(
            Literal::i128_unsuffixed(value).with_span(Span::call_site()),
        )
    }

    pub(crate) fn parse_fallback(&self) -> ExecutionResult<FallbackInteger> {
        self.0.base10_digits().parse().map_err(|err| {
            self.0.value_error(format!(
                "Could not parse as the default inferred type {}: {}",
                core::any::type_name::<FallbackInteger>(),
                err
            ))
        })
    }

    pub(super) fn to_unspanned_literal(&self) -> Literal {
        self.0.token()
    }
}

impl Owned<UntypedInteger> {
    pub(crate) fn paired_operation(
        self,
        rhs: Owned<IntegerValue>,
        context: BinaryOperationCallContext,
        perform_fn: fn(FallbackInteger, FallbackInteger) -> Option<FallbackInteger>,
    ) -> ExecutionResult<IntegerValue> {
        let lhs = self.parse_fallback()?;
        let rhs: UntypedInteger = rhs.resolve_as("This operand")?;
        let rhs = rhs.parse_fallback()?;
        let output = perform_fn(lhs, rhs)
            .ok_or_else(|| UntypedInteger::binary_overflow_error(context, lhs, rhs))?;
        Ok(IntegerValue::Untyped(UntypedInteger::from_fallback(output)))
    }

    pub(crate) fn shift_operation(
        self,
        rhs: u32,
        context: BinaryOperationCallContext,
        perform_fn: fn(FallbackInteger, u32) -> Option<FallbackInteger>,
    ) -> ExecutionResult<IntegerValue> {
        let lhs = self.parse_fallback()?;
        let output = perform_fn(lhs, rhs)
            .ok_or_else(|| UntypedInteger::binary_overflow_error(context, lhs, rhs))?;
        Ok(IntegerValue::Untyped(UntypedInteger::from_fallback(output)))
    }

    pub(crate) fn into_kind(self, kind: IntegerKind) -> ExecutionResult<IntegerValue> {
        Ok(match kind {
            IntegerKind::Untyped => IntegerValue::Untyped(self.value),
            IntegerKind::I8 => IntegerValue::I8(self.as_ref().parse_as()?),
            IntegerKind::I16 => IntegerValue::I16(self.as_ref().parse_as()?),
            IntegerKind::I32 => IntegerValue::I32(self.as_ref().parse_as()?),
            IntegerKind::I64 => IntegerValue::I64(self.as_ref().parse_as()?),
            IntegerKind::I128 => IntegerValue::I128(self.as_ref().parse_as()?),
            IntegerKind::Isize => IntegerValue::Isize(self.as_ref().parse_as()?),
            IntegerKind::U8 => IntegerValue::U8(self.as_ref().parse_as()?),
            IntegerKind::U16 => IntegerValue::U16(self.as_ref().parse_as()?),
            IntegerKind::U32 => IntegerValue::U32(self.as_ref().parse_as()?),
            IntegerKind::U64 => IntegerValue::U64(self.as_ref().parse_as()?),
            IntegerKind::U128 => IntegerValue::U128(self.as_ref().parse_as()?),
            IntegerKind::Usize => IntegerValue::Usize(self.as_ref().parse_as()?),
        })
    }
}

impl<'a> SpannedAnyRef<'a, UntypedInteger> {
    pub(crate) fn parse_as<N>(self) -> ExecutionResult<N>
    where
        N: FromStr,
        N::Err: core::fmt::Display,
    {
        self.value.0.base10_digits().parse().map_err(|err| {
            self.span_range.value_error(format!(
                "Could not parse as {}: {}",
                core::any::type_name::<N>(),
                err
            ))
        })
    }
}

impl HasValueKind for UntypedInteger {
    type SpecificKind = IntegerKind;

    fn kind(&self) -> IntegerKind {
        IntegerKind::Untyped
    }
}

impl IntoValue for UntypedInteger {
    fn into_value(self) -> Value {
        Value::Integer(IntegerValue::Untyped(self))
    }
}

define_interface! {
    struct UntypedIntegerTypeData,
    parent: IntegerTypeData,
    pub(crate) mod untyped_integer_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
            fn neg(this: Owned<UntypedInteger>) -> ExecutionResult<UntypedInteger> {
                let (value, span_range) = this.deconstruct();
                let input = value.parse_fallback()?;
                match input.checked_neg() {
                    Some(negated) => Ok(UntypedInteger::from_fallback(negated)),
                    None => span_range.value_err("Negating this value would overflow in i128 space"),
                }
            }

            fn cast_to_untyped_integer(input: UntypedIntegerFallback) -> UntypedInteger {
                UntypedInteger::from_fallback(input.0)
            }

            fn cast_to_i8(input: UntypedIntegerFallback) -> i8 {
                input.0 as i8
            }

            fn cast_to_i16(input: UntypedIntegerFallback) -> i16 {
                input.0 as i16
            }

            fn cast_to_i32(input: UntypedIntegerFallback) -> i32 {
                input.0 as i32
            }

            fn cast_to_i64(input: UntypedIntegerFallback) -> i64 {
                input.0 as i64
            }

            fn cast_to_i128(input: UntypedIntegerFallback) -> i128 {
                input.0
            }

            fn cast_to_isize(input: UntypedIntegerFallback) -> isize {
                input.0 as isize
            }

            fn cast_to_u8(input: UntypedIntegerFallback) -> u8 {
                input.0 as u8
            }

            fn cast_to_u16(input: UntypedIntegerFallback) -> u16 {
                input.0 as u16
            }

            fn cast_to_u32(input: UntypedIntegerFallback) -> u32 {
                input.0 as u32
            }

            fn cast_to_u64(input: UntypedIntegerFallback) -> u64 {
                input.0 as u64
            }

            fn cast_to_u128(input: UntypedIntegerFallback) -> u128 {
                input.0 as u128
            }

            fn cast_to_usize(input: UntypedIntegerFallback) -> usize {
                input.0 as usize
            }

            fn cast_to_untyped_float(input: UntypedIntegerFallback) -> UntypedFloat {
                UntypedFloat::from_fallback(input.0 as FallbackFloat)
            }

            fn cast_to_f32(input: UntypedIntegerFallback) -> f32 {
                input.0 as f32
            }

            fn cast_to_f64(input: UntypedIntegerFallback) -> f64 {
                input.0 as f64
            }

            fn cast_to_string(input: UntypedIntegerFallback) -> String {
                input.0.to_string()
            }
        }
        pub(crate) mod binary_operations {
            [context] fn eq(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerValue>,
            ) -> ExecutionResult<bool> {
                UntypedInteger::paired_comparison(lhs, rhs, context, |a, b| a == b)
            }

            [context] fn ne(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerValue>,
            ) -> ExecutionResult<bool> {
                UntypedInteger::paired_comparison(lhs, rhs, context, |a, b| a != b)
            }

            [context] fn lt(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerValue>,
            ) -> ExecutionResult<bool> {
                UntypedInteger::paired_comparison(lhs, rhs, context, |a, b| a < b)
            }

            [context] fn le(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerValue>,
            ) -> ExecutionResult<bool> {
                UntypedInteger::paired_comparison(lhs, rhs, context, |a, b| a <= b)
            }

            [context] fn ge(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerValue>,
            ) -> ExecutionResult<bool> {
                UntypedInteger::paired_comparison(lhs, rhs, context, |a, b| a >= b)
            }

            [context] fn gt(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerValue>,
            ) -> ExecutionResult<bool> {
                UntypedInteger::paired_comparison(lhs, rhs, context, |a, b| a > b)
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
                    // Most operations are defined on the integer value directly
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

pub(crate) struct UntypedIntegerFallback(pub(crate) FallbackInteger);

impl ResolvableArgumentTarget for UntypedIntegerFallback {
    type ValueType = UntypedIntegerTypeData;
}

impl ResolvableOwned<Value> for UntypedIntegerFallback {
    fn resolve_from_value(input_value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
        let value: UntypedInteger =
            ResolvableOwned::<Value>::resolve_from_value(input_value, context)?;
        Ok(UntypedIntegerFallback(value.parse_fallback()?))
    }
}

impl ResolvableOwned<IntegerValue> for UntypedInteger {
    fn resolve_from_value(
        value: IntegerValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        match value {
            IntegerValue::Untyped(value) => Ok(value),
            _ => context.err("untyped integer", value),
        }
    }
}

impl_resolvable_argument_for! {
    UntypedIntegerTypeData,
    (value, context) -> UntypedInteger {
        match value {
            Value::Integer(IntegerValue::Untyped(x)) => Ok(x),
            _ => context.err("untyped integer", value),
        }
    }
}
