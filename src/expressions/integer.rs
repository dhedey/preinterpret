use super::*;

#[derive(Clone)]
pub(crate) struct EvaluationInteger {
    pub(super) value: EvaluationIntegerValue,
    /// The span of the source code that generated this boolean value.
    /// It may not have a value if generated from a complex expression.
    pub(super) source_span: Option<Span>,
}

impl EvaluationInteger {
    pub(super) fn for_litint(lit: syn::LitInt) -> ParseResult<Self> {
        let source_span = Some(lit.span());
        Ok(Self {
            value: EvaluationIntegerValue::for_litint(lit)?,
            source_span,
        })
    }

    pub(crate) fn try_into_i128(self) -> Option<i128> {
        match self.value {
            EvaluationIntegerValue::Untyped(x) => x.parse_fallback().ok(),
            EvaluationIntegerValue::U8(x) => Some(x.into()),
            EvaluationIntegerValue::U16(x) => Some(x.into()),
            EvaluationIntegerValue::U32(x) => Some(x.into()),
            EvaluationIntegerValue::U64(x) => Some(x.into()),
            EvaluationIntegerValue::U128(x) => x.try_into().ok(),
            EvaluationIntegerValue::Usize(x) => x.try_into().ok(),
            EvaluationIntegerValue::I8(x) => Some(x.into()),
            EvaluationIntegerValue::I16(x) => Some(x.into()),
            EvaluationIntegerValue::I32(x) => Some(x.into()),
            EvaluationIntegerValue::I64(x) => Some(x.into()),
            EvaluationIntegerValue::I128(x) => Some(x),
            EvaluationIntegerValue::Isize(x) => x.try_into().ok(),
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match self.value {
            EvaluationIntegerValue::Untyped(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::U8(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::U16(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::U32(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::U64(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::U128(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::Usize(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::I8(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::I16(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::I32(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::I64(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::I128(input) => input.handle_unary_operation(&operation),
            EvaluationIntegerValue::Isize(input) => input.handle_unary_operation(&operation),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: EvaluationInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match self.value {
            EvaluationIntegerValue::Untyped(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::U8(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::U16(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::U32(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::U64(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::U128(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::Usize(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::I8(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::I16(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::I32(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::I64(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::I128(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            EvaluationIntegerValue::Isize(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
        }
    }

    pub(super) fn to_literal(&self, fallback_span: Span) -> Literal {
        self.value
            .to_unspanned_literal()
            .with_span(self.source_span.unwrap_or(fallback_span))
    }
}

pub(super) enum EvaluationIntegerValuePair {
    Untyped(UntypedInteger, UntypedInteger),
    U8(u8, u8),
    U16(u16, u16),
    U32(u32, u32),
    U64(u64, u64),
    U128(u128, u128),
    Usize(usize, usize),
    I8(i8, i8),
    I16(i16, i16),
    I32(i32, i32),
    I64(i64, i64),
    I128(i128, i128),
    Isize(isize, isize),
}

impl EvaluationIntegerValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: &PairedBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match self {
            Self::Untyped(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::U8(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::U16(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::U32(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::U64(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::U128(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::Usize(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::I8(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::I16(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::I32(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::I64(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::I128(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::Isize(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum IntegerKind {
    Untyped,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

#[derive(Clone)]
pub(super) enum EvaluationIntegerValue {
    Untyped(UntypedInteger),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
}

impl EvaluationIntegerValue {
    pub(super) fn for_litint(lit: syn::LitInt) -> ParseResult<Self> {
        Ok(match lit.suffix() {
            "" => Self::Untyped(UntypedInteger::new_from_lit_int(lit)),
            "u8" => Self::U8(lit.base10_parse()?),
            "u16" => Self::U16(lit.base10_parse()?),
            "u32" => Self::U32(lit.base10_parse()?),
            "u64" => Self::U64(lit.base10_parse()?),
            "u128" => Self::U128(lit.base10_parse()?),
            "usize" => Self::Usize(lit.base10_parse()?),
            "i8" => Self::I8(lit.base10_parse()?),
            "i16" => Self::I16(lit.base10_parse()?),
            "i32" => Self::I32(lit.base10_parse()?),
            "i64" => Self::I64(lit.base10_parse()?),
            "i128" => Self::I128(lit.base10_parse()?),
            "isize" => Self::Isize(lit.base10_parse()?),
            suffix => {
                return lit.span().parse_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        })
    }

    pub(super) fn describe_type(&self) -> &'static str {
        match self {
            EvaluationIntegerValue::Untyped(_) => "untyped integer",
            EvaluationIntegerValue::U8(_) => "u8",
            EvaluationIntegerValue::U16(_) => "u16",
            EvaluationIntegerValue::U32(_) => "u32",
            EvaluationIntegerValue::U64(_) => "u64",
            EvaluationIntegerValue::U128(_) => "u128",
            EvaluationIntegerValue::Usize(_) => "usize",
            EvaluationIntegerValue::I8(_) => "i8",
            EvaluationIntegerValue::I16(_) => "i16",
            EvaluationIntegerValue::I32(_) => "i32",
            EvaluationIntegerValue::I64(_) => "i64",
            EvaluationIntegerValue::I128(_) => "i128",
            EvaluationIntegerValue::Isize(_) => "isize",
        }
    }

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            EvaluationIntegerValue::Untyped(int) => int.to_unspanned_literal(),
            EvaluationIntegerValue::U8(int) => Literal::u8_suffixed(*int),
            EvaluationIntegerValue::U16(int) => Literal::u16_suffixed(*int),
            EvaluationIntegerValue::U32(int) => Literal::u32_suffixed(*int),
            EvaluationIntegerValue::U64(int) => Literal::u64_suffixed(*int),
            EvaluationIntegerValue::U128(int) => Literal::u128_suffixed(*int),
            EvaluationIntegerValue::Usize(int) => Literal::usize_suffixed(*int),
            EvaluationIntegerValue::I8(int) => Literal::i8_suffixed(*int),
            EvaluationIntegerValue::I16(int) => Literal::i16_suffixed(*int),
            EvaluationIntegerValue::I32(int) => Literal::i32_suffixed(*int),
            EvaluationIntegerValue::I64(int) => Literal::i64_suffixed(*int),
            EvaluationIntegerValue::I128(int) => Literal::i128_suffixed(*int),
            EvaluationIntegerValue::Isize(int) => Literal::isize_suffixed(*int),
        }
    }
}

#[derive(Clone)]
pub(super) struct UntypedInteger(
    /// The span of the literal is ignored, and will be set when converted to an output.
    syn::LitInt,
);
pub(super) type FallbackInteger = i128;

impl UntypedInteger {
    pub(super) fn new_from_lit_int(lit_int: LitInt) -> Self {
        Self(lit_int)
    }

    pub(super) fn new_from_literal(literal: Literal) -> Self {
        Self(syn::LitInt::from(literal))
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: &UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        let input = self.parse_fallback()?;
        match operation {
            UnaryOperation::Neg { .. } => operation.output(Self::from_fallback(-input)),
            UnaryOperation::Not { .. } => {
                operation.unsupported_for_value_type_err("untyped integer")
            }
            UnaryOperation::GroupedNoOp { .. } => operation.output(self),
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    operation.output(UntypedInteger::from_fallback(input as FallbackInteger))
                }
                CastTarget::Integer(IntegerKind::I8) => operation.output(input as i8),
                CastTarget::Integer(IntegerKind::I16) => operation.output(input as i16),
                CastTarget::Integer(IntegerKind::I32) => operation.output(input as i32),
                CastTarget::Integer(IntegerKind::I64) => operation.output(input as i64),
                CastTarget::Integer(IntegerKind::I128) => operation.output(input),
                CastTarget::Integer(IntegerKind::Isize) => operation.output(input as isize),
                CastTarget::Integer(IntegerKind::U8) => operation.output(input as u8),
                CastTarget::Integer(IntegerKind::U16) => operation.output(input as u16),
                CastTarget::Integer(IntegerKind::U32) => operation.output(input as u32),
                CastTarget::Integer(IntegerKind::U64) => operation.output(input as u64),
                CastTarget::Integer(IntegerKind::U128) => operation.output(input as u128),
                CastTarget::Integer(IntegerKind::Usize) => operation.output(input as usize),
                CastTarget::Float(FloatKind::Untyped) => {
                    operation.output(UntypedFloat::from_fallback(input as FallbackFloat))
                }
                CastTarget::Float(FloatKind::F32) => operation.output(input as f32),
                CastTarget::Float(FloatKind::F64) => operation.output(input as f64),
                CastTarget::Boolean | CastTarget::Char => {
                    operation.execution_err("This cast is not supported")
                }
            },
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        rhs: EvaluationInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        let lhs = self.parse_fallback()?;
        match operation {
            IntegerBinaryOperation::ShiftLeft { .. } => match rhs.value {
                EvaluationIntegerValue::Untyped(rhs) => {
                    operation.output(lhs << rhs.parse_fallback()?)
                }
                EvaluationIntegerValue::U8(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::U16(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::U32(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::U64(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::U128(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::Usize(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::I8(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::I16(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::I32(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::I64(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::I128(rhs) => operation.output(lhs << rhs),
                EvaluationIntegerValue::Isize(rhs) => operation.output(lhs << rhs),
            },
            IntegerBinaryOperation::ShiftRight { .. } => match rhs.value {
                EvaluationIntegerValue::Untyped(rhs) => {
                    operation.output(lhs >> rhs.parse_fallback()?)
                }
                EvaluationIntegerValue::U8(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::U16(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::U32(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::U64(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::U128(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::Usize(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::I8(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::I16(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::I32(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::I64(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::I128(rhs) => operation.output(lhs >> rhs),
                EvaluationIntegerValue::Isize(rhs) => operation.output(lhs >> rhs),
            },
        }
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &PairedBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        let lhs = self.parse_fallback()?;
        let rhs = rhs.parse_fallback()?;
        let overflow_error = || {
            format!(
                "The untyped integer operation {:?} {} {:?} overflowed in i128 space",
                lhs,
                operation.symbol(),
                rhs
            )
        };
        match operation {
            PairedBinaryOperation::Addition { .. } => operation.output_if_some(
                lhs.checked_add(rhs).map(Self::from_fallback),
                overflow_error,
            ),
            PairedBinaryOperation::Subtraction { .. } => operation.output_if_some(
                lhs.checked_sub(rhs).map(Self::from_fallback),
                overflow_error,
            ),
            PairedBinaryOperation::Multiplication { .. } => operation.output_if_some(
                lhs.checked_mul(rhs).map(Self::from_fallback),
                overflow_error,
            ),
            PairedBinaryOperation::Division { .. } => operation.output_if_some(
                lhs.checked_div(rhs).map(Self::from_fallback),
                overflow_error,
            ),
            PairedBinaryOperation::LogicalAnd { .. } | PairedBinaryOperation::LogicalOr { .. } => {
                operation.unsupported_for_value_type_err("untyped integer")
            }
            PairedBinaryOperation::Remainder { .. } => operation.output_if_some(
                lhs.checked_rem(rhs).map(Self::from_fallback),
                overflow_error,
            ),
            PairedBinaryOperation::BitXor { .. } => {
                operation.output(Self::from_fallback(lhs ^ rhs))
            }
            PairedBinaryOperation::BitAnd { .. } => {
                operation.output(Self::from_fallback(lhs & rhs))
            }
            PairedBinaryOperation::BitOr { .. } => operation.output(Self::from_fallback(lhs | rhs)),
            PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
            PairedBinaryOperation::LessThan { .. } => operation.output(lhs < rhs),
            PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
            PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
            PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
            PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs > rhs),
        }
    }

    pub(super) fn from_fallback(value: FallbackInteger) -> Self {
        Self::new_from_literal(Literal::i128_unsuffixed(value))
    }

    pub(super) fn parse_fallback(&self) -> ExecutionResult<FallbackInteger> {
        self.0.base10_digits().parse().map_err(|err| {
            self.0.span().execution_error(format!(
                "Could not parse as the default inferred type {}: {}",
                core::any::type_name::<FallbackInteger>(),
                err
            ))
        })
    }

    pub(super) fn parse_as<N>(&self) -> ExecutionResult<N>
    where
        N: FromStr,
        N::Err: core::fmt::Display,
    {
        self.0.base10_digits().parse().map_err(|err| {
            self.0.span().execution_error(format!(
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

impl ToEvaluationValue for UntypedInteger {
    fn to_value(self, source_span: Option<Span>) -> EvaluationValue {
        EvaluationValue::Integer(EvaluationInteger {
            value: EvaluationIntegerValue::Untyped(self),
            source_span,
        })
    }
}

// We have to use a macro because we don't have checked xx traits :(
macro_rules! impl_int_operations_except_unary {
    (
        $($integer_enum_variant:ident($integer_type:ident)),* $(,)?
    ) => {$(
        impl ToEvaluationValue for $integer_type {
            fn to_value(self, source_span: Option<Span>) -> EvaluationValue {
                EvaluationValue::Integer(EvaluationInteger {
                    value: EvaluationIntegerValue::$integer_enum_variant(self),
                    source_span,
                })
            }
        }

        impl HandleBinaryOperation for $integer_type {
            fn handle_paired_binary_operation(self, rhs: Self, operation: &PairedBinaryOperation) -> ExecutionResult<EvaluationValue> {
                let lhs = self;
                let overflow_error = || format!("The {} operation {:?} {} {:?} overflowed", stringify!($integer_type), lhs, operation.symbol(), rhs);
                match operation {
                    PairedBinaryOperation::Addition { .. } => operation.output_if_some(lhs.checked_add(rhs), overflow_error),
                    PairedBinaryOperation::Subtraction { .. } => operation.output_if_some(lhs.checked_sub(rhs), overflow_error),
                    PairedBinaryOperation::Multiplication { .. } => operation.output_if_some(lhs.checked_mul(rhs), overflow_error),
                    PairedBinaryOperation::Division { .. } => operation.output_if_some(lhs.checked_div(rhs), overflow_error),
                    PairedBinaryOperation::LogicalAnd { .. }
                    | PairedBinaryOperation::LogicalOr { .. } => operation.unsupported_for_value_type_err(stringify!($integer_type)),
                    PairedBinaryOperation::Remainder { .. } => operation.output_if_some(lhs.checked_rem(rhs), overflow_error),
                    PairedBinaryOperation::BitXor { .. } => operation.output(lhs ^ rhs),
                    PairedBinaryOperation::BitAnd { .. } => operation.output(lhs & rhs),
                    PairedBinaryOperation::BitOr { .. } => operation.output(lhs | rhs),
                    PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
                    PairedBinaryOperation::LessThan { .. } => operation.output(lhs < rhs),
                    PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
                    PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
                    PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
                    PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs > rhs),
                }
            }

            fn handle_integer_binary_operation(
                self,
                rhs: EvaluationInteger,
                operation: &IntegerBinaryOperation,
            ) -> ExecutionResult<EvaluationValue> {
                let lhs = self;
                match operation {
                    IntegerBinaryOperation::ShiftLeft { .. } => {
                        match rhs.value {
                            EvaluationIntegerValue::Untyped(rhs) => operation.output(lhs << rhs.parse_fallback()?),
                            EvaluationIntegerValue::U8(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::U16(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::U32(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::U64(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::U128(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::Usize(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::I8(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::I16(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::I32(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::I64(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::I128(rhs) => operation.output(lhs << rhs),
                            EvaluationIntegerValue::Isize(rhs) => operation.output(lhs << rhs),
                        }
                    },
                    IntegerBinaryOperation::ShiftRight { .. } => {
                        match rhs.value {
                            EvaluationIntegerValue::Untyped(rhs) => operation.output(lhs >> rhs.parse_fallback()?),
                            EvaluationIntegerValue::U8(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::U16(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::U32(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::U64(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::U128(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::Usize(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::I8(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::I16(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::I32(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::I64(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::I128(rhs) => operation.output(lhs >> rhs),
                            EvaluationIntegerValue::Isize(rhs) => operation.output(lhs >> rhs),
                        }
                    },
                }
            }
        }
    )*};
}

macro_rules! impl_unsigned_unary_operations {
    ($($integer_type:ident),* $(,)?) => {$(
        impl HandleUnaryOperation for $integer_type {
            fn handle_unary_operation(self, operation: &UnaryOperation) -> ExecutionResult<EvaluationValue> {
                match operation {
                    UnaryOperation::GroupedNoOp { .. } => operation.output(self),
                    UnaryOperation::Neg { .. }
                    | UnaryOperation::Not { .. } => {
                        operation.unsupported_for_value_type_err(stringify!($integer_type))
                    },
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::Integer(IntegerKind::Untyped) => operation.output(UntypedInteger::from_fallback(self as FallbackInteger)),
                        CastTarget::Integer(IntegerKind::I8) => operation.output(self as i8),
                        CastTarget::Integer(IntegerKind::I16) => operation.output(self as i16),
                        CastTarget::Integer(IntegerKind::I32) => operation.output(self as i32),
                        CastTarget::Integer(IntegerKind::I64) => operation.output(self as i64),
                        CastTarget::Integer(IntegerKind::I128) => operation.output(self as i128),
                        CastTarget::Integer(IntegerKind::Isize) => operation.output(self as isize),
                        CastTarget::Integer(IntegerKind::U8) => operation.output(self as u8),
                        CastTarget::Integer(IntegerKind::U16) => operation.output(self as u16),
                        CastTarget::Integer(IntegerKind::U32) => operation.output(self as u32),
                        CastTarget::Integer(IntegerKind::U64) => operation.output(self as u64),
                        CastTarget::Integer(IntegerKind::U128) => operation.output(self as u128),
                        CastTarget::Integer(IntegerKind::Usize) => operation.output(self as usize),
                        CastTarget::Float(FloatKind::Untyped) => operation.output(UntypedFloat::from_fallback(self as FallbackFloat)),
                        CastTarget::Float(FloatKind::F32) => operation.output(self as f32),
                        CastTarget::Float(FloatKind::F64) => operation.output(self as f64),
                        CastTarget::Boolean | CastTarget::Char => operation.execution_err("This cast is not supported"),
                    }
                }
            }
        }
    )*};
}

macro_rules! impl_signed_unary_operations {
    ($($integer_type:ident),* $(,)?) => {$(
        impl HandleUnaryOperation for $integer_type {
            fn handle_unary_operation(self, operation: &UnaryOperation) -> ExecutionResult<EvaluationValue> {
                match operation {
                    UnaryOperation::GroupedNoOp { .. } => operation.output(self),
                    UnaryOperation::Neg { .. } => operation.output(-self),
                    UnaryOperation::Not { .. } => {
                        operation.unsupported_for_value_type_err(stringify!($integer_type))
                    },
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::Integer(IntegerKind::Untyped) => operation.output(UntypedInteger::from_fallback(self as FallbackInteger)),
                        CastTarget::Integer(IntegerKind::I8) => operation.output(self as i8),
                        CastTarget::Integer(IntegerKind::I16) => operation.output(self as i16),
                        CastTarget::Integer(IntegerKind::I32) => operation.output(self as i32),
                        CastTarget::Integer(IntegerKind::I64) => operation.output(self as i64),
                        CastTarget::Integer(IntegerKind::I128) => operation.output(self as i128),
                        CastTarget::Integer(IntegerKind::Isize) => operation.output(self as isize),
                        CastTarget::Integer(IntegerKind::U8) => operation.output(self as u8),
                        CastTarget::Integer(IntegerKind::U16) => operation.output(self as u16),
                        CastTarget::Integer(IntegerKind::U32) => operation.output(self as u32),
                        CastTarget::Integer(IntegerKind::U64) => operation.output(self as u64),
                        CastTarget::Integer(IntegerKind::U128) => operation.output(self as u128),
                        CastTarget::Integer(IntegerKind::Usize) => operation.output(self as usize),
                        CastTarget::Float(FloatKind::Untyped) => operation.output(UntypedFloat::from_fallback(self as FallbackFloat)),
                        CastTarget::Float(FloatKind::F32) => operation.output(self as f32),
                        CastTarget::Float(FloatKind::F64) => operation.output(self as f64),
                        CastTarget::Boolean | CastTarget::Char => operation.execution_err("This cast is not supported"),
                    }
                }
            }
        }
    )*};
}

impl HandleUnaryOperation for u8 {
    fn handle_unary_operation(
        self,
        operation: &UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match operation {
            UnaryOperation::GroupedNoOp { .. } => operation.output(self),
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => {
                operation.unsupported_for_value_type_err("u8")
            }
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    operation.output(UntypedInteger::from_fallback(self as FallbackInteger))
                }
                CastTarget::Integer(IntegerKind::I8) => operation.output(self as i8),
                CastTarget::Integer(IntegerKind::I16) => operation.output(self as i16),
                CastTarget::Integer(IntegerKind::I32) => operation.output(self as i32),
                CastTarget::Integer(IntegerKind::I64) => operation.output(self as i64),
                CastTarget::Integer(IntegerKind::I128) => operation.output(self as i128),
                CastTarget::Integer(IntegerKind::Isize) => operation.output(self as isize),
                CastTarget::Integer(IntegerKind::U8) => operation.output(self),
                CastTarget::Integer(IntegerKind::U16) => operation.output(self as u16),
                CastTarget::Integer(IntegerKind::U32) => operation.output(self as u32),
                CastTarget::Integer(IntegerKind::U64) => operation.output(self as u64),
                CastTarget::Integer(IntegerKind::U128) => operation.output(self as u128),
                CastTarget::Integer(IntegerKind::Usize) => operation.output(self as usize),
                CastTarget::Float(FloatKind::Untyped) => {
                    operation.output(UntypedFloat::from_fallback(self as FallbackFloat))
                }
                CastTarget::Float(FloatKind::F32) => operation.output(self as f32),
                CastTarget::Float(FloatKind::F64) => operation.output(self as f64),
                CastTarget::Char => operation.output(self as char),
                CastTarget::Boolean => operation.execution_err("This cast is not supported"),
            },
        }
    }
}

impl_int_operations_except_unary!(
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
);

// U8 has a char cast so is handled separately
impl_unsigned_unary_operations!(u16, u32, u64, u128, usize);
impl_signed_unary_operations!(i8, i16, i32, i64, i128, isize);
