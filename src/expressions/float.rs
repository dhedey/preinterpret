use super::*;
use crate::internal_prelude::*;

pub(crate) struct EvaluationFloat {
    pub(super) source_span: SpanRange,
    pub(super) value: EvaluationFloatValue,
}

impl EvaluationFloat {
    pub(super) fn new(value: EvaluationFloatValue, source_span: SpanRange) -> Self {
        Self { value, source_span }
    }

    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ExecutionResult<Self> {
        Ok(Self {
            source_span: lit.span().span_range(),
            value: EvaluationFloatValue::for_litfloat(lit)?,
        })
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
        match self.value {
            EvaluationFloatValue::Untyped(input) => input.handle_unary_operation(&operation),
            EvaluationFloatValue::F32(input) => input.handle_unary_operation(&operation),
            EvaluationFloatValue::F64(input) => input.handle_unary_operation(&operation),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: EvaluationInteger,
        operation: BinaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
        match self.value {
            EvaluationFloatValue::Untyped(input) => {
                input.handle_integer_binary_operation(right, &operation)
            }
            EvaluationFloatValue::F32(input) => {
                input.handle_integer_binary_operation(right, &operation)
            }
            EvaluationFloatValue::F64(input) => {
                input.handle_integer_binary_operation(right, &operation)
            }
        }
    }

    pub(super) fn to_literal(&self) -> Literal {
        self.value
            .to_unspanned_literal()
            .with_span(self.source_span.start())
    }
}

impl quote::ToTokens for EvaluationFloat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_literal().to_tokens(tokens)
    }
}

pub(super) enum EvaluationFloatValuePair {
    Untyped(UntypedFloat, UntypedFloat),
    F32(f32, f32),
    F64(f64, f64),
}

