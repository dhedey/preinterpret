use super::*;

#[derive(Clone)]
pub(crate) enum EvaluationValue {
    Integer(EvaluationInteger),
    Float(EvaluationFloat),
    Boolean(EvaluationBoolean),
    String(EvaluationString),
    Char(EvaluationChar),
}

pub(super) trait ToEvaluationValue: Sized {
    fn to_value(self, source_span: Option<Span>) -> EvaluationValue;
}

impl EvaluationValue {
    pub(super) fn for_literal(lit: syn::Lit) -> ParseResult<Self> {
        // https://docs.rs/syn/latest/syn/enum.Lit.html
        Ok(match lit {
            Lit::Int(lit) => Self::Integer(EvaluationInteger::for_litint(lit)?),
            Lit::Float(lit) => Self::Float(EvaluationFloat::for_litfloat(lit)?),
            Lit::Bool(lit) => Self::Boolean(EvaluationBoolean::for_litbool(lit)),
            Lit::Str(lit) => Self::String(EvaluationString::for_litstr(lit)),
            Lit::Char(lit) => Self::Char(EvaluationChar::for_litchar(lit)),
            other_literal => {
                return other_literal
                    .span()
                    .parse_err("This literal is not supported in preinterpret expressions");
            }
        })
    }

    pub(super) fn expect_value_pair(
        self,
        operation: &PairedBinaryOperation,
        right: Self,
    ) -> ExecutionResult<EvaluationLiteralPair> {
        Ok(match (self, right) {
            (EvaluationValue::Integer(left), EvaluationValue::Integer(right)) => {
                let integer_pair = match (left.value, right.value) {
                    (EvaluationIntegerValue::Untyped(untyped_lhs), rhs) => match rhs {
                        EvaluationIntegerValue::Untyped(untyped_rhs) => {
                            EvaluationIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationIntegerValue::U8(rhs) => {
                            EvaluationIntegerValuePair::U8(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U16(rhs) => {
                            EvaluationIntegerValuePair::U16(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U32(rhs) => {
                            EvaluationIntegerValuePair::U32(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U64(rhs) => {
                            EvaluationIntegerValuePair::U64(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U128(rhs) => {
                            EvaluationIntegerValuePair::U128(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::Usize(rhs) => {
                            EvaluationIntegerValuePair::Usize(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I8(rhs) => {
                            EvaluationIntegerValuePair::I8(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I16(rhs) => {
                            EvaluationIntegerValuePair::I16(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I32(rhs) => {
                            EvaluationIntegerValuePair::I32(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I64(rhs) => {
                            EvaluationIntegerValuePair::I64(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I128(rhs) => {
                            EvaluationIntegerValuePair::I128(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::Isize(rhs) => {
                            EvaluationIntegerValuePair::Isize(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, EvaluationIntegerValue::Untyped(untyped_rhs)) => match lhs {
                        EvaluationIntegerValue::Untyped(untyped_lhs) => {
                            EvaluationIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationIntegerValue::U8(lhs) => {
                            EvaluationIntegerValuePair::U8(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U16(lhs) => {
                            EvaluationIntegerValuePair::U16(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U32(lhs) => {
                            EvaluationIntegerValuePair::U32(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U64(lhs) => {
                            EvaluationIntegerValuePair::U64(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U128(lhs) => {
                            EvaluationIntegerValuePair::U128(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::Usize(lhs) => {
                            EvaluationIntegerValuePair::Usize(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I8(lhs) => {
                            EvaluationIntegerValuePair::I8(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I16(lhs) => {
                            EvaluationIntegerValuePair::I16(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I32(lhs) => {
                            EvaluationIntegerValuePair::I32(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I64(lhs) => {
                            EvaluationIntegerValuePair::I64(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I128(lhs) => {
                            EvaluationIntegerValuePair::I128(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::Isize(lhs) => {
                            EvaluationIntegerValuePair::Isize(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (EvaluationIntegerValue::U8(lhs), EvaluationIntegerValue::U8(rhs)) => {
                        EvaluationIntegerValuePair::U8(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U16(lhs), EvaluationIntegerValue::U16(rhs)) => {
                        EvaluationIntegerValuePair::U16(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U32(lhs), EvaluationIntegerValue::U32(rhs)) => {
                        EvaluationIntegerValuePair::U32(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U64(lhs), EvaluationIntegerValue::U64(rhs)) => {
                        EvaluationIntegerValuePair::U64(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U128(lhs), EvaluationIntegerValue::U128(rhs)) => {
                        EvaluationIntegerValuePair::U128(lhs, rhs)
                    }
                    (EvaluationIntegerValue::Usize(lhs), EvaluationIntegerValue::Usize(rhs)) => {
                        EvaluationIntegerValuePair::Usize(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I8(lhs), EvaluationIntegerValue::I8(rhs)) => {
                        EvaluationIntegerValuePair::I8(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I16(lhs), EvaluationIntegerValue::I16(rhs)) => {
                        EvaluationIntegerValuePair::I16(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I32(lhs), EvaluationIntegerValue::I32(rhs)) => {
                        EvaluationIntegerValuePair::I32(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I64(lhs), EvaluationIntegerValue::I64(rhs)) => {
                        EvaluationIntegerValuePair::I64(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I128(lhs), EvaluationIntegerValue::I128(rhs)) => {
                        EvaluationIntegerValuePair::I128(lhs, rhs)
                    }
                    (EvaluationIntegerValue::Isize(lhs), EvaluationIntegerValue::Isize(rhs)) => {
                        EvaluationIntegerValuePair::Isize(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.execution_err(format!("The {} operator cannot infer a common integer operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left_value.describe_type(), right_value.describe_type()));
                    }
                };
                EvaluationLiteralPair::Integer(integer_pair)
            }
            (EvaluationValue::Boolean(left), EvaluationValue::Boolean(right)) => {
                EvaluationLiteralPair::BooleanPair(left, right)
            }
            (EvaluationValue::Float(left), EvaluationValue::Float(right)) => {
                let float_pair = match (left.value, right.value) {
                    (EvaluationFloatValue::Untyped(untyped_lhs), rhs) => match rhs {
                        EvaluationFloatValue::Untyped(untyped_rhs) => {
                            EvaluationFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationFloatValue::F32(rhs) => {
                            EvaluationFloatValuePair::F32(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationFloatValue::F64(rhs) => {
                            EvaluationFloatValuePair::F64(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, EvaluationFloatValue::Untyped(untyped_rhs)) => match lhs {
                        EvaluationFloatValue::Untyped(untyped_lhs) => {
                            EvaluationFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationFloatValue::F32(lhs) => {
                            EvaluationFloatValuePair::F32(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationFloatValue::F64(lhs) => {
                            EvaluationFloatValuePair::F64(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (EvaluationFloatValue::F32(lhs), EvaluationFloatValue::F32(rhs)) => {
                        EvaluationFloatValuePair::F32(lhs, rhs)
                    }
                    (EvaluationFloatValue::F64(lhs), EvaluationFloatValue::F64(rhs)) => {
                        EvaluationFloatValuePair::F64(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.execution_err(format!("The {} operator cannot infer a common float operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left_value.describe_type(), right_value.describe_type()));
                    }
                };
                EvaluationLiteralPair::Float(float_pair)
            }
            (EvaluationValue::String(left), EvaluationValue::String(right)) => {
                EvaluationLiteralPair::StringPair(left, right)
            }
            (EvaluationValue::Char(left), EvaluationValue::Char(right)) => {
                EvaluationLiteralPair::CharPair(left, right)
            }
            (left, right) => {
                return operation.execution_err(format!("The {} operator cannot infer a common operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left.describe_type(), right.describe_type()));
            }
        })
    }

    pub(crate) fn into_integer(self) -> Option<EvaluationInteger> {
        match self {
            EvaluationValue::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn into_bool(self) -> Option<EvaluationBoolean> {
        match self {
            EvaluationValue::Boolean(value) => Some(value),
            _ => None,
        }
    }

    /// The span is used if there isn't already a span available
    pub(super) fn to_token_tree(&self, fallback_output_span: Span) -> TokenTree {
        match self {
            Self::Integer(int) => int.to_literal(fallback_output_span).into(),
            Self::Float(float) => float.to_literal(fallback_output_span).into(),
            Self::Boolean(bool) => bool.to_ident(fallback_output_span).into(),
            Self::String(string) => string.to_literal(fallback_output_span).into(),
            Self::Char(char) => char.to_literal(fallback_output_span).into(),
        }
    }

    pub(super) fn describe_type(&self) -> &'static str {
        match self {
            Self::Integer(int) => int.value.describe_type(),
            Self::Float(float) => float.value.describe_type(),
            Self::Boolean(_) => "bool",
            Self::String(_) => "string",
            Self::Char(_) => "char",
        }
    }

    pub(super) fn source_span(&self) -> Option<Span> {
        match self {
            Self::Integer(int) => int.source_span,
            Self::Float(float) => float.source_span,
            Self::Boolean(bool) => bool.source_span,
            Self::String(str) => str.source_span,
            Self::Char(char) => char.source_span,
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: UnaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match self {
            EvaluationValue::Integer(value) => value.handle_unary_operation(operation),
            EvaluationValue::Float(value) => value.handle_unary_operation(operation),
            EvaluationValue::Boolean(value) => value.handle_unary_operation(operation),
            EvaluationValue::String(value) => value.handle_unary_operation(operation),
            EvaluationValue::Char(value) => value.handle_unary_operation(operation),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: EvaluationInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
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
            EvaluationValue::String(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            EvaluationValue::Char(value) => value.handle_integer_binary_operation(right, operation),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum CastTarget {
    Integer(IntegerKind),
    Float(FloatKind),
    Boolean,
    Char,
}

pub(super) enum EvaluationLiteralPair {
    Integer(EvaluationIntegerValuePair),
    Float(EvaluationFloatValuePair),
    BooleanPair(EvaluationBoolean, EvaluationBoolean),
    StringPair(EvaluationString, EvaluationString),
    CharPair(EvaluationChar, EvaluationChar),
}

impl EvaluationLiteralPair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: &PairedBinaryOperation,
    ) -> ExecutionResult<EvaluationValue> {
        match self {
            Self::Integer(pair) => pair.handle_paired_binary_operation(operation),
            Self::Float(pair) => pair.handle_paired_binary_operation(operation),
            Self::BooleanPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::StringPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::CharPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }
}
