use super::*;

#[derive(Clone)]
pub(crate) enum ExpressionValue {
    Integer(ExpressionInteger),
    Float(ExpressionFloat),
    Boolean(ExpressionBoolean),
    String(ExpressionString),
    Char(ExpressionChar),
}

pub(super) trait ToExpressionValue: Sized {
    fn to_value(self, source_span: Option<Span>) -> ExpressionValue;
}

impl ExpressionValue {
    pub(super) fn for_literal(lit: syn::Lit) -> ParseResult<Self> {
        // https://docs.rs/syn/latest/syn/enum.Lit.html
        Ok(match lit {
            Lit::Int(lit) => Self::Integer(ExpressionInteger::for_litint(lit)?),
            Lit::Float(lit) => Self::Float(ExpressionFloat::for_litfloat(lit)?),
            Lit::Bool(lit) => Self::Boolean(ExpressionBoolean::for_litbool(lit)),
            Lit::Str(lit) => Self::String(ExpressionString::for_litstr(lit)),
            Lit::Char(lit) => Self::Char(ExpressionChar::for_litchar(lit)),
            other_literal => {
                return other_literal
                    .span()
                    .parse_err("This literal is not supported in preinterpret expressions");
            }
        })
    }

    pub(super) fn expect_value_pair(
        self,
        operation: &impl Operation,
        right: Self,
    ) -> ExecutionResult<EvaluationLiteralPair> {
        Ok(match (self, right) {
            (ExpressionValue::Integer(left), ExpressionValue::Integer(right)) => {
                let integer_pair = match (left.value, right.value) {
                    (ExpressionIntegerValue::Untyped(untyped_lhs), rhs) => match rhs {
                        ExpressionIntegerValue::Untyped(untyped_rhs) => {
                            ExpressionIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionIntegerValue::U8(rhs) => {
                            ExpressionIntegerValuePair::U8(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U16(rhs) => {
                            ExpressionIntegerValuePair::U16(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U32(rhs) => {
                            ExpressionIntegerValuePair::U32(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U64(rhs) => {
                            ExpressionIntegerValuePair::U64(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U128(rhs) => {
                            ExpressionIntegerValuePair::U128(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::Usize(rhs) => {
                            ExpressionIntegerValuePair::Usize(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I8(rhs) => {
                            ExpressionIntegerValuePair::I8(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I16(rhs) => {
                            ExpressionIntegerValuePair::I16(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I32(rhs) => {
                            ExpressionIntegerValuePair::I32(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I64(rhs) => {
                            ExpressionIntegerValuePair::I64(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I128(rhs) => {
                            ExpressionIntegerValuePair::I128(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::Isize(rhs) => {
                            ExpressionIntegerValuePair::Isize(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, ExpressionIntegerValue::Untyped(untyped_rhs)) => match lhs {
                        ExpressionIntegerValue::Untyped(untyped_lhs) => {
                            ExpressionIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionIntegerValue::U8(lhs) => {
                            ExpressionIntegerValuePair::U8(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U16(lhs) => {
                            ExpressionIntegerValuePair::U16(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U32(lhs) => {
                            ExpressionIntegerValuePair::U32(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U64(lhs) => {
                            ExpressionIntegerValuePair::U64(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U128(lhs) => {
                            ExpressionIntegerValuePair::U128(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::Usize(lhs) => {
                            ExpressionIntegerValuePair::Usize(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I8(lhs) => {
                            ExpressionIntegerValuePair::I8(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I16(lhs) => {
                            ExpressionIntegerValuePair::I16(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I32(lhs) => {
                            ExpressionIntegerValuePair::I32(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I64(lhs) => {
                            ExpressionIntegerValuePair::I64(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I128(lhs) => {
                            ExpressionIntegerValuePair::I128(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::Isize(lhs) => {
                            ExpressionIntegerValuePair::Isize(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (ExpressionIntegerValue::U8(lhs), ExpressionIntegerValue::U8(rhs)) => {
                        ExpressionIntegerValuePair::U8(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U16(lhs), ExpressionIntegerValue::U16(rhs)) => {
                        ExpressionIntegerValuePair::U16(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U32(lhs), ExpressionIntegerValue::U32(rhs)) => {
                        ExpressionIntegerValuePair::U32(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U64(lhs), ExpressionIntegerValue::U64(rhs)) => {
                        ExpressionIntegerValuePair::U64(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U128(lhs), ExpressionIntegerValue::U128(rhs)) => {
                        ExpressionIntegerValuePair::U128(lhs, rhs)
                    }
                    (ExpressionIntegerValue::Usize(lhs), ExpressionIntegerValue::Usize(rhs)) => {
                        ExpressionIntegerValuePair::Usize(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I8(lhs), ExpressionIntegerValue::I8(rhs)) => {
                        ExpressionIntegerValuePair::I8(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I16(lhs), ExpressionIntegerValue::I16(rhs)) => {
                        ExpressionIntegerValuePair::I16(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I32(lhs), ExpressionIntegerValue::I32(rhs)) => {
                        ExpressionIntegerValuePair::I32(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I64(lhs), ExpressionIntegerValue::I64(rhs)) => {
                        ExpressionIntegerValuePair::I64(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I128(lhs), ExpressionIntegerValue::I128(rhs)) => {
                        ExpressionIntegerValuePair::I128(lhs, rhs)
                    }
                    (ExpressionIntegerValue::Isize(lhs), ExpressionIntegerValue::Isize(rhs)) => {
                        ExpressionIntegerValuePair::Isize(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.execution_err(format!("The {} operator cannot infer a common integer operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left_value.describe_type(), right_value.describe_type()));
                    }
                };
                EvaluationLiteralPair::Integer(integer_pair)
            }
            (ExpressionValue::Boolean(left), ExpressionValue::Boolean(right)) => {
                EvaluationLiteralPair::BooleanPair(left, right)
            }
            (ExpressionValue::Float(left), ExpressionValue::Float(right)) => {
                let float_pair = match (left.value, right.value) {
                    (ExpressionFloatValue::Untyped(untyped_lhs), rhs) => match rhs {
                        ExpressionFloatValue::Untyped(untyped_rhs) => {
                            ExpressionFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionFloatValue::F32(rhs) => {
                            ExpressionFloatValuePair::F32(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionFloatValue::F64(rhs) => {
                            ExpressionFloatValuePair::F64(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, ExpressionFloatValue::Untyped(untyped_rhs)) => match lhs {
                        ExpressionFloatValue::Untyped(untyped_lhs) => {
                            ExpressionFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionFloatValue::F32(lhs) => {
                            ExpressionFloatValuePair::F32(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionFloatValue::F64(lhs) => {
                            ExpressionFloatValuePair::F64(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (ExpressionFloatValue::F32(lhs), ExpressionFloatValue::F32(rhs)) => {
                        ExpressionFloatValuePair::F32(lhs, rhs)
                    }
                    (ExpressionFloatValue::F64(lhs), ExpressionFloatValue::F64(rhs)) => {
                        ExpressionFloatValuePair::F64(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.execution_err(format!("The {} operator cannot infer a common float operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left_value.describe_type(), right_value.describe_type()));
                    }
                };
                EvaluationLiteralPair::Float(float_pair)
            }
            (ExpressionValue::String(left), ExpressionValue::String(right)) => {
                EvaluationLiteralPair::StringPair(left, right)
            }
            (ExpressionValue::Char(left), ExpressionValue::Char(right)) => {
                EvaluationLiteralPair::CharPair(left, right)
            }
            (left, right) => {
                return operation.execution_err(format!("The {} operator cannot infer a common operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left.describe_type(), right.describe_type()));
            }
        })
    }

    pub(crate) fn into_integer(self) -> Option<ExpressionInteger> {
        match self {
            ExpressionValue::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn into_bool(self) -> Option<ExpressionBoolean> {
        match self {
            ExpressionValue::Boolean(value) => Some(value),
            _ => None,
        }
    }

    /// The span is used if there isn't already a span available
    pub(crate) fn to_token_tree(&self, fallback_output_span: Span) -> TokenTree {
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
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            ExpressionValue::Integer(value) => value.handle_unary_operation(operation),
            ExpressionValue::Float(value) => value.handle_unary_operation(operation),
            ExpressionValue::Boolean(value) => value.handle_unary_operation(operation),
            ExpressionValue::String(value) => value.handle_unary_operation(operation),
            ExpressionValue::Char(value) => value.handle_unary_operation(operation),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: ExpressionInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            ExpressionValue::Integer(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Float(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Boolean(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::String(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Char(value) => value.handle_integer_binary_operation(right, operation),
        }
    }

    pub(crate) fn create_range(
        self,
        other: Self,
        range_limits: &syn::RangeLimits,
    ) -> ExecutionResult<Box<dyn Iterator<Item = ExpressionValue> + '_>> {
        self.expect_value_pair(range_limits, other)?
            .create_range(range_limits)
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
    Integer(ExpressionIntegerValuePair),
    Float(ExpressionFloatValuePair),
    BooleanPair(ExpressionBoolean, ExpressionBoolean),
    StringPair(ExpressionString, ExpressionString),
    CharPair(ExpressionChar, ExpressionChar),
}

impl EvaluationLiteralPair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: &PairedBinaryOperation,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            Self::Integer(pair) => pair.handle_paired_binary_operation(operation),
            Self::Float(pair) => pair.handle_paired_binary_operation(operation),
            Self::BooleanPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::StringPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::CharPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }

    pub(super) fn create_range(
        self,
        range_limits: &syn::RangeLimits,
    ) -> ExecutionResult<Box<dyn Iterator<Item = ExpressionValue> + '_>> {
        Ok(match self {
            EvaluationLiteralPair::Integer(pair) => return pair.create_range(range_limits),
            EvaluationLiteralPair::CharPair(left, right) => left.create_range(right, range_limits),
            _ => {
                return range_limits
                    .execution_err("The range must be between two integers or two characters")
            }
        })
    }
}
