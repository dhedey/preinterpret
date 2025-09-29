use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionInteger {
    pub(super) value: ExpressionIntegerValue,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(super) span_range: SpanRange,
}

impl ExpressionInteger {
    pub(super) fn for_litint(lit: &syn::LitInt) -> ParseResult<Self> {
        Ok(Self {
            span_range: lit.span().span_range(),
            value: ExpressionIntegerValue::for_litint(lit)?,
        })
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self.value {
            ExpressionIntegerValue::Untyped(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::U8(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::U16(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::U32(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::U64(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::U128(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::Usize(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::I8(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::I16(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::I32(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::I64(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::I128(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionIntegerValue::Isize(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
        }
    }

    pub(crate) fn expect_usize(&self) -> ExecutionResult<usize> {
        Ok(match &self.value {
            ExpressionIntegerValue::Untyped(input) => input.parse_as()?,
            ExpressionIntegerValue::Usize(input) => *input,
            _ => {
                return self
                    .span_range
                    .execution_err("Expected a usize or untyped integer")
            }
        })
    }

    pub(super) fn to_literal(&self) -> Literal {
        self.value
            .to_unspanned_literal()
            .with_span(self.span_range.join_into_span_else_start())
    }
}

impl HasValueType for ExpressionInteger {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct IntegerTypeData;

impl MethodResolutionTarget for IntegerTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);
}

pub(super) enum ExpressionIntegerValuePair {
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

impl ExpressionIntegerValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum IntegerKind {
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

impl IntegerKind {
    pub(super) fn method_resolver(&self) -> &'static dyn MethodResolver {
        static UNTYPED: UntypedIntegerTypeData = UntypedIntegerTypeData;
        static I8: I8TypeData = I8TypeData;
        static I16: I16TypeData = I16TypeData;
        static I32: I32TypeData = I32TypeData;
        static I64: I64TypeData = I64TypeData;
        static I128: I128TypeData = I128TypeData;
        static ISIZE: IsizeTypeData = IsizeTypeData;
        static U8: U8TypeData = U8TypeData;
        static U16: U16TypeData = U16TypeData;
        static U32: U32TypeData = U32TypeData;
        static U64: U64TypeData = U64TypeData;
        static U128: U128TypeData = U128TypeData;
        static USIZE: UsizeTypeData = UsizeTypeData;
        match self {
            IntegerKind::Untyped => &UNTYPED,
            IntegerKind::I8 => &I8,
            IntegerKind::I16 => &I16,
            IntegerKind::I32 => &I32,
            IntegerKind::I64 => &I64,
            IntegerKind::I128 => &I128,
            IntegerKind::Isize => &ISIZE,
            IntegerKind::U8 => &U8,
            IntegerKind::U16 => &U16,
            IntegerKind::U32 => &U32,
            IntegerKind::U64 => &U64,
            IntegerKind::U128 => &U128,
            IntegerKind::Usize => &USIZE,
        }
    }
}

#[derive(Clone)]
pub(super) enum ExpressionIntegerValue {
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

impl ExpressionIntegerValue {
    pub(super) fn kind(&self) -> IntegerKind {
        match self {
            Self::Untyped(_) => IntegerKind::Untyped,
            Self::U8(_) => IntegerKind::U8,
            Self::U16(_) => IntegerKind::U16,
            Self::U32(_) => IntegerKind::U32,
            Self::U64(_) => IntegerKind::U64,
            Self::U128(_) => IntegerKind::U128,
            Self::Usize(_) => IntegerKind::Usize,
            Self::I8(_) => IntegerKind::I8,
            Self::I16(_) => IntegerKind::I16,
            Self::I32(_) => IntegerKind::I32,
            Self::I64(_) => IntegerKind::I64,
            Self::I128(_) => IntegerKind::I128,
            Self::Isize(_) => IntegerKind::Isize,
        }
    }
}

impl ExpressionIntegerValue {
    pub(super) fn for_litint(lit: &syn::LitInt) -> ParseResult<Self> {
        Ok(match lit.suffix() {
            "" => Self::Untyped(UntypedInteger::new_from_lit_int(lit.clone())),
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

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            ExpressionIntegerValue::Untyped(int) => int.to_unspanned_literal(),
            ExpressionIntegerValue::U8(int) => Literal::u8_suffixed(*int),
            ExpressionIntegerValue::U16(int) => Literal::u16_suffixed(*int),
            ExpressionIntegerValue::U32(int) => Literal::u32_suffixed(*int),
            ExpressionIntegerValue::U64(int) => Literal::u64_suffixed(*int),
            ExpressionIntegerValue::U128(int) => Literal::u128_suffixed(*int),
            ExpressionIntegerValue::Usize(int) => Literal::usize_suffixed(*int),
            ExpressionIntegerValue::I8(int) => Literal::i8_suffixed(*int),
            ExpressionIntegerValue::I16(int) => Literal::i16_suffixed(*int),
            ExpressionIntegerValue::I32(int) => Literal::i32_suffixed(*int),
            ExpressionIntegerValue::I64(int) => Literal::i64_suffixed(*int),
            ExpressionIntegerValue::I128(int) => Literal::i128_suffixed(*int),
            ExpressionIntegerValue::Isize(int) => Literal::isize_suffixed(*int),
        }
    }
}

impl HasValueType for ExpressionIntegerValue {
    fn value_type(&self) -> &'static str {
        match self {
            ExpressionIntegerValue::Untyped(value) => value.value_type(),
            ExpressionIntegerValue::U8(value) => value.value_type(),
            ExpressionIntegerValue::U16(value) => value.value_type(),
            ExpressionIntegerValue::U32(value) => value.value_type(),
            ExpressionIntegerValue::U64(value) => value.value_type(),
            ExpressionIntegerValue::U128(value) => value.value_type(),
            ExpressionIntegerValue::Usize(value) => value.value_type(),
            ExpressionIntegerValue::I8(value) => value.value_type(),
            ExpressionIntegerValue::I16(value) => value.value_type(),
            ExpressionIntegerValue::I32(value) => value.value_type(),
            ExpressionIntegerValue::I64(value) => value.value_type(),
            ExpressionIntegerValue::I128(value) => value.value_type(),
            ExpressionIntegerValue::Isize(value) => value.value_type(),
        }
    }
}

impl HasValueType for UntypedInteger {
    fn value_type(&self) -> &'static str {
        "untyped integer"
    }
}

#[derive(Clone)]
pub(crate) struct UntypedInteger(
    /// The span of the literal is ignored, and will be set when converted to an output.
    syn::LitInt,
    SpanRange,
);
pub(crate) type FallbackInteger = i128;

impl UntypedInteger {
    pub(super) fn new_from_lit_int(lit_int: LitInt) -> Self {
        let span_range = lit_int.span().span_range();
        Self(lit_int, span_range)
    }

    pub(super) fn new_from_literal(literal: Literal) -> Self {
        Self::new_from_lit_int(literal.into())
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        rhs: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.parse_fallback()?;
        Ok(match operation.operation {
            IntegerBinaryOperation::ShiftLeft { .. } => match rhs.value {
                ExpressionIntegerValue::Untyped(rhs) => {
                    operation.output(lhs << rhs.parse_fallback()?)
                }
                ExpressionIntegerValue::U8(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::U16(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::U32(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::U64(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::U128(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::Usize(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::I8(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::I16(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::I32(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::I64(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::I128(rhs) => operation.output(lhs << rhs),
                ExpressionIntegerValue::Isize(rhs) => operation.output(lhs << rhs),
            },
            IntegerBinaryOperation::ShiftRight { .. } => match rhs.value {
                ExpressionIntegerValue::Untyped(rhs) => {
                    operation.output(lhs >> rhs.parse_fallback()?)
                }
                ExpressionIntegerValue::U8(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::U16(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::U32(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::U64(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::U128(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::Usize(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::I8(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::I16(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::I32(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::I64(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::I128(rhs) => operation.output(lhs >> rhs),
                ExpressionIntegerValue::Isize(rhs) => operation.output(lhs >> rhs),
            },
        })
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.parse_fallback()?;
        let rhs = rhs.parse_fallback()?;
        let overflow_error = || {
            format!(
                "The untyped integer operation {:?} {} {:?} overflowed in i128 space",
                lhs,
                operation.symbolic_description(),
                rhs
            )
        };
        Ok(match operation.operation {
            PairedBinaryOperation::Addition { .. } => {
                return operation.output_if_some(
                    lhs.checked_add(rhs).map(Self::from_fallback),
                    overflow_error,
                )
            }
            PairedBinaryOperation::Subtraction { .. } => {
                return operation.output_if_some(
                    lhs.checked_sub(rhs).map(Self::from_fallback),
                    overflow_error,
                )
            }
            PairedBinaryOperation::Multiplication { .. } => {
                return operation.output_if_some(
                    lhs.checked_mul(rhs).map(Self::from_fallback),
                    overflow_error,
                )
            }
            PairedBinaryOperation::Division { .. } => {
                return operation.output_if_some(
                    lhs.checked_div(rhs).map(Self::from_fallback),
                    overflow_error,
                )
            }
            PairedBinaryOperation::LogicalAnd { .. } | PairedBinaryOperation::LogicalOr { .. } => {
                return operation.unsupported(self);
            }
            PairedBinaryOperation::Remainder { .. } => {
                return operation.output_if_some(
                    lhs.checked_rem(rhs).map(Self::from_fallback),
                    overflow_error,
                )
            }
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
        })
    }

    pub(crate) fn from_fallback(value: FallbackInteger) -> Self {
        Self::new_from_literal(Literal::i128_unsuffixed(value))
    }

    pub(super) fn parse_fallback(&self) -> ExecutionResult<FallbackInteger> {
        self.0.base10_digits().parse().map_err(|err| {
            self.1.execution_error(format!(
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
            self.1.execution_error(format!(
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

impl ToExpressionValue for UntypedInteger {
    fn to_value(mut self, span_range: SpanRange) -> ExpressionValue {
        self.1 = span_range;
        ExpressionValue::Integer(ExpressionInteger {
            value: ExpressionIntegerValue::Untyped(self),
            span_range,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UntypedIntegerTypeData;

impl MethodResolutionTarget for UntypedIntegerTypeData {
    type Parent = IntegerTypeData;
    const PARENT: Option<Self::Parent> = Some(IntegerTypeData);

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Neg { .. } => {
                wrap_unary!((this: Owned<UntypedInteger>) -> ExecutionResult<UntypedInteger> {
                    let (value, span_range) = this.deconstruct();
                    let input = value.parse_fallback()?;
                    match input.checked_neg() {
                        Some(negated) => Ok(UntypedInteger::from_fallback(negated)),
                        None => span_range.execution_err("Negating this value would overflow in i128 space"),
                    }
                })
            }
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> UntypedInteger {
                        UntypedInteger::from_fallback(input.0)
                    })
                }
                CastTarget::Integer(IntegerKind::I8) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> i8 {
                        input.0 as i8
                    })
                }
                CastTarget::Integer(IntegerKind::I16) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> i16 {
                        input.0 as i16
                    })
                }
                CastTarget::Integer(IntegerKind::I32) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> i32 {
                        input.0 as i32
                    })
                }
                CastTarget::Integer(IntegerKind::I64) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> i64 {
                        input.0 as i64
                    })
                }
                CastTarget::Integer(IntegerKind::I128) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> i128 {
                        input.0
                    })
                }
                CastTarget::Integer(IntegerKind::Isize) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> isize {
                        input.0 as isize
                    })
                }
                CastTarget::Integer(IntegerKind::U8) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> u8 {
                        input.0 as u8
                    })
                }
                CastTarget::Integer(IntegerKind::U16) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> u16 {
                        input.0 as u16
                    })
                }
                CastTarget::Integer(IntegerKind::U32) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> u32 {
                        input.0 as u32
                    })
                }
                CastTarget::Integer(IntegerKind::U64) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> u64 {
                        input.0 as u64
                    })
                }
                CastTarget::Integer(IntegerKind::U128) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> u128 {
                        input.0 as u128
                    })
                }
                CastTarget::Integer(IntegerKind::Usize) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> usize {
                        input.0 as usize
                    })
                }
                CastTarget::Float(FloatKind::Untyped) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> UntypedFloat {
                        UntypedFloat::from_fallback(input.0 as FallbackFloat)
                    })
                }
                CastTarget::Float(FloatKind::F32) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> f32 {
                        input.0 as f32
                    })
                }
                CastTarget::Float(FloatKind::F64) => {
                    wrap_unary!((input: UntypedIntegerFallback) -> f64 {
                        input.0 as f64
                    })
                }
                CastTarget::String => {
                    wrap_unary!((input: UntypedIntegerFallback) -> String {
                        input.0.to_string()
                    })
                }
                _ => return None,
            },
            _ => return None,
        })
    }
}

// We have to use a macro because we don't have checked xx traits :(
macro_rules! impl_int_operations {
    (
        $($integer_type_data:ident: [$(CharCast[$($char_cast:ident)*],)?$(Signed[$($signed:ident)*],)?] $integer_enum_variant:ident($integer_type:ident)),* $(,)?
    ) => {$(
        #[derive(Clone, Copy)]
        pub(crate) struct $integer_type_data;

        impl MethodResolutionTarget for $integer_type_data {
            type Parent = IntegerTypeData;
            const PARENT: Option<Self::Parent> = Some(IntegerTypeData);

            #[allow(unreachable_code)]
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    $(
                        UnaryOperation::Neg { .. } => wrap_unary!((this: Owned<$integer_type>) -> ExecutionResult<$integer_type> {
                            ignore_all!($($signed)*); // Make it so that it is only defined for signed types
                            let (value, span_range) = this.deconstruct();
                            match value.checked_neg() {
                                Some(negated) => Ok(negated),
                                None => span_range.execution_err("Negating this value would overflow"),
                            }
                        }),
                    )?
                    UnaryOperation::Cast { target, .. } => match target {
                        $(
                            CastTarget::Char => {
                                wrap_unary!((input: $integer_type) -> char {
                                    ignore_all!($($char_cast)*); // Make it so that it is only defined for opted-in types
                                    input as char
                                })
                            }
                        )?
                        CastTarget::Integer(IntegerKind::Untyped) => {
                            wrap_unary!((input: $integer_type) -> UntypedInteger {
                                UntypedInteger::from_fallback(input as FallbackInteger)
                            })
                        }
                        CastTarget::Integer(IntegerKind::I8) => {
                            wrap_unary!((input: $integer_type) -> i8 {
                                input as i8
                            })
                        }
                        CastTarget::Integer(IntegerKind::I16) => {
                            wrap_unary!((input: $integer_type) -> i16 {
                                input as i16
                            })
                        }
                        CastTarget::Integer(IntegerKind::I32) => {
                            wrap_unary!((input: $integer_type) -> i32 {
                                input as i32
                            })
                        }
                        CastTarget::Integer(IntegerKind::I64) => {
                            wrap_unary!((input: $integer_type) -> i64 {
                                input as i64
                            })
                        }
                        CastTarget::Integer(IntegerKind::I128) => {
                            wrap_unary!((input: $integer_type) -> i128 {
                                input as i128
                            })
                        }
                        CastTarget::Integer(IntegerKind::Isize) => {
                            wrap_unary!((input: $integer_type) -> isize {
                                input as isize
                            })
                        }
                        CastTarget::Integer(IntegerKind::U8) => {
                            wrap_unary!((input: $integer_type) -> u8 {
                                input as u8
                            })
                        }
                        CastTarget::Integer(IntegerKind::U16) => {
                            wrap_unary!((input: $integer_type) -> u16 {
                                input as u16
                            })
                        }
                        CastTarget::Integer(IntegerKind::U32) => {
                            wrap_unary!((input: $integer_type) -> u32 {
                                input as u32
                            })
                        }
                        CastTarget::Integer(IntegerKind::U64) => {
                            wrap_unary!((input: $integer_type) -> u64 {
                                input as u64
                            })
                        }
                        CastTarget::Integer(IntegerKind::U128) => {
                            wrap_unary!((input: $integer_type) -> u128 {
                                input as u128
                            })
                        }
                        CastTarget::Integer(IntegerKind::Usize) => {
                            wrap_unary!((input: $integer_type) -> usize {
                                input as usize
                            })
                        }
                        CastTarget::Float(FloatKind::Untyped) => {
                            wrap_unary!((input: $integer_type) -> UntypedFloat {
                                UntypedFloat::from_fallback(input as FallbackFloat)
                            })
                        }
                        CastTarget::Float(FloatKind::F32) => {
                            wrap_unary!((input: $integer_type) -> f32 {
                                input as f32
                            })
                        }
                        CastTarget::Float(FloatKind::F64) => {
                            wrap_unary!((input: $integer_type) -> f64 {
                                input as f64
                            })
                        }
                        CastTarget::String => {
                            wrap_unary!((input: $integer_type) -> String {
                                input.to_string()
                            })
                        }
                        _ => return None,
                    }
                    _ => return None,
                })
            }
        }

        impl HasValueType for $integer_type {
            fn value_type(&self) -> &'static str {
                stringify!($integer_type)
            }
        }

        impl ToExpressionValue for $integer_type {
            fn to_value(self, span_range: SpanRange) -> ExpressionValue {
                ExpressionValue::Integer(ExpressionInteger {
                    value: ExpressionIntegerValue::$integer_enum_variant(self),
                    span_range,
                })
            }
        }

        impl HandleBinaryOperation for $integer_type {
            fn handle_paired_binary_operation(self, rhs: Self, operation: OutputSpanned<PairedBinaryOperation>) -> ExecutionResult<ExpressionValue> {
                let lhs = self;
                let overflow_error = || format!("The {} operation {:?} {} {:?} overflowed", stringify!($integer_type), lhs, operation.symbolic_description(), rhs);
                Ok(match operation.operation {
                    PairedBinaryOperation::Addition { .. } => return operation.output_if_some(lhs.checked_add(rhs), overflow_error),
                    PairedBinaryOperation::Subtraction { .. } => return operation.output_if_some(lhs.checked_sub(rhs), overflow_error),
                    PairedBinaryOperation::Multiplication { .. } => return operation.output_if_some(lhs.checked_mul(rhs), overflow_error),
                    PairedBinaryOperation::Division { .. } => return operation.output_if_some(lhs.checked_div(rhs), overflow_error),
                    PairedBinaryOperation::LogicalAnd { .. }
                    | PairedBinaryOperation::LogicalOr { .. } => return operation.unsupported(self),
                    PairedBinaryOperation::Remainder { .. } => return operation.output_if_some(lhs.checked_rem(rhs), overflow_error),
                    PairedBinaryOperation::BitXor { .. } => operation.output(lhs ^ rhs),
                    PairedBinaryOperation::BitAnd { .. } => operation.output(lhs & rhs),
                    PairedBinaryOperation::BitOr { .. } => operation.output(lhs | rhs),
                    PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
                    PairedBinaryOperation::LessThan { .. } => operation.output(lhs < rhs),
                    PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
                    PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
                    PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
                    PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs > rhs),
                })
            }

            fn handle_integer_binary_operation(
                self,
                rhs: ExpressionInteger,
                operation: OutputSpanned<IntegerBinaryOperation>,
            ) -> ExecutionResult<ExpressionValue> {
                let lhs = self;
                Ok(match operation.operation {
                    IntegerBinaryOperation::ShiftLeft { .. } => {
                        match rhs.value {
                            ExpressionIntegerValue::Untyped(rhs) => operation.output(lhs << rhs.parse_fallback()?),
                            ExpressionIntegerValue::U8(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::U16(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::U32(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::U64(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::U128(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::Usize(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::I8(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::I16(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::I32(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::I64(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::I128(rhs) => operation.output(lhs << rhs),
                            ExpressionIntegerValue::Isize(rhs) => operation.output(lhs << rhs),
                        }
                    },
                    IntegerBinaryOperation::ShiftRight { .. } => {
                        match rhs.value {
                            ExpressionIntegerValue::Untyped(rhs) => operation.output(lhs >> rhs.parse_fallback()?),
                            ExpressionIntegerValue::U8(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::U16(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::U32(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::U64(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::U128(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::Usize(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::I8(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::I16(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::I32(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::I64(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::I128(rhs) => operation.output(lhs >> rhs),
                            ExpressionIntegerValue::Isize(rhs) => operation.output(lhs >> rhs),
                        }
                    },
                })
            }
        }
    )*};
}

impl_int_operations!(
    U8TypeData: [CharCast[],] U8(u8),
    U16TypeData: [] U16(u16),
    U32TypeData: [] U32(u32),
    U64TypeData: [] U64(u64),
    U128TypeData: [] U128(u128),
    UsizeTypeData: [] Usize(usize),
    I8TypeData: [Signed[],] I8(i8),
    I16TypeData: [Signed[],] I16(i16),
    I32TypeData: [Signed[],] I32(i32),
    I64TypeData: [Signed[],] I64(i64),
    I128TypeData: [Signed[],] I128(i128),
    IsizeTypeData: [Signed[],] Isize(isize),
);
