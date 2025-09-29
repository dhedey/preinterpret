use super::*;
use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct ExpressionFloat {
    pub(super) value: ExpressionFloatValue,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(super) span_range: SpanRange,
}

impl ExpressionFloat {
    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ParseResult<Self> {
        let span_range = lit.span().span_range();
        Ok(Self {
            value: ExpressionFloatValue::for_litfloat(lit)?,
            span_range,
        })
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self.value {
            ExpressionFloatValue::Untyped(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionFloatValue::F32(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            ExpressionFloatValue::F64(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
        }
    }

    pub(super) fn to_literal(&self) -> Literal {
        self.value
            .to_unspanned_literal()
            .with_span(self.span_range.join_into_span_else_start())
    }
}

impl HasValueType for ExpressionFloat {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FloatTypeData;

impl MethodResolutionTarget for FloatTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);
}

pub(super) enum ExpressionFloatValuePair {
    Untyped(UntypedFloat, UntypedFloat),
    F32(f32, f32),
    F64(f64, f64),
}

impl ExpressionFloatValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            Self::Untyped(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::F32(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::F64(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }
}

#[derive(Clone)]
pub(super) enum ExpressionFloatValue {
    Untyped(UntypedFloat),
    F32(f32),
    F64(f64),
}

impl ExpressionFloatValue {
    pub(super) fn kind(&self) -> FloatKind {
        match self {
            Self::Untyped(_) => FloatKind::Untyped,
            Self::F32(_) => FloatKind::F32,
            Self::F64(_) => FloatKind::F64,
        }
    }

    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ParseResult<Self> {
        Ok(match lit.suffix() {
            "" => Self::Untyped(UntypedFloat::new_from_lit_float(lit.clone())),
            "f32" => Self::F32(lit.base10_parse()?),
            "f64" => Self::F64(lit.base10_parse()?),
            suffix => {
                return lit.span().parse_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        })
    }

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            ExpressionFloatValue::Untyped(float) => float.to_unspanned_literal(),
            ExpressionFloatValue::F32(float) => Literal::f32_suffixed(*float),
            ExpressionFloatValue::F64(float) => Literal::f64_suffixed(*float),
        }
    }
}

impl HasValueType for ExpressionFloatValue {
    fn value_type(&self) -> &'static str {
        match self {
            ExpressionFloatValue::Untyped(_) => "untyped float",
            ExpressionFloatValue::F32(_) => "f32",
            ExpressionFloatValue::F64(_) => "f64",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FloatKind {
    Untyped,
    F32,
    F64,
}

impl FloatKind {
    pub(super) fn method_resolver(&self) -> &'static dyn MethodResolver {
        static UNTYPED: UntypedFloatTypeData = UntypedFloatTypeData;
        static F32: F32TypeData = F32TypeData;
        static F64: F64TypeData = F64TypeData;
        match self {
            FloatKind::Untyped => &UNTYPED,
            FloatKind::F32 => &F32,
            FloatKind::F64 => &F64,
        }
    }
}

#[derive(Clone)]
pub(super) struct UntypedFloat(
    /// The span of the literal is ignored, and will be set when converted to an output.
    LitFloat,
    SpanRange,
);
pub(super) type FallbackFloat = f64;

impl UntypedFloat {
    pub(super) fn new_from_lit_float(lit_float: LitFloat) -> Self {
        let span_range = lit_float.span().span_range();
        Self(lit_float, span_range)
    }

    pub(super) fn new_from_literal(literal: Literal) -> Self {
        Self::new_from_lit_float(literal.into())
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _rhs: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match operation.operation {
            IntegerBinaryOperation::ShiftLeft { .. }
            | IntegerBinaryOperation::ShiftRight { .. } => operation.unsupported(self),
        }
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.parse_fallback()?;
        let rhs = rhs.parse_fallback()?;
        Ok(match operation.operation {
            PairedBinaryOperation::Addition { .. } => {
                operation.output(Self::from_fallback(lhs + rhs))
            }
            PairedBinaryOperation::Subtraction { .. } => {
                operation.output(Self::from_fallback(lhs - rhs))
            }
            PairedBinaryOperation::Multiplication { .. } => {
                operation.output(Self::from_fallback(lhs * rhs))
            }
            PairedBinaryOperation::Division { .. } => {
                operation.output(Self::from_fallback(lhs / rhs))
            }
            PairedBinaryOperation::LogicalAnd { .. } | PairedBinaryOperation::LogicalOr { .. } => {
                return operation.unsupported(self)
            }
            PairedBinaryOperation::Remainder { .. } => {
                operation.output(Self::from_fallback(lhs % rhs))
            }
            PairedBinaryOperation::BitXor { .. }
            | PairedBinaryOperation::BitAnd { .. }
            | PairedBinaryOperation::BitOr { .. } => return operation.unsupported(self),
            PairedBinaryOperation::Equal { .. } => operation.output(lhs == rhs),
            PairedBinaryOperation::LessThan { .. } => operation.output(lhs < rhs),
            PairedBinaryOperation::LessThanOrEqual { .. } => operation.output(lhs <= rhs),
            PairedBinaryOperation::NotEqual { .. } => operation.output(lhs != rhs),
            PairedBinaryOperation::GreaterThanOrEqual { .. } => operation.output(lhs >= rhs),
            PairedBinaryOperation::GreaterThan { .. } => operation.output(lhs > rhs),
        })
    }

    pub(super) fn from_fallback(value: FallbackFloat) -> Self {
        Self::new_from_literal(Literal::f64_unsuffixed(value))
    }

    pub(super) fn parse_fallback(&self) -> ExecutionResult<FallbackFloat> {
        self.0.base10_digits().parse().map_err(|err| {
            self.1.execution_error(format!(
                "Could not parse as the default inferred type {}: {}",
                core::any::type_name::<FallbackFloat>(),
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

    fn to_unspanned_literal(&self) -> Literal {
        self.0.token()
    }
}

impl HasValueType for UntypedFloat {
    fn value_type(&self) -> &'static str {
        "untyped float"
    }
}

impl ToExpressionValue for UntypedFloat {
    fn to_value(mut self, span_range: SpanRange) -> ExpressionValue {
        self.1 = span_range;
        ExpressionValue::Float(ExpressionFloat {
            value: ExpressionFloatValue::Untyped(self),
            span_range,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UntypedFloatTypeData;

impl MethodResolutionTarget for UntypedFloatTypeData {
    type Parent = FloatTypeData;
    const PARENT: Option<Self::Parent> = Some(FloatTypeData);

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Neg { .. } => {
                wrap_unary!((input: UntypedFloatFallback) -> UntypedFloat {
                    UntypedFloat::from_fallback(-input.0)
                })
            }
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    wrap_unary!((input: UntypedFloatFallback) -> UntypedInteger {
                        UntypedInteger::from_fallback(input.0 as FallbackInteger)
                    })
                }
                CastTarget::Integer(IntegerKind::I8) => {
                    wrap_unary!((input: UntypedFloatFallback) -> i8 {
                        input.0 as i8
                    })
                }
                CastTarget::Integer(IntegerKind::I16) => {
                    wrap_unary!((input: UntypedFloatFallback) -> i16 {
                        input.0 as i16
                    })
                }
                CastTarget::Integer(IntegerKind::I32) => {
                    wrap_unary!((input: UntypedFloatFallback) -> i32 {
                        input.0 as i32
                    })
                }
                CastTarget::Integer(IntegerKind::I64) => {
                    wrap_unary!((input: UntypedFloatFallback) -> i64 {
                        input.0 as i64
                    })
                }
                CastTarget::Integer(IntegerKind::I128) => {
                    wrap_unary!((input: UntypedFloatFallback) -> i128 {
                        input.0 as i128
                    })
                }
                CastTarget::Integer(IntegerKind::Isize) => {
                    wrap_unary!((input: UntypedFloatFallback) -> isize {
                        input.0 as isize
                    })
                }
                CastTarget::Integer(IntegerKind::U8) => {
                    wrap_unary!((input: UntypedFloatFallback) -> u8 {
                        input.0 as u8
                    })
                }
                CastTarget::Integer(IntegerKind::U16) => {
                    wrap_unary!((input: UntypedFloatFallback) -> u16 {
                        input.0 as u16
                    })
                }
                CastTarget::Integer(IntegerKind::U32) => {
                    wrap_unary!((input: UntypedFloatFallback) -> u32 {
                        input.0 as u32
                    })
                }
                CastTarget::Integer(IntegerKind::U64) => {
                    wrap_unary!((input: UntypedFloatFallback) -> u64 {
                        input.0 as u64
                    })
                }
                CastTarget::Integer(IntegerKind::U128) => {
                    wrap_unary!((input: UntypedFloatFallback) -> u128 {
                        input.0 as u128
                    })
                }
                CastTarget::Integer(IntegerKind::Usize) => {
                    wrap_unary!((input: UntypedFloatFallback) -> usize {
                        input.0 as usize
                    })
                }
                CastTarget::Float(FloatKind::Untyped) => {
                    wrap_unary!((input: UntypedFloatFallback) -> UntypedFloat {
                        UntypedFloat::from_fallback(input.0)
                    })
                }
                CastTarget::Float(FloatKind::F32) => {
                    wrap_unary!((input: UntypedFloatFallback) -> f32 {
                        input.0 as f32
                    })
                }
                CastTarget::Float(FloatKind::F64) => {
                    wrap_unary!((input: UntypedFloatFallback) -> f64 {
                        input.0
                    })
                }
                CastTarget::String => {
                    wrap_unary!((input: UntypedFloatFallback) -> String {
                        input.0.to_string()
                    })
                }
                _ => return None,
            },
            _ => return None,
        })
    }
}

macro_rules! impl_float_operations {
    (
        $($type_data:ident: $float_enum_variant:ident($float_type:ident)),* $(,)?
    ) => {$(
        #[derive(Clone, Copy)]
        pub(crate) struct $type_data;

        impl MethodResolutionTarget for $type_data {
            type Parent = FloatTypeData;
            const PARENT: Option<Self::Parent> = Some(FloatTypeData);

            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Neg { .. } => wrap_unary!((this: Owned<$float_type>) -> ExecutionResult<$float_type> {
                        Ok(-this.into_inner())
                    }),
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::Integer(IntegerKind::Untyped) => {
                            wrap_unary!((input: $float_type) -> UntypedInteger {
                                UntypedInteger::from_fallback(input as FallbackInteger)
                            })
                        }
                        CastTarget::Integer(IntegerKind::I8) => {
                            wrap_unary!((input: $float_type) -> i8 {
                                input as i8
                            })
                        }
                        CastTarget::Integer(IntegerKind::I16) => {
                            wrap_unary!((input: $float_type) -> i16 {
                                input as i16
                            })
                        }
                        CastTarget::Integer(IntegerKind::I32) => {
                            wrap_unary!((input: $float_type) -> i32 {
                                input as i32
                            })
                        }
                        CastTarget::Integer(IntegerKind::I64) => {
                            wrap_unary!((input: $float_type) -> i64 {
                                input as i64
                            })
                        }
                        CastTarget::Integer(IntegerKind::I128) => {
                            wrap_unary!((input: $float_type) -> i128 {
                                input as i128
                            })
                        }
                        CastTarget::Integer(IntegerKind::Isize) => {
                            wrap_unary!((input: $float_type) -> isize {
                                input as isize
                            })
                        }
                        CastTarget::Integer(IntegerKind::U8) => {
                            wrap_unary!((input: $float_type) -> u8 {
                                input as u8
                            })
                        }
                        CastTarget::Integer(IntegerKind::U16) => {
                            wrap_unary!((input: $float_type) -> u16 {
                                input as u16
                            })
                        }
                        CastTarget::Integer(IntegerKind::U32) => {
                            wrap_unary!((input: $float_type) -> u32 {
                                input as u32
                            })
                        }
                        CastTarget::Integer(IntegerKind::U64) => {
                            wrap_unary!((input: $float_type) -> u64 {
                                input as u64
                            })
                        }
                        CastTarget::Integer(IntegerKind::U128) => {
                            wrap_unary!((input: $float_type) -> u128 {
                                input as u128
                            })
                        }
                        CastTarget::Integer(IntegerKind::Usize) => {
                            wrap_unary!((input: $float_type) -> usize {
                                input as usize
                            })
                        }
                        CastTarget::Float(FloatKind::Untyped) => {
                            wrap_unary!((input: $float_type) -> UntypedFloat {
                                UntypedFloat::from_fallback(input as FallbackFloat)
                            })
                        }
                        CastTarget::Float(FloatKind::F32) => {
                            wrap_unary!((input: $float_type) -> f32 {
                                input as f32
                            })
                        }
                        CastTarget::Float(FloatKind::F64) => {
                            wrap_unary!((input: $float_type) -> f64 {
                                input as f64
                            })
                        }
                        CastTarget::String => {
                            wrap_unary!((input: $float_type) -> String {
                                input.to_string()
                            })
                        }
                        _ => return None,
                    }
                    _ => return None,
                })
            }
        }

        impl HasValueType for $float_type {
            fn value_type(&self) -> &'static str {
                stringify!($float_type)
            }
        }

        impl ToExpressionValue for $float_type {
            fn to_value(self, span_range: SpanRange) -> ExpressionValue {
                ExpressionValue::Float(ExpressionFloat {
                    value: ExpressionFloatValue::$float_enum_variant(self),
                    span_range,
                })
            }
        }


        impl HandleBinaryOperation for $float_type {
            fn handle_paired_binary_operation(self, rhs: Self, operation: OutputSpanned<PairedBinaryOperation>) -> ExecutionResult<ExpressionValue> {
                // Unlike integer arithmetic, float arithmetic does not overflow
                // and instead falls back to NaN or infinity. In future we could
                // allow trapping on these codes, but for now this is good enough
                let lhs = self;
                Ok(match operation.operation {
                    PairedBinaryOperation::Addition { .. } => operation.output(lhs + rhs),
                    PairedBinaryOperation::Subtraction { .. } => operation.output(lhs - rhs),
                    PairedBinaryOperation::Multiplication { .. } => operation.output(lhs * rhs),
                    PairedBinaryOperation::Division { .. } => operation.output(lhs / rhs),
                    PairedBinaryOperation::LogicalAnd { .. }
                    | PairedBinaryOperation::LogicalOr { .. } => {
                        return operation.unsupported(self)
                    }
                    PairedBinaryOperation::Remainder { .. } => operation.output(lhs % rhs),
                    PairedBinaryOperation::BitXor { .. }
                    | PairedBinaryOperation::BitAnd { .. }
                    | PairedBinaryOperation::BitOr { .. } => {
                        return operation.unsupported(self)
                    }
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
                _rhs: ExpressionInteger,
                operation: OutputSpanned<IntegerBinaryOperation>,
            ) -> ExecutionResult<ExpressionValue> {
                match operation.operation {
                    IntegerBinaryOperation::ShiftLeft { .. } | IntegerBinaryOperation::ShiftRight { .. } => {
                        operation.unsupported(self)
                    },
                }
            }
        }
    )*};
}
impl_float_operations!(F32TypeData: F32(f32), F64TypeData: F64(f64));
