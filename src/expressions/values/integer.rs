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

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.value.to_unspanned_literal().with_span(span)
    }

    pub(crate) fn resolve_untyped_to_match(self, other: &ExpressionValue) -> ExecutionResult<Self> {
        let value = match (self.value, other) {
            (IntegerExpressionValue::Untyped(this), ExpressionValue::Integer(other)) => {
                this.into_kind(other.value.kind())?
            }
            (value, _) => value,
        };
        Ok(Self { value })
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

impl_resolvable_argument_for! {
    IntegerTypeData,
    (value, context) -> IntegerExpression {
        match value {
            ExpressionValue::Integer(value) => Ok(value),
            other => context.err("integer", other),
        }
    }
}

pub(crate) struct CoercedToU32(pub(crate) u32);

impl ResolvableArgumentTarget for CoercedToU32 {
    type ValueType = IntegerTypeData;
}

impl ResolvableArgumentOwned for CoercedToU32 {
    fn resolve_from_value(
        input_value: ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        let integer = match input_value {
            ExpressionValue::Integer(IntegerExpression { value, .. }) => value,
            other => return context.err("integer", other),
        };
        let coerced = match integer.clone() {
            IntegerExpressionValue::U8(x) => Some(x as u32),
            IntegerExpressionValue::U16(x) => Some(x as u32),
            IntegerExpressionValue::U32(x) => Some(x),
            IntegerExpressionValue::U64(x) => x.try_into().ok(),
            IntegerExpressionValue::U128(x) => x.try_into().ok(),
            IntegerExpressionValue::Usize(x) => x.try_into().ok(),
            IntegerExpressionValue::I8(x) => x.try_into().ok(),
            IntegerExpressionValue::I16(x) => x.try_into().ok(),
            IntegerExpressionValue::I32(x) => x.try_into().ok(),
            IntegerExpressionValue::I64(x) => x.try_into().ok(),
            IntegerExpressionValue::I128(x) => x.try_into().ok(),
            IntegerExpressionValue::Isize(x) => x.try_into().ok(),
            IntegerExpressionValue::Untyped(x) => x.parse_as().ok(),
        };
        match coerced {
            Some(value) => Ok(CoercedToU32(value)),
            None => context.err(
                "u32-compatible integer",
                ExpressionValue::Integer(IntegerExpression { value: integer }),
            ),
        }
    }
}
