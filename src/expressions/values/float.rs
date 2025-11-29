use super::*;
use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct FloatExpression {
    pub(super) value: FloatExpressionValue,
}

impl ToExpressionValue for FloatExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Float(self)
    }
}

impl FloatExpression {
    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ParseResult<Owned<Self>> {
        Ok(Self {
            value: FloatExpressionValue::for_litfloat(lit)?,
        }
        .into_owned(lit.span()))
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.value.to_unspanned_literal().with_span(span)
    }
}

impl HasValueType for FloatExpression {
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
        pub(crate) mod binary_operations {}
        interface_items {
        }
    }
}

#[derive(Clone)]
pub(crate) enum FloatExpressionValue {
    Untyped(UntypedFloat),
    F32(f32),
    F64(f64),
}

impl FloatExpressionValue {
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
            FloatExpressionValue::Untyped(float) => float.to_unspanned_literal(),
            FloatExpressionValue::F32(float) => Literal::f32_suffixed(*float),
            FloatExpressionValue::F64(float) => Literal::f64_suffixed(*float),
        }
    }
}

impl HasValueType for FloatExpressionValue {
    fn value_type(&self) -> &'static str {
        match self {
            FloatExpressionValue::Untyped(_) => "untyped float",
            FloatExpressionValue::F32(_) => "f32",
            FloatExpressionValue::F64(_) => "f64",
        }
    }
}

impl ToExpressionValue for FloatExpressionValue {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Float(FloatExpression { value: self })
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
    ) -> ExecutionResult<ResolvedValue> {
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
                UntypedFloat::from_fallback(output).to_resolved_value(context.output_span_range)
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
            ) -> ExecutionResult<ResolvedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a + b))
            }

            [context] fn sub(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a - b))
            }

            [context] fn mul(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a * b))
            }

            [context] fn div(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedFloat::paired_operation(lhs, rhs, context, |a, b| Some(a / b))
            }

            [context] fn rem(
                lhs: Owned<UntypedFloat>,
                rhs: Owned<FloatExpression>,
            ) -> ExecutionResult<ResolvedValue> {
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
                pub(crate) mod binary_operations {
                    [context] fn add(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a + b))
                    }

                    [context] fn sub(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a - b))
                    }

                    [context] fn mul(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a * b))
                    }

                    [context] fn div(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a / b))
                    }

                    [context] fn rem(
                        lhs: $float_type,
                        rhs: $float_type,
                    ) -> ExecutionResult<$float_type> {
                        $float_type::paired_operation(lhs, rhs, context, |a, b| Some(a % b))
                    }

                    fn eq(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs == rhs
                    }

                    fn ne(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs != rhs
                    }

                    fn lt(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs < rhs
                    }

                    fn le(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs <= rhs
                    }

                    fn ge(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs >= rhs
                    }

                    fn gt(lhs: $float_type, rhs: $float_type) -> bool {
                        lhs > rhs
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

        impl HasValueType for $float_type {
            fn value_type(&self) -> &'static str {
                stringify!($float_type)
            }
        }

        impl ToExpressionValue for $float_type {
            fn into_value(self) -> ExpressionValue {
                ExpressionValue::Float(FloatExpression {
                    value: FloatExpressionValue::$float_enum_variant(self),
                })
            }
        }


        impl HandleBinaryOperation for $float_type {
            fn type_name() -> &'static str {
                stringify!($integer_type)
            }
        }
    )*};
}
impl_float_operations!(F32TypeData mod f32_interface: F32(f32), F64TypeData mod f64_interface: F64(f64));

impl_resolvable_argument_for! {
    FloatTypeData,
    (value, context) -> FloatExpression {
        match value {
            ExpressionValue::Float(value) => Ok(value),
            other => context.err("float", other),
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

macro_rules! impl_resolvable_float_subtype {
    ($value_type:ty, $type:ty, $variant:ident, $expected_msg:expr) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_value(
                value: ExpressionValue,
                context: ResolutionContext,
            ) -> ExecutionResult<Self> {
                match value {
                    ExpressionValue::Float(FloatExpression {
                        value: FloatExpressionValue::Untyped(x),
                        ..
                    }) => x.parse_as(),
                    ExpressionValue::Float(FloatExpression {
                        value: FloatExpressionValue::$variant(x),
                        ..
                    }) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref<'a>(
                value: &'a ExpressionValue,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a Self> {
                match value {
                    ExpressionValue::Float(FloatExpression {
                        value: FloatExpressionValue::$variant(x),
                        ..
                    }) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut<'a>(
                value: &'a mut ExpressionValue,
                context: ResolutionContext,
            ) -> ExecutionResult<&'a mut Self> {
                match value {
                    ExpressionValue::Float(FloatExpression {
                        value: FloatExpressionValue::$variant(x),
                        ..
                    }) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }
    };
}

impl_resolvable_float_subtype!(F32TypeData, f32, F32, "f32");
impl_resolvable_float_subtype!(F64TypeData, f64, F64, "f64");
