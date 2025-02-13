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
    Array(ExpressionArray),
    Stream(ExpressionStream),
    Iterator(ExpressionIterator),
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
                        return operation.execution_err(format!("The {} operator cannot infer a common integer operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbolic_description(), left_value.value_type(), right_value.value_type()));
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
                        return operation.execution_err(format!("The {} operator cannot infer a common float operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbolic_description(), left_value.value_type(), right_value.value_type()));
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
            (ExpressionValue::Array(left), ExpressionValue::Array(right)) => {
                EvaluationLiteralPair::ArrayPair(left, right)
            }
            (ExpressionValue::Stream(left), ExpressionValue::Stream(right)) => {
                EvaluationLiteralPair::StreamPair(left, right)
            }
            (left, right) => {
                return operation.execution_err(format!("Cannot infer common type from {} {} {}. Consider using `as` to cast the operands to matching types.", left.value_type(), operation.symbolic_description(), right.value_type()));
            }
        })
    }

    pub(crate) fn into_integer(self) -> Option<ExpressionInteger> {
        match self {
            ExpressionValue::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn expect_bool(self, place_descriptor: &str) -> ExecutionResult<ExpressionBoolean> {
        match self {
            ExpressionValue::Boolean(value) => Ok(value),
            other => other.execution_err(format!(
                "{} must be a boolean, but it is a {}",
                place_descriptor,
                other.value_type(),
            )),
        }
    }

    pub(crate) fn expect_integer(
        self,
        place_descriptor: &str,
    ) -> ExecutionResult<ExpressionInteger> {
        match self {
            ExpressionValue::Integer(value) => Ok(value),
            other => other.execution_err(format!(
                "{} must be an integer, but it is a {}",
                place_descriptor,
                other.value_type(),
            )),
        }
    }

    pub(crate) fn expect_string(self, place_descriptor: &str) -> ExecutionResult<ExpressionString> {
        match self {
            ExpressionValue::String(value) => Ok(value),
            other => other.execution_err(format!(
                "{} must be a string, but it is a {}",
                place_descriptor,
                other.value_type(),
            )),
        }
    }

    pub(crate) fn expect_array(self, place_descriptor: &str) -> ExecutionResult<ExpressionArray> {
        match self {
            ExpressionValue::Array(value) => Ok(value),
            other => other.execution_err(format!(
                "{} must be an array, but it is a {}",
                place_descriptor,
                other.value_type(),
            )),
        }
    }

    pub(crate) fn expect_stream(self, place_descriptor: &str) -> ExecutionResult<ExpressionStream> {
        match self {
            ExpressionValue::Stream(value) => Ok(value),
            other => other.execution_err(format!(
                "{} must be a stream, but it is a {}",
                place_descriptor,
                other.value_type(),
            )),
        }
    }

    pub(crate) fn expect_any_iterator(
        self,
        place_descriptor: &str,
    ) -> ExecutionResult<ExpressionIterator> {
        match self {
            ExpressionValue::Array(value) => Ok(ExpressionIterator::new_for_array(value)),
            ExpressionValue::Stream(value) => Ok(ExpressionIterator::new_for_stream(value)),
            ExpressionValue::Iterator(value) => Ok(value),
            other => other.execution_err(format!(
                "{} must be iterable (an array or stream or iterator), but it is a {}",
                place_descriptor,
                other.value_type(),
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
            ExpressionValue::Array(value) => value.handle_unary_operation(operation),
            ExpressionValue::UnsupportedLiteral(value) => operation.unsupported(value),
            ExpressionValue::Iterator(value) => value.handle_unary_operation(operation),
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
            ExpressionValue::Array(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Stream(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Iterator(value) => operation.unsupported(value),
        }
    }

    pub(crate) fn create_range(
        self,
        other: Self,
        range_limits: &syn::RangeLimits,
    ) -> ExecutionResult<Box<dyn CustomExpressionIterator>> {
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
            Self::Array(value) => &mut value.span_range,
            Self::Stream(value) => &mut value.span_range,
            Self::Iterator(value) => &mut value.span_range,
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

    pub(crate) fn into_new_output_stream(
        self,
        grouping: Grouping,
        behaviour: StreamOutputBehaviour,
    ) -> ExecutionResult<OutputStream> {
        Ok(match (self, grouping) {
            (Self::Stream(value), Grouping::Flattened) => value.value,
            (other, grouping) => {
                let mut output = OutputStream::new();
                other.output_to(grouping, &mut output, behaviour)?;
                output
            }
        })
    }

    pub(crate) fn output_to(
        &self,
        grouping: Grouping,
        output: &mut OutputStream,
        behaviour: StreamOutputBehaviour,
    ) -> ExecutionResult<()> {
        match grouping {
            Grouping::Grouped => {
                // Grouping can be important for different values, to ensure they're read atomically
                // when the output stream is viewed as an array/iterable, e.g. in a for loop.
                // * Grouping means -1 is interpreted atomically, rather than as a punct then a number
                // * Grouping means that a stream is interpreted atomically
                output.push_grouped(
                    |inner| self.output_flattened_to(inner, behaviour),
                    Delimiter::None,
                    self.span_range().join_into_span_else_start(),
                )?;
            }
            Grouping::Flattened => {
                self.output_flattened_to(output, behaviour)?;
            }
        }
        Ok(())
    }

    fn output_flattened_to(
        &self,
        output: &mut OutputStream,
        behaviour: StreamOutputBehaviour,
    ) -> ExecutionResult<()> {
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
            Self::Array(array) => {
                if behaviour.should_output_arrays() {
                    array.output_grouped_items_to(output)?
                } else {
                    return self.execution_err("Arrays cannot be output to a stream. You likely wish to use the !for! command or if you wish to output every element, use `#(XXX as stream)` to cast the array to a stream.");
                }
            }
            Self::Stream(value) => value.value.append_cloned_into(output),
            Self::Iterator(iterator) => {
                if behaviour.should_output_iterators() {
                    iterator.clone().output_grouped_items_to(output)?
                } else {
                    return self.execution_err("Iterators cannot be output to a stream. You likely wish to use the !for! command or if you wish to output every element, use `#(XXX as stream)` to cast the iterator to a stream.");
                }
            }
        };
        Ok(())
    }

    pub(crate) fn into_debug_string_value(self) -> ExecutionResult<ExpressionValue> {
        let span_range = self.span_range();
        let value = self
            .concat_recursive(&ConcatBehaviour::debug())?
            .to_value(span_range);
        Ok(value)
    }

    pub(crate) fn concat_recursive(self, behaviour: &ConcatBehaviour) -> ExecutionResult<String> {
        let mut output = String::new();
        self.concat_recursive_into(&mut output, behaviour)?;
        Ok(output)
    }

    pub(crate) fn concat_recursive_into(
        self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        match self {
            ExpressionValue::None { .. } => {
                if behaviour.show_none_values {
                    output.push_str("None");
                }
            }
            ExpressionValue::Stream(stream) => {
                stream.concat_recursive_into(output, behaviour);
            }
            ExpressionValue::Array(array) => {
                array.concat_recursive_into(output, behaviour)?;
            }
            ExpressionValue::Iterator(iterator) => {
                iterator.concat_recursive_into(output, behaviour)?;
            }
            ExpressionValue::Integer(_)
            | ExpressionValue::Float(_)
            | ExpressionValue::Char(_)
            | ExpressionValue::Boolean(_)
            | ExpressionValue::UnsupportedLiteral(_)
            | ExpressionValue::String(_) => {
                // This isn't the most efficient, but it's less code and debug doesn't need to be super efficient.
                let mut stream = OutputStream::new();
                self.output_flattened_to(&mut stream, StreamOutputBehaviour::Standard)
                    .expect("Non-composite values should all be able to be outputted to a stream");
                stream.concat_recursive_into(output, behaviour);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) enum StreamOutputBehaviour {
    Standard,
    PermitArrays,
}

impl StreamOutputBehaviour {
    pub(super) fn should_output_arrays(&self) -> bool {
        match self {
            Self::Standard => false,
            Self::PermitArrays => true,
        }
    }

    pub(super) fn should_output_iterators(&self) -> bool {
        match self {
            Self::Standard => false,
            Self::PermitArrays => true,
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
            Self::Array(value) => value.value_type(),
            Self::Stream(value) => value.value_type(),
            Self::Iterator(value) => value.value_type(),
        }
    }
}

impl HasSpanRange for ExpressionValue {
    fn span_range(&self) -> SpanRange {
        match self {
            ExpressionValue::None(span_range) => *span_range,
            ExpressionValue::Integer(int) => int.span_range,
            ExpressionValue::Float(float) => float.span_range,
            ExpressionValue::Boolean(bool) => bool.span_range,
            ExpressionValue::String(str) => str.span_range,
            ExpressionValue::Char(char) => char.span_range,
            ExpressionValue::UnsupportedLiteral(lit) => lit.span_range,
            ExpressionValue::Array(array) => array.span_range,
            ExpressionValue::Stream(stream) => stream.span_range,
            ExpressionValue::Iterator(iterator) => iterator.span_range,
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

pub(super) enum EvaluationLiteralPair {
    Integer(ExpressionIntegerValuePair),
    Float(ExpressionFloatValuePair),
    BooleanPair(ExpressionBoolean, ExpressionBoolean),
    StringPair(ExpressionString, ExpressionString),
    CharPair(ExpressionChar, ExpressionChar),
    ArrayPair(ExpressionArray, ExpressionArray),
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
            Self::ArrayPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::StreamPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }

    pub(super) fn create_range(
        self,
        range_limits: OutputSpanned<syn::RangeLimits>,
    ) -> ExecutionResult<Box<dyn CustomExpressionIterator>> {
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

#[derive(Clone)]
pub(crate) struct ExpressionIterator {
    iterator: ExpressionIteratorInner,
    #[allow(unused)]
    pub(crate) span_range: SpanRange,
}

impl ExpressionIterator {
    pub(crate) fn new_for_array(array: ExpressionArray) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Array(array.items.into_iter()),
            span_range: array.span_range,
        }
    }

    pub(crate) fn new_for_stream(stream: ExpressionStream) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Stream(stream.value.into_iter()),
            span_range: stream.span_range,
        }
    }

    pub(crate) fn new_custom(
        iterator: Box<dyn CustomExpressionIterator>,
        span_range: SpanRange,
    ) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Other(iterator),
            span_range,
        }
    }

    pub(super) fn handle_unary_operation(
        self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        Ok(match operation.operation {
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => {
                return operation.unsupported(self)
            }
            UnaryOperation::Cast {
                target,
                target_ident,
                ..
            } => match target {
                CastTarget::Stream => operation.output(self.into_stream_with_grouped_items()?),
                CastTarget::Group => operation.output(
                    operation
                        .output(self.into_stream_with_grouped_items()?)
                        .into_new_output_stream(
                            Grouping::Grouped,
                            StreamOutputBehaviour::Standard,
                        )?,
                ),
                CastTarget::String => operation.output({
                    let mut output = String::new();
                    self.concat_recursive_into(&mut output, &ConcatBehaviour::standard())?;
                    output
                }),
                CastTarget::DebugString => {
                    operation.output(self.iterator).into_debug_string_value()?
                }
                CastTarget::Boolean
                | CastTarget::Char
                | CastTarget::Integer(_)
                | CastTarget::Float(_) => match self.singleton_value() {
                    Some(value) => value.handle_unary_operation(operation)?,
                    None => {
                        return operation.execution_err(format!(
                            "Only an iterator with one item can be cast to {}",
                            target_ident,
                        ))
                    }
                },
            },
        })
    }

    pub(crate) fn singleton_value(mut self) -> Option<ExpressionValue> {
        let first = self.next()?;
        if self.next().is_none() {
            Some(first)
        } else {
            None
        }
    }

    fn into_stream_with_grouped_items(self) -> ExecutionResult<OutputStream> {
        let mut output = OutputStream::new();
        self.output_grouped_items_to(&mut output)?;
        Ok(output)
    }

    fn output_grouped_items_to(self, output: &mut OutputStream) -> ExecutionResult<()> {
        const LIMIT: usize = 10_000;
        let span_range = self.span_range;
        for (i, item) in self.enumerate() {
            if i > LIMIT {
                return span_range.execution_err(format!("Only a maximum of {} items can be output to a stream from an iterator, to protect you from infinite loops. This can't currently be reconfigured with the iteration limit.", LIMIT));
            }
            item.output_to(
                Grouping::Grouped,
                output,
                StreamOutputBehaviour::PermitArrays,
            )?;
        }
        Ok(())
    }

    pub(crate) fn concat_recursive_into(
        self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        if behaviour.output_array_structure {
            output.push_str("[<iterator> ");
        }
        let max = self.size_hint().1;
        let span_range = self.span_range;
        for (i, item) in self.enumerate() {
            if i >= behaviour.iterator_limit {
                if behaviour.error_after_iterator_limit {
                    return span_range.execution_err(format!("To protect against infinite loops, only a maximum of {} items can be output to a string from an iterator. Try casting `as stream` to avoid this limit. This can't currently be reconfigured with the iteration limit.", behaviour.iterator_limit));
                } else {
                    if behaviour.output_array_structure {
                        match max {
                            Some(max) => output.push_str(&format!(
                                ", ..<{} further items>",
                                max.saturating_sub(i)
                            )),
                            None => output.push_str(", ..<possibly unbounded>"),
                        }
                    }
                    break;
                }
            }
            if i != 0 && behaviour.output_array_structure {
                output.push(',');
            }
            if i != 0 && behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            item.concat_recursive_into(output, behaviour)?;
        }
        if behaviour.output_array_structure {
            output.push(']');
        }
        Ok(())
    }
}

impl ToExpressionValue for ExpressionIteratorInner {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator {
            iterator: self,
            span_range,
        })
    }
}

impl ToExpressionValue for Box<dyn CustomExpressionIterator> {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator::new_custom(self, span_range))
    }
}

impl HasValueType for ExpressionIterator {
    fn value_type(&self) -> &'static str {
        "iterator"
    }
}

#[derive(Clone)]
enum ExpressionIteratorInner {
    Array(<Vec<ExpressionValue> as IntoIterator>::IntoIter),
    Stream(<OutputStream as IntoIterator>::IntoIter),
    Other(Box<dyn CustomExpressionIterator>),
}

impl<T: Iterator<Item = ExpressionValue> + Clone + 'static> CustomExpressionIterator for T {
    fn clone_box(&self) -> Box<dyn CustomExpressionIterator> {
        Box::new(self.clone())
    }
}

pub(crate) trait CustomExpressionIterator: Iterator<Item = ExpressionValue> {
    fn clone_box(&self) -> Box<dyn CustomExpressionIterator>;
}

impl Clone for Box<dyn CustomExpressionIterator> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl Iterator for ExpressionIterator {
    type Item = ExpressionValue;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iterator {
            ExpressionIteratorInner::Array(iter) => iter.next(),
            ExpressionIteratorInner::Stream(iter) => {
                let item = iter.next()?;
                let span = item.span();
                let stream: OutputStream = item.into();
                Some(stream.coerce_into_value(span.span_range()))
            }
            ExpressionIteratorInner::Other(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iterator {
            ExpressionIteratorInner::Array(iter) => iter.size_hint(),
            ExpressionIteratorInner::Stream(iter) => iter.size_hint(),
            ExpressionIteratorInner::Other(iter) => iter.size_hint(),
        }
    }
}
