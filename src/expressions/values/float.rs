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

impl_resolvable_argument_for! {
    FloatTypeData,
    (value, context) -> FloatExpression {
        match value {
            ExpressionValue::Float(value) => Ok(value),
            other => context.err("float", other),
        }
    }
}