impl EvaluationFloatValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: &BinaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
        match self {
            Self::Untyped(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::F32(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::F64(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }
}

pub(super) enum EvaluationFloatValue {
    Untyped(UntypedFloat),
    F32(f32),
    F64(f64),
}

impl EvaluationFloatValue {
    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ExecutionResult<Self> {
        Ok(match lit.suffix() {
            "" => Self::Untyped(UntypedFloat::new_from_lit_float(lit)),
            "f32" => Self::F32(lit.base10_parse()?),
            "f64" => Self::F64(lit.base10_parse()?),
            suffix => {
                return lit.span().execution_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        })
    }

    pub(super) fn describe_type(&self) -> &'static str {
        match self {
            EvaluationFloatValue::Untyped(_) => "untyped float",
            EvaluationFloatValue::F32(_) => "f32",
            EvaluationFloatValue::F64(_) => "f64",
        }
    }

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            EvaluationFloatValue::Untyped(float) => float.to_unspanned_literal(),
            EvaluationFloatValue::F32(float) => Literal::f32_suffixed(*float),
            EvaluationFloatValue::F64(float) => Literal::f64_suffixed(*float),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum FloatKind {
    Untyped,
    F32,
    F64,
}

pub(super) struct UntypedFloat(
    /// The span of the literal is ignored, and will be set when converted to an output.
    LitFloat,
);
pub(super) type FallbackFloat = f64;

impl UntypedFloat {
    pub(super) fn new_from_lit_float(lit_float: &LitFloat) -> Self {
        // LitFloat doesn't support Clone, so we have to do this
        Self::new_from_literal(lit_float.token())
    }

    pub(super) fn new_from_literal(literal: Literal) -> Self {
        Self(syn::LitFloat::from(literal))
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: &UnaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
        let input = self.parse_fallback()?;
        match operation.operator {
            UnaryOperator::Neg => operation.output(Self::from_fallback(-input)),
            UnaryOperator::Not => operation.unsupported_for_value_type_err("untyped float"),
            UnaryOperator::NoOp => operation.output(self),
            UnaryOperator::Cast(target) => match target {
                CastTarget::Integer(IntegerKind::Untyped) => {
                    operation.output(UntypedInteger::from_fallback(input as FallbackInteger))
                }
                CastTarget::Integer(IntegerKind::I8) => operation.output(input as i8),
                CastTarget::Integer(IntegerKind::I16) => operation.output(input as i16),
                CastTarget::Integer(IntegerKind::I32) => operation.output(input as i32),
                CastTarget::Integer(IntegerKind::I64) => operation.output(input as i64),
                CastTarget::Integer(IntegerKind::I128) => operation.output(input as i128),
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
                CastTarget::Float(FloatKind::F64) => operation.output(input),
                CastTarget::Boolean | CastTarget::Char => {
                    operation.err("This cast is not supported")
                }
            },
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _rhs: EvaluationInteger,
        operation: &BinaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
        match operation.integer_operator() {
            IntegerBinaryOperator::ShiftLeft | IntegerBinaryOperator::ShiftRight => {
                operation.unsupported_for_value_type_err("untyped float")
            }
        }
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &BinaryOperation,
    ) -> ExecutionResult<EvaluationOutput> {
        let lhs = self.parse_fallback()?;
        let rhs = rhs.parse_fallback()?;
        match operation.paired_operator() {
            PairedBinaryOperator::Addition => operation.output(Self::from_fallback(lhs + rhs)),
            PairedBinaryOperator::Subtraction => operation.output(Self::from_fallback(lhs - rhs)),
            PairedBinaryOperator::Multiplication => {
                operation.output(Self::from_fallback(lhs * rhs))
            }
            PairedBinaryOperator::Division => operation.output(Self::from_fallback(lhs / rhs)),
            PairedBinaryOperator::LogicalAnd | PairedBinaryOperator::LogicalOr => {
                operation.unsupported_for_value_type_err(stringify!($float_type))
            }
            PairedBinaryOperator::Remainder => operation.output(Self::from_fallback(lhs % rhs)),
            PairedBinaryOperator::BitXor
            | PairedBinaryOperator::BitAnd
            | PairedBinaryOperator::BitOr => {
                operation.unsupported_for_value_type_err("untyped float")
            }
            PairedBinaryOperator::Equal => operation.output(lhs == rhs),
            PairedBinaryOperator::LessThan => operation.output(lhs < rhs),
            PairedBinaryOperator::LessThanOrEqual => operation.output(lhs <= rhs),
            PairedBinaryOperator::NotEqual => operation.output(lhs != rhs),
            PairedBinaryOperator::GreaterThanOrEqual => operation.output(lhs >= rhs),
            PairedBinaryOperator::GreaterThan => operation.output(lhs > rhs),
        }
    }

    pub(super) fn from_fallback(value: FallbackFloat) -> Self {
        Self::new_from_literal(Literal::f64_unsuffixed(value))
    }

    fn parse_fallback(&self) -> ExecutionResult<FallbackFloat> {
        self.0.base10_digits().parse().map_err(|err| {
            self.0.span().execution_error(format!(
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
            self.0.span().execution_error(format!(
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

impl ToEvaluationOutput for UntypedFloat {
    fn to_output(self, span_range: SpanRange) -> EvaluationOutput {
        EvaluationValue::Float(EvaluationFloat::new(
            EvaluationFloatValue::Untyped(self),
            span_range,
        ))
        .into()
    }
}

macro_rules! impl_float_operations {
    (
        $($float_enum_variant:ident($float_type:ident)),* $(,)?
    ) => {$(
        impl ToEvaluationOutput for $float_type {
            fn to_output(self, span_range: SpanRange) -> EvaluationOutput {
                EvaluationValue::Float(EvaluationFloat::new(EvaluationFloatValue::$float_enum_variant(self), span_range)).into()
            }
        }

        impl HandleUnaryOperation for $float_type {
            fn handle_unary_operation(self, operation: &UnaryOperation) -> ExecutionResult<EvaluationOutput> {
                match operation.operator {
                    UnaryOperator::Neg => operation.output(-self),
                    UnaryOperator::Not => operation.unsupported_for_value_type_err(stringify!($float_type)),
                    UnaryOperator::NoOp => operation.output(self),
                    UnaryOperator::Cast(target) => match target {
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
                        CastTarget::Boolean | CastTarget::Char => operation.err("This cast is not supported"),
                    }
                }
            }
        }

        impl HandleBinaryOperation for $float_type {
            fn handle_paired_binary_operation(self, rhs: Self, operation: &BinaryOperation) -> ExecutionResult<EvaluationOutput> {
                // Unlike integer arithmetic, float arithmetic does not overflow
                // and instead falls back to NaN or infinity. In future we could
                // allow trapping on these codes, but for now this is good enough
                let lhs = self;
                match operation.paired_operator() {
                    PairedBinaryOperator::Addition => operation.output(lhs + rhs),
                    PairedBinaryOperator::Subtraction => operation.output(lhs - rhs),
                    PairedBinaryOperator::Multiplication => operation.output(lhs * rhs),
                    PairedBinaryOperator::Division => operation.output(lhs / rhs),
                    PairedBinaryOperator::LogicalAnd
                    | PairedBinaryOperator::LogicalOr => {
                        operation.unsupported_for_value_type_err(stringify!($float_type))
                    }
                    PairedBinaryOperator::Remainder => operation.output(lhs % rhs),
                    PairedBinaryOperator::BitXor
                    | PairedBinaryOperator::BitAnd
                    | PairedBinaryOperator::BitOr => {
                        operation.unsupported_for_value_type_err(stringify!($float_type))
                    }
                    PairedBinaryOperator::Equal => operation.output(lhs == rhs),
                    PairedBinaryOperator::LessThan => operation.output(lhs < rhs),
                    PairedBinaryOperator::LessThanOrEqual => operation.output(lhs <= rhs),
                    PairedBinaryOperator::NotEqual => operation.output(lhs != rhs),
                    PairedBinaryOperator::GreaterThanOrEqual => operation.output(lhs >= rhs),
                    PairedBinaryOperator::GreaterThan => operation.output(lhs > rhs),
                }
            }

            fn handle_integer_binary_operation(
                self,
                _rhs: EvaluationInteger,
                operation: &BinaryOperation,
            ) -> ExecutionResult<EvaluationOutput> {
                match operation.integer_operator() {
                    IntegerBinaryOperator::ShiftLeft | IntegerBinaryOperator::ShiftRight => {
                        operation.unsupported_for_value_type_err(stringify!($float_type))
                    },
                }
            }
        }
    )*};
}
impl_float_operations!(F32(f32), F64(f64));
