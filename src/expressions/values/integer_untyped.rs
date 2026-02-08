use super::*;

define_leaf_type! {
    pub(crate) UntypedIntegerType => IntegerType(IntegerContent::Untyped) => AnyType,
    content: UntypedInteger,
    kind: pub(crate) UntypedIntegerKind,
    type_name: "untyped_int",
    articled_value_name: "an untyped integer",
    dyn_impls: {},
}

#[derive(Copy, Clone)]
pub(crate) struct UntypedInteger(FallbackInteger);
pub(crate) type FallbackInteger = i128;

impl UntypedInteger {
    pub(super) fn new_from_lit_int(lit_int: &LitInt) -> ParseResult<Self> {
        Ok(Self(lit_int.base10_digits().parse().map_err(|err| {
            lit_int.parse_error(format!(
                "Untyped integers in preinterpret must fit inside a {}: {}",
                core::any::type_name::<FallbackInteger>(),
                err
            ))
        })?))
    }

    fn binary_overflow_error(
        context: BinaryOperationCallContext,
        lhs: impl std::fmt::Display,
        rhs: impl std::fmt::Display,
    ) -> FunctionError {
        context.error(format!(
            "The untyped integer operation {} {} {} overflowed in {} space",
            lhs,
            context.operation.symbolic_description(),
            rhs,
            core::any::type_name::<FallbackInteger>(),
        ))
    }

    /// Tries to convert to a specific integer kind, returning None if the value doesn't fit.
    pub(crate) fn try_into_kind(self, kind: IntegerLeafKind) -> Option<IntegerValue> {
        let value = self.0;
        Some(match kind {
            IntegerLeafKind::Untyped(_) => IntegerValue::Untyped(UntypedInteger(value)),
            IntegerLeafKind::I8(_) => IntegerValue::I8(value.try_into().ok()?),
            IntegerLeafKind::I16(_) => IntegerValue::I16(value.try_into().ok()?),
            IntegerLeafKind::I32(_) => IntegerValue::I32(value.try_into().ok()?),
            IntegerLeafKind::I64(_) => IntegerValue::I64(value.try_into().ok()?),
            IntegerLeafKind::I128(_) => IntegerValue::I128(value),
            IntegerLeafKind::Isize(_) => IntegerValue::Isize(value.try_into().ok()?),
            IntegerLeafKind::U8(_) => IntegerValue::U8(value.try_into().ok()?),
            IntegerLeafKind::U16(_) => IntegerValue::U16(value.try_into().ok()?),
            IntegerLeafKind::U32(_) => IntegerValue::U32(value.try_into().ok()?),
            IntegerLeafKind::U64(_) => IntegerValue::U64(value.try_into().ok()?),
            IntegerLeafKind::U128(_) => IntegerValue::U128(value.try_into().ok()?),
            IntegerLeafKind::Usize(_) => IntegerValue::Usize(value.try_into().ok()?),
        })
    }

    pub(crate) fn paired_operation(
        self,
        rhs: Spanned<IntegerValue>,
        context: BinaryOperationCallContext,
        perform_fn: fn(FallbackInteger, FallbackInteger) -> Option<FallbackInteger>,
    ) -> FunctionResult<IntegerValue> {
        let lhs = self.0;
        let rhs: UntypedInteger = rhs.downcast_resolve("This operand")?;
        let rhs = rhs.0;
        let output = perform_fn(lhs, rhs)
            .ok_or_else(|| UntypedInteger::binary_overflow_error(context, lhs, rhs))?;
        Ok(IntegerValue::Untyped(UntypedInteger::from_fallback(output)))
    }

    pub(crate) fn paired_comparison(
        self,
        rhs: Spanned<IntegerValue>,
        compare_fn: fn(FallbackInteger, FallbackInteger) -> bool,
    ) -> FunctionResult<bool> {
        let lhs = self.0;
        let rhs: UntypedInteger = rhs.downcast_resolve("This operand")?;
        let rhs = rhs.0;
        Ok(compare_fn(lhs, rhs))
    }

    pub(crate) fn shift_operation(
        self,
        rhs: u32,
        context: BinaryOperationCallContext,
        perform_fn: fn(FallbackInteger, u32) -> Option<FallbackInteger>,
    ) -> FunctionResult<IntegerValue> {
        let lhs = self.0;
        let output = perform_fn(lhs, rhs)
            .ok_or_else(|| UntypedInteger::binary_overflow_error(context, lhs, rhs))?;
        Ok(IntegerValue::Untyped(UntypedInteger::from_fallback(output)))
    }

    pub(crate) fn from_fallback(value: FallbackInteger) -> Self {
        Self(value)
    }

    pub(super) fn into_fallback(self) -> FallbackInteger {
        self.0
    }

    pub(super) fn to_unspanned_literal(self) -> Literal {
        Literal::i128_unsuffixed(self.0)
    }
}

impl Spanned<UntypedInteger> {
    pub(crate) fn into_kind(self, kind: IntegerLeafKind) -> FunctionResult<IntegerValue> {
        let Spanned(value, span_range) = self;
        value
            .try_into_kind(kind)
            .ok_or_else(|| -> FunctionError {
                span_range.value_error(format!(
                    "The integer value {} does not fit into {}",
                    value.0,
                    kind.articled_value_name()
                ))
            })
    }
}

define_type_features! {
    impl UntypedIntegerType,
    pub(crate) mod untyped_integer_interface {
        unary_operations {
            fn neg(Spanned(value, span): Spanned<UntypedInteger>) -> FunctionResult<UntypedInteger> {
                let input = value.into_fallback();
                match input.checked_neg() {
                    Some(negated) => Ok(UntypedInteger::from_fallback(negated)),
                    None => span.value_err("Negating this value would overflow in i128 space"),
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
                    },
                    _ => return None,
                })
            }

            fn resolve_own_binary_operation(
                _operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                // All operations are defined on the parent IntegerValue type
                None
            }
        }
    }
}

pub(crate) struct UntypedIntegerFallback(pub(crate) FallbackInteger);

impl IsValueContent for UntypedIntegerFallback {
    type Type = IntegerType;
    type Form = BeOwned;
}

impl IsArgument for UntypedIntegerFallback {
    type ValueType = UntypedIntegerType;
    const OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;
    fn from_argument(value: Spanned<ArgumentValue>) -> FunctionResult<Self> {
        Self::resolve_value(value.expect_owned(), "This argument")
    }
}

impl ResolvableArgumentTarget for UntypedIntegerFallback {
    type ValueType = UntypedIntegerType;
}

impl ResolvableOwned<AnyValue> for UntypedIntegerFallback {
    fn resolve_from_value(
        input_value: AnyValue,
        context: ResolutionContext,
    ) -> FunctionResult<Self> {
        let value: UntypedInteger =
            ResolvableOwned::<AnyValue>::resolve_from_value(input_value, context)?;
        Ok(UntypedIntegerFallback(value.into_fallback()))
    }
}

impl ResolvableOwned<IntegerValue> for UntypedInteger {
    fn resolve_from_value(
        value: IntegerValue,
        context: ResolutionContext,
    ) -> FunctionResult<Self> {
        match value {
            IntegerValue::Untyped(value) => Ok(value),
            _ => context.err("an untyped integer", value),
        }
    }
}

impl_resolvable_argument_for! {
    UntypedIntegerType,
    (value, context) -> UntypedInteger {
        match value {
            AnyValue::Integer(IntegerValue::Untyped(x)) => Ok(x),
            _ => context.err("an untyped integer", value),
        }
    }
}
