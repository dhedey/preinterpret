use super::*;

#[derive(Clone)]
pub(crate) struct IntegerExpression {
    pub(crate) value: IntegerExpressionValue,
}

impl ToExpressionValue for IntegerExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Integer(self)
    }
}

impl IntegerExpression {
    pub(super) fn for_litint(lit: &syn::LitInt) -> ParseResult<Owned<Self>> {
        Ok(Self {
            value: IntegerExpressionValue::for_litint(lit)?,
        }
        .into_owned(lit.span_range()))
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: IntegerExpression,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<ExpressionValue> {
        match self.value {
            IntegerExpressionValue::Untyped(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::U8(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::U16(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::U32(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::U64(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::U128(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::Usize(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::I8(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::I16(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::I32(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::I64(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::I128(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
            IntegerExpressionValue::Isize(input) => {
                input.handle_integer_binary_operation(right, operation)
            }
        }
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.value.to_unspanned_literal().with_span(span)
    }
}

impl HasValueType for IntegerExpression {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

define_interface! {
    struct IntegerTypeData,
    parent: ValueTypeData,
    pub(crate) mod integer_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {}
        interface_items {
        }
    }
}

pub(crate) enum IntegerExpressionValuePair {
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

impl IntegerExpressionValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: &PairedBinaryOperation,
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
pub(crate) enum IntegerExpressionValue {
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

impl IntegerExpressionValue {
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

impl IntegerExpressionValue {
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
            IntegerExpressionValue::Untyped(int) => int.to_unspanned_literal(),
            IntegerExpressionValue::U8(int) => Literal::u8_suffixed(*int),
            IntegerExpressionValue::U16(int) => Literal::u16_suffixed(*int),
            IntegerExpressionValue::U32(int) => Literal::u32_suffixed(*int),
            IntegerExpressionValue::U64(int) => Literal::u64_suffixed(*int),
            IntegerExpressionValue::U128(int) => Literal::u128_suffixed(*int),
            IntegerExpressionValue::Usize(int) => Literal::usize_suffixed(*int),
            IntegerExpressionValue::I8(int) => Literal::i8_suffixed(*int),
            IntegerExpressionValue::I16(int) => Literal::i16_suffixed(*int),
            IntegerExpressionValue::I32(int) => Literal::i32_suffixed(*int),
            IntegerExpressionValue::I64(int) => Literal::i64_suffixed(*int),
            IntegerExpressionValue::I128(int) => Literal::i128_suffixed(*int),
            IntegerExpressionValue::Isize(int) => Literal::isize_suffixed(*int),
        }
    }
}

impl HasValueType for IntegerExpressionValue {
    fn value_type(&self) -> &'static str {
        match self {
            IntegerExpressionValue::Untyped(value) => value.value_type(),
            IntegerExpressionValue::U8(value) => value.value_type(),
            IntegerExpressionValue::U16(value) => value.value_type(),
            IntegerExpressionValue::U32(value) => value.value_type(),
            IntegerExpressionValue::U64(value) => value.value_type(),
            IntegerExpressionValue::U128(value) => value.value_type(),
            IntegerExpressionValue::Usize(value) => value.value_type(),
            IntegerExpressionValue::I8(value) => value.value_type(),
            IntegerExpressionValue::I16(value) => value.value_type(),
            IntegerExpressionValue::I32(value) => value.value_type(),
            IntegerExpressionValue::I64(value) => value.value_type(),
            IntegerExpressionValue::I128(value) => value.value_type(),
            IntegerExpressionValue::Isize(value) => value.value_type(),
        }
    }
}

impl ToExpressionValue for IntegerExpressionValue {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Integer(IntegerExpression { value: self })
    }
}

impl HasValueType for UntypedInteger {
    fn value_type(&self) -> &'static str {
        "untyped integer"
    }
}

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

    fn into_kind(self, kind: IntegerKind) -> ExecutionResult<IntegerExpressionValue> {
        Ok(match kind {
            IntegerKind::Untyped => IntegerExpressionValue::Untyped(self),
            IntegerKind::I8 => IntegerExpressionValue::I8(self.parse_as()?),
            IntegerKind::I16 => IntegerExpressionValue::I16(self.parse_as()?),
            IntegerKind::I32 => IntegerExpressionValue::I32(self.parse_as()?),
            IntegerKind::I64 => IntegerExpressionValue::I64(self.parse_as()?),
            IntegerKind::I128 => IntegerExpressionValue::I128(self.parse_as()?),
            IntegerKind::Isize => IntegerExpressionValue::Isize(self.parse_as()?),
            IntegerKind::U8 => IntegerExpressionValue::U8(self.parse_as()?),
            IntegerKind::U16 => IntegerExpressionValue::U16(self.parse_as()?),
            IntegerKind::U32 => IntegerExpressionValue::U32(self.parse_as()?),
            IntegerKind::U64 => IntegerExpressionValue::U64(self.parse_as()?),
            IntegerKind::U128 => IntegerExpressionValue::U128(self.parse_as()?),
            IntegerKind::Usize => IntegerExpressionValue::Usize(self.parse_as()?),
        })
    }

    fn paired_operation(
        lhs: Owned<UntypedInteger>,
        rhs: Owned<IntegerExpression>,
        context: BinaryOperationCallContext,
        perform_fn: fn(FallbackInteger, FallbackInteger) -> Option<FallbackInteger>,
    ) -> ExecutionResult<ResolvedValue> {
        let (lhs, lhs_span_range) = lhs.deconstruct();
        let (rhs, rhs_span_range) = rhs.deconstruct();
        match rhs.value {
            IntegerExpressionValue::Untyped(rhs) => {
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
                UntypedInteger::from_fallback(output).to_resolved_value(context.output_span_range)
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

    pub(super) fn handle_integer_binary_operation(
        self,
        rhs: IntegerExpression,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.parse_fallback()?;
        Ok(match operation {
            IntegerBinaryOperation::ShiftLeft { .. } => match rhs.value {
                IntegerExpressionValue::Untyped(rhs) => {
                    operation.output(lhs << rhs.parse_fallback()?)
                }
                IntegerExpressionValue::U8(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::U16(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::U32(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::U64(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::U128(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::Usize(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::I8(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::I16(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::I32(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::I64(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::I128(rhs) => operation.output(lhs << rhs),
                IntegerExpressionValue::Isize(rhs) => operation.output(lhs << rhs),
            },
            IntegerBinaryOperation::ShiftRight { .. } => match rhs.value {
                IntegerExpressionValue::Untyped(rhs) => {
                    operation.output(lhs >> rhs.parse_fallback()?)
                }
                IntegerExpressionValue::U8(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::U16(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::U32(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::U64(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::U128(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::Usize(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::I8(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::I16(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::I32(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::I64(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::I128(rhs) => operation.output(lhs >> rhs),
                IntegerExpressionValue::Isize(rhs) => operation.output(lhs >> rhs),
            },
        })
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &PairedBinaryOperation,
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
        Ok(match operation {
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

impl ToExpressionValue for UntypedInteger {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Integer(IntegerExpression {
            value: IntegerExpressionValue::Untyped(self),
        })
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
            [context] fn add(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedInteger::paired_operation(lhs, rhs, context, FallbackInteger::checked_add)
            }

            [context] fn sub(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedInteger::paired_operation(lhs, rhs, context, FallbackInteger::checked_sub)
            }

            [context] fn mul(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedInteger::paired_operation(lhs, rhs, context, FallbackInteger::checked_mul)
            }

            [context] fn div(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedInteger::paired_operation(lhs, rhs, context, FallbackInteger::checked_div)
            }

            [context] fn rem(
                lhs: Owned<UntypedInteger>,
                rhs: Owned<IntegerExpression>,
            ) -> ExecutionResult<ResolvedValue> {
                UntypedInteger::paired_operation(lhs, rhs, context, FallbackInteger::checked_rem)
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
                    _ => return None,
                })
            }
        }
    }
}

// We have to use a macro because we don't have checked xx traits :(
macro_rules! impl_int_operations {
    (
        $($integer_type_data:ident mod $mod_name:ident: [$(CharCast[$char_cast:ident],)?$(Signed[$signed:ident],)?] $integer_enum_variant:ident($integer_type:ident)),* $(,)?
    ) => {$(
        define_interface! {
            struct $integer_type_data,
            parent: IntegerTypeData,
            pub(crate) mod $mod_name {
                pub(crate) mod methods {
                }
                pub(crate) mod unary_operations {
                    $(
                        fn neg(this: Owned<$integer_type>) -> ExecutionResult<$integer_type> {
                            ignore_all!($signed); // Include only for signed types
                            let (value, span_range) = this.deconstruct();
                            match value.checked_neg() {
                                Some(negated) => Ok(negated),
                                None => span_range.value_err("Negating this value would overflow"),
                            }
                        }
                    )?

                    $(
                        fn cast_to_char(input: $integer_type) -> char {
                            ignore_all!($char_cast); // Include only for types with CharCast
                            input as char
                        }
                    )?

                    fn cast_to_untyped_integer(input: $integer_type) -> UntypedInteger {
                        UntypedInteger::from_fallback(input as FallbackInteger)
                    }

                    fn cast_to_i8(input: $integer_type) -> i8 {
                        input as i8
                    }

                    fn cast_to_i16(input: $integer_type) -> i16 {
                        input as i16
                    }

                    fn cast_to_i32(input: $integer_type) -> i32 {
                        input as i32
                    }

                    fn cast_to_i64(input: $integer_type) -> i64 {
                        input as i64
                    }

                    fn cast_to_i128(input: $integer_type) -> i128 {
                        input as i128
                    }

                    fn cast_to_isize(input: $integer_type) -> isize {
                        input as isize
                    }

                    fn cast_to_u8(input: $integer_type) -> u8 {
                        input as u8
                    }

                    fn cast_to_u16(input: $integer_type) -> u16 {
                        input as u16
                    }

                    fn cast_to_u32(input: $integer_type) -> u32 {
                        input as u32
                    }

                    fn cast_to_u64(input: $integer_type) -> u64 {
                        input as u64
                    }

                    fn cast_to_u128(input: $integer_type) -> u128 {
                        input as u128
                    }

                    fn cast_to_usize(input: $integer_type) -> usize {
                        input as usize
                    }

                    fn cast_to_untyped_float(input: $integer_type) -> UntypedFloat {
                        UntypedFloat::from_fallback(input as FallbackFloat)
                    }

                    fn cast_to_f32(input: $integer_type) -> f32 {
                        input as f32
                    }

                    fn cast_to_f64(input: $integer_type) -> f64 {
                        input as f64
                    }

                    fn cast_to_string(input: $integer_type) -> String {
                        input.to_string()
                    }
                }
                pub(crate) mod binary_operations {
                    [context] fn add(
                        lhs: $integer_type,
                        rhs: $integer_type,
                    ) -> ExecutionResult<$integer_type> {
                        $integer_type::paired_operation(lhs, rhs, context, <$integer_type>::checked_add)
                    }

                    [context] fn sub(
                        lhs: $integer_type,
                        rhs: $integer_type,
                    ) -> ExecutionResult<$integer_type> {
                        $integer_type::paired_operation(lhs, rhs, context, <$integer_type>::checked_sub)
                    }

                    [context] fn mul(
                        lhs: $integer_type,
                        rhs: $integer_type,
                    ) -> ExecutionResult<$integer_type> {
                        $integer_type::paired_operation(lhs, rhs, context, <$integer_type>::checked_mul)
                    }

                    [context] fn div(
                        lhs: $integer_type,
                        rhs: $integer_type,
                    ) -> ExecutionResult<$integer_type> {
                        $integer_type::paired_operation(lhs, rhs, context, <$integer_type>::checked_div)
                    }

                    [context] fn rem(
                        lhs: $integer_type,
                        rhs: $integer_type,
                    ) -> ExecutionResult<$integer_type> {
                        $integer_type::paired_operation(lhs, rhs, context, <$integer_type>::checked_rem)
                    }
                }
                interface_items {
                    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                        Some(match operation {
                            $(
                                UnaryOperation::Neg { .. } => {
                                    ignore_all!($signed); // Only include for signed types
                                    unary_definitions::neg()
                                }
                            )?
                            UnaryOperation::Cast { target, .. } => match target {
                                $(
                                    CastTarget::Char => {
                                        ignore_all!($char_cast); // Only include for types with CharCast
                                        unary_definitions::cast_to_char()
                                    }
                                )?
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
                            _ => return None,
                        })
                    }
                }
            }
        }

        impl HasValueType for $integer_type {
            fn value_type(&self) -> &'static str {
                stringify!($integer_type)
            }
        }

        impl ToExpressionValue for $integer_type {
            fn into_value(self) -> ExpressionValue {
                ExpressionValue::Integer(IntegerExpression {
                    value: IntegerExpressionValue::$integer_enum_variant(self),
                })
            }
        }

        impl HandleBinaryOperation for $integer_type {
            fn type_name() -> &'static str {
                stringify!($integer_type)
            }

            fn handle_paired_binary_operation(self, rhs: Self, operation: &PairedBinaryOperation) -> ExecutionResult<ExpressionValue> {
                let lhs = self;
                let overflow_error = || format!("The {} operation {:?} {} {:?} overflowed", stringify!($integer_type), lhs, operation.symbolic_description(), rhs);
                Ok(match operation {
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
                rhs: IntegerExpression,
                operation: &IntegerBinaryOperation,
            ) -> ExecutionResult<ExpressionValue> {
                let lhs = self;
                Ok(match operation {
                    IntegerBinaryOperation::ShiftLeft { .. } => {
                        match rhs.value {
                            IntegerExpressionValue::Untyped(rhs) => operation.output(lhs << rhs.parse_fallback()?),
                            IntegerExpressionValue::U8(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::U16(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::U32(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::U64(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::U128(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::Usize(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::I8(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::I16(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::I32(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::I64(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::I128(rhs) => operation.output(lhs << rhs),
                            IntegerExpressionValue::Isize(rhs) => operation.output(lhs << rhs),
                        }
                    },
                    IntegerBinaryOperation::ShiftRight { .. } => {
                        match rhs.value {
                            IntegerExpressionValue::Untyped(rhs) => operation.output(lhs >> rhs.parse_fallback()?),
                            IntegerExpressionValue::U8(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::U16(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::U32(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::U64(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::U128(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::Usize(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::I8(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::I16(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::I32(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::I64(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::I128(rhs) => operation.output(lhs >> rhs),
                            IntegerExpressionValue::Isize(rhs) => operation.output(lhs >> rhs),
                        }
                    },
                })
            }
        }
    )*};
}

impl_int_operations!(
    U8TypeData mod u8_interface: [CharCast[yes],] U8(u8),
    U16TypeData mod u16_interface: [] U16(u16),
    U32TypeData mod u32_interface: [] U32(u32),
    U64TypeData mod u64_interface: [] U64(u64),
    U128TypeData mod u128_interface: [] U128(u128),
    UsizeTypeData mod usize_interface: [] Usize(usize),
    I8TypeData mod i8_interface: [Signed[yes],] I8(i8),
    I16TypeData mod i16_interface: [Signed[yes],] I16(i16),
    I32TypeData mod i32_interface: [Signed[yes],] I32(i32),
    I64TypeData mod i64_interface: [Signed[yes],] I64(i64),
    I128TypeData mod i128_interface: [Signed[yes],] I128(i128),
    IsizeTypeData mod isize_interface: [Signed[yes],] Isize(isize),
);

impl_resolvable_argument_for! {
    IntegerTypeData,
    (value, context) -> IntegerExpression {
        match value {
            ExpressionValue::Integer(value) => Ok(value),
            other => context.err("integer", other),
        }
    }
}

pub(crate) struct UntypedIntegerFallback(pub(crate) FallbackInteger);

impl ResolvableArgumentTarget for UntypedIntegerFallback {
    type ValueType = UntypedIntegerTypeData;
}

impl ResolvableArgumentOwned for UntypedIntegerFallback {
    fn resolve_from_value(
        input_value: ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        let value: UntypedInteger =
            ResolvableArgumentOwned::resolve_from_value(input_value, context)?;
        Ok(UntypedIntegerFallback(value.parse_fallback()?))
    }
}

impl_resolvable_argument_for! {
    UntypedIntegerTypeData,
    (value, context) -> UntypedInteger {
        match value {
            ExpressionValue::Integer(IntegerExpression { value: IntegerExpressionValue::Untyped(x), ..}) => Ok(x),
            _ => context.err("untyped integer", value),
        }
    }
}

macro_rules! impl_resolvable_integer_subtype {
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
                    ExpressionValue::Integer(IntegerExpression {
                        value: IntegerExpressionValue::Untyped(x),
                        ..
                    }) => x.parse_as(),
                    ExpressionValue::Integer(IntegerExpression {
                        value: IntegerExpressionValue::$variant(x),
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
                    ExpressionValue::Integer(IntegerExpression {
                        value: IntegerExpressionValue::$variant(x),
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
                    ExpressionValue::Integer(IntegerExpression {
                        value: IntegerExpressionValue::$variant(x),
                        ..
                    }) => Ok(x),
                    other => context.err($expected_msg, other),
                }
            }
        }
    };
}

impl_resolvable_integer_subtype!(I8TypeData, i8, I8, "i8");
impl_resolvable_integer_subtype!(I16TypeData, i16, I16, "i16");
impl_resolvable_integer_subtype!(I32TypeData, i32, I32, "i32");
impl_resolvable_integer_subtype!(I64TypeData, i64, I64, "i64");
impl_resolvable_integer_subtype!(I128TypeData, i128, I128, "i128");
impl_resolvable_integer_subtype!(IsizeTypeData, isize, Isize, "isize");
impl_resolvable_integer_subtype!(U8TypeData, u8, U8, "u8");
impl_resolvable_integer_subtype!(U16TypeData, u16, U16, "u16");
impl_resolvable_integer_subtype!(U32TypeData, u32, U32, "u32");
impl_resolvable_integer_subtype!(U64TypeData, u64, U64, "u64");
impl_resolvable_integer_subtype!(U128TypeData, u128, U128, "u128");
impl_resolvable_integer_subtype!(UsizeTypeData, usize, Usize, "usize");
