use super::*;

#[derive(Clone)]
pub(crate) enum ExpressionValue {
    None(SpanRange),
    Integer(ExpressionInteger),
    Float(ExpressionFloat),
    Boolean(ExpressionBoolean),
    String(ExpressionString),
    Char(ExpressionChar),
    // Unsupported literal is a type here so that we can parse such a token
    // as a value rather than a stream, and give it better error messages
    UnsupportedLiteral(UnsupportedLiteral),
    Stream(ExpressionStream),
}

pub(crate) trait ToExpressionValue: Sized {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue;
}

impl ExpressionValue {
    pub(crate) fn for_literal(literal: Literal) -> Self {
        // The unwrap should be safe because all Literal should be parsable
        // as syn::Lit; falling back to syn::Lit::Verbatim if necessary.
        Self::for_syn_lit(literal.to_token_stream().source_parse_as().unwrap())
    }

    pub(crate) fn for_syn_lit(lit: syn::Lit) -> Self {
        // https://docs.rs/syn/latest/syn/enum.Lit.html
        match lit {
            Lit::Int(lit) => match ExpressionInteger::for_litint(&lit) {
                Ok(int) => Self::Integer(int),
                Err(_) => Self::UnsupportedLiteral(UnsupportedLiteral {
                    span_range: lit.span().span_range(),
                    lit: Lit::Int(lit),
                }),
            },
            Lit::Float(lit) => match ExpressionFloat::for_litfloat(&lit) {
                Ok(float) => Self::Float(float),
                Err(_) => Self::UnsupportedLiteral(UnsupportedLiteral {
                    span_range: lit.span().span_range(),
                    lit: Lit::Float(lit),
                }),
            },
            Lit::Bool(lit) => Self::Boolean(ExpressionBoolean::for_litbool(lit)),
            Lit::Str(lit) => Self::String(ExpressionString::for_litstr(lit)),
            Lit::Char(lit) => Self::Char(ExpressionChar::for_litchar(lit)),
            other => Self::UnsupportedLiteral(UnsupportedLiteral {
                span_range: other.span().span_range(),
                lit: other,
            }),
        }
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
                        return operation.execution_err(format!("The {} operator cannot infer a common integer operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left_value.value_type(), right_value.value_type()));
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
                        return operation.execution_err(format!("The {} operator cannot infer a common float operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbol(), left_value.value_type(), right_value.value_type()));
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
            (ExpressionValue::Stream(left), ExpressionValue::Stream(right)) => {
                EvaluationLiteralPair::StreamPair(left, right)
            }
            (left, right) => {
                return operation.execution_err(format!("Cannot infer common type from {} {} {}. Consider using `as` to cast the operands to matching types.", left.value_type(), operation.symbol(), right.value_type()));
            }
        })
    }

    pub(crate) fn into_integer(self) -> Option<ExpressionInteger> {
        match self {
            ExpressionValue::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn into_bool(self) -> Result<ExpressionBoolean, &'static str> {
        match self {
            ExpressionValue::Boolean(value) => Ok(value),
            other => Err(other.value_type()),
        }
    }

    pub(crate) fn expect_bool(self, place_descriptor: &str) -> ExecutionResult<ExpressionBoolean> {
        let error_span = self.span_range();
        match self.into_bool() {
            Ok(boolean) => Ok(boolean),
            Err(value_type) => error_span.execution_err(format!(
                "{} must be a boolean, but it is a {}",
                place_descriptor, value_type,
            )),
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            ExpressionValue::None(_) => operation.unsupported(self),
            ExpressionValue::Integer(value) => value.handle_unary_operation(operation),
            ExpressionValue::Float(value) => value.handle_unary_operation(operation),
            ExpressionValue::Boolean(value) => value.handle_unary_operation(operation),
            ExpressionValue::String(value) => value.handle_unary_operation(operation),
            ExpressionValue::Char(value) => value.handle_unary_operation(operation),
            ExpressionValue::Stream(value) => value.handle_unary_operation(operation),
            ExpressionValue::UnsupportedLiteral(value) => operation.unsupported(value),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            ExpressionValue::None(_) => operation.unsupported(self),
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
            ExpressionValue::UnsupportedLiteral(value) => operation.unsupported(value),
            ExpressionValue::Stream(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
        }
    }

    pub(crate) fn create_range(
        self,
        other: Self,
        range_limits: &syn::RangeLimits,
    ) -> ExecutionResult<Box<dyn Iterator<Item = ExpressionValue> + '_>> {
        let span_range = SpanRange::new_between(self.span_range().start(), self.span_range().end());
        self.expect_value_pair(range_limits, other)?
            .create_range(range_limits.with_output_span_range(span_range))
    }

    fn span_range_mut(&mut self) -> &mut SpanRange {
        match self {
            Self::None(span_range) => span_range,
            Self::Integer(value) => &mut value.span_range,
            Self::Float(value) => &mut value.span_range,
            Self::Boolean(value) => &mut value.span_range,
            Self::String(value) => &mut value.span_range,
            Self::Char(value) => &mut value.span_range,
            Self::UnsupportedLiteral(value) => &mut value.span_range,
            Self::Stream(value) => &mut value.span_range,
        }
    }

    pub(crate) fn with_span(mut self, source_span: Span) -> ExpressionValue {
        *self.span_range_mut() = source_span.span_range();
        self
    }

    pub(crate) fn with_span_range(mut self, source_span_range: SpanRange) -> ExpressionValue {
        *self.span_range_mut() = source_span_range;
        self
    }

    pub(crate) fn into_new_output_stream(self, grouping: Grouping) -> OutputStream {
        match (self, grouping) {
            (Self::Stream(value), Grouping::Flattened) => value.value,
            (other, grouping) => {
                let mut output = OutputStream::new();
                other.output_to(grouping, &mut output);
                output
            }
        }
    }

    pub(crate) fn output_to(&self, grouping: Grouping, output: &mut OutputStream) {
        match grouping {
            Grouping::Grouped => {
                // Grouping can be important for different values, to ensure they're read atomically
                // when the output stream is viewed as an array/iterable, e.g. in a for loop.
                // * Grouping means -1 is interpreted atomically, rather than as a punct then a number
                // * Grouping means that a stream is interpreted atomically
                let span = self.span_range().join_into_span_else_start();
                output
                    .push_grouped(
                        |inner| {
                            self.output_flattened_to(inner);
                            Ok(())
                        },
                        Delimiter::None,
                        span,
                    )
                    .unwrap()
            }
            Grouping::Flattened => {
                self.output_flattened_to(output);
            }
        }
    }

    fn output_flattened_to(&self, output: &mut OutputStream) {
        match self {
            Self::None { .. } => {}
            Self::Integer(value) => output.push_literal(value.to_literal()),
            Self::Float(value) => output.push_literal(value.to_literal()),
            Self::Boolean(value) => output.push_ident(value.to_ident()),
            Self::String(value) => output.push_literal(value.to_literal()),
            Self::Char(value) => output.push_literal(value.to_literal()),
            Self::UnsupportedLiteral(literal) => {
                output.extend_raw_tokens(literal.lit.to_token_stream())
            }
            Self::Stream(value) => value.value.append_cloned_into(output),
        }
    }
}

pub(crate) enum Grouping {
    Grouped,
    Flattened,
}

impl HasValueType for ExpressionValue {
    fn value_type(&self) -> &'static str {
        match self {
            Self::None { .. } => "none",
            Self::Integer(value) => value.value_type(),
            Self::Float(value) => value.value_type(),
            Self::Boolean(value) => value.value_type(),
            Self::String(value) => value.value_type(),
            Self::Char(value) => value.value_type(),
            Self::UnsupportedLiteral(value) => value.value_type(),
            Self::Stream(value) => value.value_type(),
        }
    }
}

impl HasSpanRange for ExpressionValue {
    fn span_range(&self) -> SpanRange {
        match self {
            Self::None(span_range) => *span_range,
            Self::Integer(int) => int.span_range,
            Self::Float(float) => float.span_range,
            Self::Boolean(bool) => bool.span_range,
            Self::String(str) => str.span_range,
            Self::Char(char) => char.span_range,
            Self::UnsupportedLiteral(lit) => lit.span_range,
            Self::Stream(stream) => stream.span_range,
        }
    }
}

pub(super) trait HasValueType {
    fn value_type(&self) -> &'static str;
}

#[derive(Clone)]
pub(crate) struct UnsupportedLiteral {
    lit: syn::Lit,
    span_range: SpanRange,
}

impl HasValueType for UnsupportedLiteral {
    fn value_type(&self) -> &'static str {
        "unsupported literal"
    }
}

#[derive(Copy, Clone)]
pub(super) enum CastTarget {
    Integer(IntegerKind),
    Float(FloatKind),
    Boolean,
    Char,
    Stream,
    Group,
}

pub(super) enum EvaluationLiteralPair {
    Integer(ExpressionIntegerValuePair),
    Float(ExpressionFloatValuePair),
    BooleanPair(ExpressionBoolean, ExpressionBoolean),
    StringPair(ExpressionString, ExpressionString),
    CharPair(ExpressionChar, ExpressionChar),
    StreamPair(ExpressionStream, ExpressionStream),
}

impl EvaluationLiteralPair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            Self::Integer(pair) => pair.handle_paired_binary_operation(operation),
            Self::Float(pair) => pair.handle_paired_binary_operation(operation),
            Self::BooleanPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::StringPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::CharPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::StreamPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }

    pub(super) fn create_range(
        self,
        range_limits: OutputSpanned<syn::RangeLimits>,
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
