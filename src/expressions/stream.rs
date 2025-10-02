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

    pub(crate) fn concat_recursive_into(&self, output: &mut String, behaviour: &ConcatBehaviour) {
        if behaviour.use_stream_literal_syntax {
            if self.value.is_empty() {
                output.push_str("%[]");
            } else {
                output.push_str("%[");
                self.value.concat_recursive_into(output, behaviour);
                output.push(']');
            }
        } else {
            self.value.concat_recursive_into(output, behaviour);
        }
    }

    pub(crate) fn resolve_content_span_range(&self) -> Option<SpanRange> {
        // Consider the case where preinterpret embeds in a declarative macro, and we have
        // an error like this:
        // %[$input].error("Expected 100, got " + %[$input].debug_string())
        //
        // In cases like this, rustc wraps $input in a transparent group, which means that
        // the span of that group is the span of the tokens "$input" in the definition of the
        // declarative macro. This is not what we want. We want the span of the tokens which
        // were fed into $input in the declarative macro.
        //
        // The simplest solution here is to get rid of all transparent groups, to get back to the
        // source spans.
        //
        // Once this workstream with macro diagnostics is stabilised:
        // https://github.com/rust-lang/rust/issues/54140#issuecomment-802701867
        //
        // Then we can revisit this and do something better, and include all spans as separate spans
        // in the error message, which will allow a user to trace an error through N different layers
        // of macros.
        //
        // (Possibly we can try to join spans together, and if they don't join, they become separate
        // spans which get printed to the error message).
        //
        // Coincidentally, rust analyzer currently does not properly support
        // transparent groups (as of Jan 2025), so gets it right without this flattening:
        // https://github.com/rust-lang/rust-analyzer/issues/18211

        let error_span_stream = self.value.to_token_stream_removing_any_transparent_groups();
        if error_span_stream.is_empty() {
            None
        } else {
            Some(error_span_stream.span_range_from_iterating_over_all_tokens())
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
                Ok(this.into_inner().value.to_token_stream_removing_any_transparent_groups())
            }

            fn infer(this: Owned<ExpressionStream>) -> ExecutionResult<ExpressionValue> {
                let span_range = this.span_range();
                Ok(this.into_inner().value.coerce_into_value(span_range))
            }

            fn error(this: Shared<ExpressionStream>, message: Shared<String>) -> ExecutionResult<Never> {
                let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                error_span_range.execution_err(message.as_str())
            }

            fn assert(this: Shared<ExpressionStream>, condition: bool, message: Shared<String>) -> ExecutionResult<()> {
                if condition {
                    Ok(())
                } else {
                    let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                    error_span_range.execution_err(message.as_str())
                }
            }

            [context] fn reinterpret_as_run(this: Owned<ExpressionStream>) -> ExecutionResult<ExpressionValue> {
                let source = unsafe {
                    // RUST-ANALYZER-SAFETY - We can't do any better than this, and we're about to parse it as source code,
                    // which handles groups/missing groups reasonably well (see tests)
                    this.into_inner().value.into_token_stream()
                };
                let reparsed = source.source_parse_as::<ExpressionBlockContent>()?;
                reparsed.evaluate(context.interpreter, context.output_span_range)
            }

            [context] fn reinterpret_as_stream(this: Owned<ExpressionStream>) -> ExecutionResult<OutputStream> {
                let source = unsafe {
                    // RUST-ANALYZER-SAFETY - We can't do any better than this, and we're about to parse it as source code,
                    // which handles groups/missing groups reasonably well (see tests)
                    this.into_inner().value.into_token_stream()
                };
                let reparsed_source_stream = source.source_parse_with(|input| SourceStream::parse(input, context.output_span_range.start()))?;
                // NB: We can't use a StreamOutput here, because it can't capture the Interpreter
                //     without some lifetime shenanigans.
                reparsed_source_stream.interpret_to_new_stream(context.interpreter)
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
                wrap_unary!([context](this: Owned<ExpressionStream>) -> ExecutionResult<ResolvedValue> {
                    let (this, span_range) = this.deconstruct();
                    let coerced = this.value.coerce_into_value(span_range);
                    if let ExpressionValue::Stream(_) = &coerced {
                        return span_range.execution_err("The stream could not be coerced into a single value");
                    }
                    context.operation.evaluate(coerced.into())
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
    Grouped(GroupedStreamLiteral),
    // We're missing a grouped raw, but that can be achieved with %group[%raw[...]]
}

#[derive(Copy, Clone)]
pub(crate) enum StreamLiteralKind {
    Regular,
    Raw,
    Grouped,
}

impl Parse<Source> for StreamLiteral {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        if let Some((_, next)) = input.cursor().punct_matching('%') {
            if next.group_matching(Delimiter::Bracket).is_some() {
                return Ok(StreamLiteral::Regular(input.parse()?));
            } else if next.ident_matching("raw").is_some() {
                return Ok(StreamLiteral::Raw(input.parse()?));
            } else if next.ident_matching("group").is_some() {
                return Ok(StreamLiteral::Grouped(input.parse()?));
            }
        }
        input.parse_err("Expected `%[..]`, `%raw[..]` or `%group[..]` to start a stream literal")
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
            StreamLiteral::Grouped(lit) => lit.interpret_into(interpreter, output),
        }
    }
}

impl HasSpanRange for StreamLiteral {
    fn span_range(&self) -> SpanRange {
        match self {
            StreamLiteral::Regular(lit) => lit.span_range(),
            StreamLiteral::Raw(lit) => lit.span_range(),
            StreamLiteral::Grouped(lit) => lit.span_range(),
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
            StreamLiteral::Grouped(lit) => lit.interpret_to_value(interpreter),
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

#[derive(Clone)]
#[allow(unused)]
pub(crate) struct GroupedStreamLiteral {
    prefix: Token![%],
    group: Ident,
    brackets: Brackets,
    content: SourceStream,
}

impl Parse<Source> for GroupedStreamLiteral {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let group = input.parse_ident_matching("group")?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = inner.parse_with_context(brackets.span())?;
        Ok(Self {
            prefix,
            group,
            brackets,
            content,
        })
    }
}

impl Interpret for GroupedStreamLiteral {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        output.push_grouped(
            |inner| self.content.interpret_into(interpreter, inner),
            Delimiter::None,
            self.brackets.span(),
        )
    }
}

impl HasSpanRange for GroupedStreamLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.span())
    }
}

impl InterpretToValue for GroupedStreamLiteral {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let span_range = self.span_range();
        Ok(self
            .interpret_to_new_stream(interpreter)?
            .to_value(span_range))
    }
}
