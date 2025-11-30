use super::*;
use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum FloatValue {
    Untyped(UntypedFloat),
    F32(f32),
    F64(f64),
}

impl IntoValue for FloatValue {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}

impl FloatValue {
    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ParseResult<Owned<Self>> {
        Ok(match lit.suffix() {
            "" => Self::Untyped(UntypedFloat::new_from_lit_float(lit.clone())),
            "f32" => Self::F32(lit.base10_parse()?),
            "f64" => Self::F64(lit.base10_parse()?),
            suffix => {
                return lit.span().parse_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        }
        .into_owned(lit.span()))
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.to_unspanned_literal().with_span(span)
    }

    pub(super) fn kind(&self) -> FloatKind {
        match self {
            Self::Untyped(_) => FloatKind::Untyped,
            Self::F32(_) => FloatKind::F32,
            Self::F64(_) => FloatKind::F64,
        }
    }

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            FloatValue::Untyped(float) => float.to_unspanned_literal(),
            FloatValue::F32(float) => Literal::f32_suffixed(*float),
            FloatValue::F64(float) => Literal::f64_suffixed(*float),
        }
    }
}

impl HasValueType for FloatValue {
    fn value_type(&self) -> &'static str {
        match self {
            FloatValue::Untyped(_) => "untyped float",
            FloatValue::F32(_) => "f32",
            FloatValue::F64(_) => "f64",
        }
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
    (value, context) -> FloatValue {
        match value {
            Value::Float(value) => Ok(value),
            other => context.err("float", other),
        }
    }
}
