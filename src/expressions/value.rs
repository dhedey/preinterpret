use super::*;

pub(crate) enum EvaluationValue {
    Integer(EvaluationInteger),
    Float(EvaluationFloat),
    Boolean(EvaluationBoolean),
}

impl EvaluationValue {
    pub(super) fn for_literal_expression(expr: &ExprLit) -> Result<Self> {
        // https://docs.rs/syn/latest/syn/enum.Lit.html
        Ok(match &expr.lit {
            Lit::Int(lit) => Self::Integer(EvaluationInteger::for_litint(lit)?),
            Lit::Float(lit) => Self::Float(EvaluationFloat::for_litfloat(lit)?),
            Lit::Bool(lit) => Self::Boolean(EvaluationBoolean::for_litbool(lit)),
            other_literal => {
                return other_literal
                    .span()
                    .err("This literal is not supported in preinterpret expressions");
            }
        })
    }

    pub(super) fn describe_type(&self) -> &'static str {
        match self {
            Self::Integer(int) => int.value.describe_type(),
            Self::Float(float) => float.value.describe_type(),
            Self::Boolean(_) => "bool",
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> Result<EvaluationOutput> {
        match self {
            EvaluationValue::Integer(value) => value.handle_unary_operation(operation),
            EvaluationValue::Float(value) => value.handle_unary_operation(operation),
            EvaluationValue::Boolean(value) => value.handle_unary_operation(operation),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: EvaluationInteger,
        operation: BinaryOperation,
    ) -> Result<EvaluationOutput> {
        match self {
            EvaluationValue::Integer(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            EvaluationValue::Float(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            EvaluationValue::Boolean(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
        }
    }

    pub(super) fn source_span(&self) -> SpanRange {
        match self {
            EvaluationValue::Integer(integer) => integer.source_span,
            EvaluationValue::Float(float) => float.source_span,
            EvaluationValue::Boolean(boolean) => boolean.source_span,
        }
    }
}

impl HasSpanRange for EvaluationValue {
    fn span_range(&self) -> SpanRange {
        self.source_span()
    }
}

impl quote::ToTokens for EvaluationValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Integer(int) => int.to_tokens(tokens),
            Self::Float(float) => float.to_tokens(tokens),
            Self::Boolean(bool) => bool.to_tokens(tokens),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum ValueKind {
    Integer(IntegerKind),
    Float(FloatKind),
    Boolean,
}

pub(super) enum EvaluationLiteralPair {
    Integer(EvaluationIntegerValuePair),
    Float(EvaluationFloatValuePair),
    BooleanPair(EvaluationBoolean, EvaluationBoolean),
}

impl EvaluationLiteralPair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: BinaryOperation,
    ) -> Result<EvaluationOutput> {
        match self {
            Self::Integer(pair) => pair.handle_paired_binary_operation(&operation),
            Self::Float(pair) => pair.handle_paired_binary_operation(&operation),
            Self::BooleanPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, &operation),
        }
    }
}
