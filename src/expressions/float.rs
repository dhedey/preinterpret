use super::*;
use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct ExpressionFloat {
    pub(super) value: ExpressionFloatValue,
}

impl ToExpressionValue for ExpressionFloat {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Float(self)
    }
}

impl ExpressionFloat {
    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ParseResult<Owned<Self>> {
        Ok(Self {
            value: ExpressionFloatValue::for_litfloat(lit)?,
        }
        .into_owned(lit.span()))
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: ExpressionInteger,
        operation: WrappedOp<IntegerBinaryOperation>,
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

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.value.to_unspanned_literal().with_span(span)
    }
}

impl HasValueType for ExpressionFloat {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

define_interface! {
    struct FloatTypeData,
    parent: ValueTypeData,
    pub(crate) mod float_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
        }
        interface_items {
        }
    }
}

pub(super) enum ExpressionFloatValuePair {
    Untyped(UntypedFloat, UntypedFloat),
    F32(f32, f32),
    F64(f64, f64),
}

impl ExpressionFloatValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: WrappedOp<PairedBinaryOperation>,
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

    pub(super) fn handle_integer_binary_operation(
        self,
        _rhs: ExpressionInteger,
        operation: WrappedOp<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match operation.operation {
            IntegerBinaryOperation::ShiftLeft { .. }
            | IntegerBinaryOperation::ShiftRight { .. } => operation.unsupported(self),
        }
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: WrappedOp<PairedBinaryOperation>,
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
        // TODO[untyped] - Have a way to store this more efficiently without going through a literal
        Self::new_from_known_float_literal(
            Literal::f64_unsuffixed(value).with_span(Span::call_site()),
        )
    }

    pub(super) fn parse_fallback(&self) -> ExecutionResult<FallbackFloat> {
        self.0.base10_digits().parse().map_err(|err| {
            self.0.value_error(format!(
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
            self.0.value_error(format!(
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
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Float(ExpressionFloat {
            value: ExpressionFloatValue::Untyped(self),
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
        }
    }
}

macro_rules! impl_float_operations {
    (
        $($type_data:ident mod $mod_name:ident: $float_enum_variant:ident($float_type:ident)),* $(,)?
    ) => {$(
        define_interface! {
            struct $type_data,
            parent: FloatTypeData,
            pub(crate) mod $mod_name {
                pub(crate) mod methods {
                }
                pub(crate) mod unary_operations {
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
                            }
                            _ => return None,
                        })
                    }
                }
            }
        }

        impl HasValueType for $float_type {
            fn value_type(&self) -> &'static str {
                stringify!($float_type)
            }
        }

        impl ToExpressionValue for $float_type {
            fn into_value(self) -> ExpressionValue {
                ExpressionValue::Float(ExpressionFloat {
                    value: ExpressionFloatValue::$float_enum_variant(self),
                })
            }
        }


        impl HandleBinaryOperation for $float_type {
            fn handle_paired_binary_operation(self, rhs: Self, operation: WrappedOp<PairedBinaryOperation>) -> ExecutionResult<ExpressionValue> {
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
                operation: WrappedOp<IntegerBinaryOperation>,
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
impl_float_operations!(F32TypeData mod f32_interface: F32(f32), F64TypeData mod f64_interface: F64(f64));
