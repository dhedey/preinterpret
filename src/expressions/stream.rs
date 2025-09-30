use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionStream {
    pub(crate) value: OutputStream,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(crate) span_range: SpanRange,
}

impl ExpressionStream {
    pub(super) fn handle_integer_binary_operation(
        self,
        _right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        Ok(match operation.operation {
            PairedBinaryOperation::Addition { .. } => operation.output({
                let mut stream = lhs;
                rhs.append_cloned_into(&mut stream);
                stream
            }),
            PairedBinaryOperation::Subtraction { .. }
            | PairedBinaryOperation::Multiplication { .. }
            | PairedBinaryOperation::Division { .. }
            | PairedBinaryOperation::LogicalAnd { .. }
            | PairedBinaryOperation::LogicalOr { .. }
            | PairedBinaryOperation::Remainder { .. }
            | PairedBinaryOperation::BitXor { .. }
            | PairedBinaryOperation::BitAnd { .. }
            | PairedBinaryOperation::BitOr { .. }
            | PairedBinaryOperation::Equal { .. }
            | PairedBinaryOperation::LessThan { .. }
            | PairedBinaryOperation::LessThanOrEqual { .. }
            | PairedBinaryOperation::NotEqual { .. }
            | PairedBinaryOperation::GreaterThanOrEqual { .. }
            | PairedBinaryOperation::GreaterThan { .. } => return operation.unsupported(lhs),
        })
    }

    pub(crate) fn concat_recursive_into(self, output: &mut String, behaviour: &ConcatBehaviour) {
        if behaviour.output_types_as_commands {
            if self.value.is_empty() {
                output.push_str("[!stream!]");
            } else {
                output.push_str("[!stream! ");
                self.value.concat_recursive_into(output, behaviour);
                output.push(']');
            }
        } else {
            self.value.concat_recursive_into(output, behaviour);
        }
    }
}

impl HasValueType for ExpressionStream {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

impl HasValueType for OutputStream {
    fn value_type(&self) -> &'static str {
        "stream"
    }
}

impl ToExpressionValue for OutputStream {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Stream(ExpressionStream {
            value: self,
            span_range,
        })
    }
}

impl ToExpressionValue for TokenStream {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        OutputStream::raw(self).to_value(span_range)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct StreamTypeData;

impl MethodResolutionTarget for StreamTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);

    fn resolve_own_method(method_name: &str) -> Option<MethodInterface> {
        define_method_matcher! {
            (match method_name on Self)

            fn len(this: Shared<ExpressionStream>) -> ExecutionResult<usize> {
                Ok(this.value.len())
            }

            fn flatten(this: Owned<ExpressionStream>) -> ExecutionResult<TokenStream> {
                Ok(this.into_inner().value.into_token_stream_removing_any_transparent_groups())
            }

            fn infer(this: Owned<ExpressionStream>) -> ExecutionResult<ExpressionValue> {
                let span_range = this.span_range();
                Ok(this.into_inner().value.coerce_into_value(span_range))
            }
        }
    }

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Cast {
                target:
                    CastTarget::Boolean
                    | CastTarget::Char
                    | CastTarget::Integer(_)
                    | CastTarget::Float(_),
                ..
            } => {
                wrap_unary!([Op=operation](this: Owned<ExpressionStream>) -> ExecutionResult<ResolvedValue> {
                    let (this, span_range) = this.deconstruct();
                    let coerced = this.value.coerce_into_value(span_range);
                    if let ExpressionValue::Stream(_) = &coerced {
                        return span_range.execution_err("The stream could not be coerced into a single value");
                    }
                    operation.evaluate(coerced.into())
                })
            }
            _ => return None,
        })
    }
}

#[derive(Clone)]
pub(crate) enum StreamLiteral {
    Regular(RegularStreamLiteral),
    Raw(RawStreamLiteral),
}

#[derive(Copy, Clone)]
pub(crate) enum StreamLiteralKind {
    Regular,
    Raw,
}

impl Parse<Source> for StreamLiteral {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        if let Some((_, next)) = input.cursor().punct_matching('%') {
            if next.ident_matching("raw").is_some() {
                return Ok(StreamLiteral::Raw(input.parse()?));
            } else if next.group_matching(Delimiter::Bracket).is_some() {
                return Ok(StreamLiteral::Regular(input.parse()?));
            }
        }
        input.parse_err("Expected `%[..]` or `%raw[..]` to start a stream literal")
    }
}

impl Interpret for StreamLiteral {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            StreamLiteral::Regular(lit) => lit.interpret_into(interpreter, output),
            StreamLiteral::Raw(lit) => lit.interpret_into(interpreter, output),
        }
    }
}

impl HasSpanRange for StreamLiteral {
    fn span_range(&self) -> SpanRange {
        match self {
            StreamLiteral::Regular(lit) => lit.span_range(),
            StreamLiteral::Raw(lit) => lit.span_range(),
        }
    }
}

impl InterpretToValue for StreamLiteral {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        match self {
            StreamLiteral::Regular(lit) => lit.interpret_to_value(interpreter),
            StreamLiteral::Raw(lit) => lit.interpret_to_value(interpreter),
        }
    }
}

#[derive(Clone)]
#[allow(unused)]
pub(crate) struct RegularStreamLiteral {
    prefix: Token![%],
    brackets: Brackets,
    content: SourceStream,
}

impl Parse<Source> for RegularStreamLiteral {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = inner.parse_with_context(brackets.span())?;
        Ok(Self {
            prefix,
            brackets,
            content,
        })
    }
}

impl Interpret for RegularStreamLiteral {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.content.interpret_into(interpreter, output)
    }
}

impl HasSpanRange for RegularStreamLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.span())
    }
}

impl InterpretToValue for RegularStreamLiteral {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let span_range = self.span_range();
        Ok(self
            .content
            .interpret_to_new_stream(interpreter)?
            .to_value(span_range))
    }
}

#[derive(Clone)]
#[allow(unused)]
pub(crate) struct RawStreamLiteral {
    prefix: Token![%],
    raw: Ident,
    brackets: Brackets,
    content: TokenStream,
}

impl Parse<Source> for RawStreamLiteral {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let raw = input.parse_ident_matching("raw")?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = inner.parse()?;
        Ok(Self {
            prefix,
            raw,
            brackets,
            content,
        })
    }
}

impl Interpret for RawStreamLiteral {
    fn interpret_into(
        self,
        _interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        output.extend_raw_tokens(self.content);
        Ok(())
    }
}

impl HasSpanRange for RawStreamLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.span())
    }
}

impl InterpretToValue for RawStreamLiteral {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        _interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let span_range = self.span_range();
        let value = self.content.to_value(span_range);
        Ok(value)
    }
}
