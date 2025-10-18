use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionStream {
    pub(crate) value: OutputStream,
}

impl ExpressionStream {
    pub(super) fn handle_integer_binary_operation(
        self,
        _right: ExpressionInteger,
        operation: WrappedOp<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: WrappedOp<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        Ok(match operation.operation {
            PairedBinaryOperation::Addition { .. } => operation.output({
                let mut stream = lhs;
                rhs.append_into(&mut stream);
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
        // %[$input].error("Expected 100, got " + %[$input].to_debug_string())
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
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Stream(ExpressionStream { value: self })
    }
}

impl ToExpressionValue for TokenStream {
    fn into_value(self) -> ExpressionValue {
        OutputStream::raw(self).into_value()
    }
}

define_interface! {
    struct StreamTypeData,
    parent: IterableTypeData,
    pub(crate) mod stream_interface {
        pub(crate) mod methods {
            // This is also on iterable, but is specialized here for performance
            fn len(this: AnyRef<OutputStream>) -> usize {
                this.len()
            }

            // This is also on iterable, but is specialized here for performance
            fn is_empty(this: AnyRef<OutputStream>) -> bool {
                this.is_empty()
            }

            fn flatten(this: OutputStream) -> ExecutionResult<TokenStream> {
                Ok(this.to_token_stream_removing_any_transparent_groups())
            }

            fn infer(this: OutputStream) -> ExecutionResult<ExpressionValue> {
                Ok(this.coerce_into_value())
            }

            fn split(this: OutputStream, separator: AnyRef<OutputStream>, settings: Option<SplitSettings>) -> ExecutionResult<ExpressionArray> {
                handle_split(this, &separator, settings.unwrap_or_default())
            }

            // STRING-BASED CONVERSION METHODS
            // ===============================

            [context] fn to_ident(this: SpannedAnyRef<OutputStream>) -> ExecutionResult<Ident> {
                let string = this.concat_recursive(&ConcatBehaviour::standard(this.span_range()));
                string_interface::methods::to_ident(context, string.as_str().into_spanned_ref(this.span_range()))
            }

            [context] fn to_ident_camel(this: SpannedAnyRef<OutputStream>) -> ExecutionResult<Ident> {
                let string = this.concat_recursive(&ConcatBehaviour::standard(this.span_range()));
                string_interface::methods::to_ident_camel(context, string.as_str().into_spanned_ref(this.span_range()))
            }

            [context] fn to_ident_snake(this: SpannedAnyRef<OutputStream>) -> ExecutionResult<Ident> {
                let string = this.concat_recursive(&ConcatBehaviour::standard(this.span_range()));
                string_interface::methods::to_ident_snake(context, string.as_str().into_spanned_ref(this.span_range()))
            }

            [context] fn to_ident_upper_snake(this: SpannedAnyRef<OutputStream>) -> ExecutionResult<Ident> {
                let string = this.concat_recursive(&ConcatBehaviour::standard(this.span_range()));
                string_interface::methods::to_ident_upper_snake(context, string.as_str().into_spanned_ref(this.span_range()))
            }

            [context] fn to_literal(this: SpannedAnyRef<OutputStream>) -> ExecutionResult<Literal> {
                let string = this.concat_recursive(&ConcatBehaviour::literal(this.span_range()));
                string_interface::methods::to_literal(context, string.as_str().into_spanned_ref(this.span_range()))
            }

            // CORE METHODS
            // ============

            fn error(this: Shared<ExpressionStream>, message: Shared<String>) -> ExecutionResult<Never> {
                let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                error_span_range.execution_err(message.as_str())
            }

            fn assert(this: Shared<ExpressionStream>, condition: bool, message: Option<AnyRef<str>>) -> ExecutionResult<()> {
                if condition {
                    Ok(())
                } else {
                    let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                    let message = match message {
                        Some(ref m) => m,
                        None => "Assertion failed",
                    };
                    error_span_range.execution_err(message)
                }
            }

            fn assert_eq(this: Shared<ExpressionStream>, lhs: SpannedAnyRef<ExpressionValue>, rhs: SpannedAnyRef<ExpressionValue>, message: Option<AnyRef<str>>) -> ExecutionResult<()> {
                let lhs_value: &ExpressionValue = &lhs;
                let rhs_value: &ExpressionValue = &rhs;
                let res = {
                    // TODO[operation-refactor]: Replace with eq when we have a solid implementation
                    let lhs_debug_str = lhs_value.concat_recursive(&ConcatBehaviour::debug(lhs.span_range()))?;
                    let rhs_debug_str = rhs_value.concat_recursive(&ConcatBehaviour::debug(rhs.span_range()))?;
                    lhs_debug_str == rhs_debug_str
                }; if res {
                    Ok(())
                } else {
                    let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                    let message = match message {
                        Some(ref m) => m.to_string(),
                        None => format!(
                            "Assertion failed: lhs != rhs, where:\n  lhs = {}\n  rhs = {}",
                            lhs.concat_recursive(&ConcatBehaviour::debug(lhs.span_range()))?,
                            rhs.concat_recursive(&ConcatBehaviour::debug(rhs.span_range()))?,
                        ),
                    };
                    error_span_range.execution_err(message)
                }
            }

            [context] fn reinterpret_as_run(this: Owned<ExpressionStream>) -> ExecutionResult<OwnedValue> {
                let source = unsafe {
                    // RUST-ANALYZER-SAFETY - We can't do any better than this, and we're about to parse it as source code,
                    // which handles groups/missing groups reasonably well (see tests)
                    this.into_inner().value.into_token_stream()
                };
                let (reparsed, scope_definitions) = source.source_parse_and_analyze(ExpressionBlockContent::parse, ExpressionBlockContent::control_flow_pass)?;
                let mut inner_interpreter = Interpreter::new(scope_definitions);
                Ok(reparsed.evaluate(&mut inner_interpreter, context.output_span_range, RequestedValueOwnership::owned())?.expect_owned())
            }

            [context] fn reinterpret_as_stream(this: Owned<ExpressionStream>) -> ExecutionResult<OutputStream> {
                let source = unsafe {
                    // RUST-ANALYZER-SAFETY - We can't do any better than this, and we're about to parse it as source code,
                    // which handles groups/missing groups reasonably well (see tests)
                    this.into_inner().value.into_token_stream()
                };
                let (reparsed, scope_definitions) = source.source_parse_and_analyze(
                    |input| SourceStream::parse_with_span(input, context.output_span_range.start()),
                    SourceStream::control_flow_pass,
                )?;
                let mut inner_interpreter = Interpreter::new(scope_definitions);
                // NB: We can't use a StreamOutput here, because it can't capture the Interpreter
                //     without some lifetime shenanigans.
                reparsed.interpret_to_new_stream(&mut inner_interpreter)
            }
        }
        pub(crate) mod unary_operations {
            [context] fn cast_to_value(this: Owned<ExpressionStream>) -> ExecutionResult<ResolvedValue> {
                let (this, span_range) = this.deconstruct();
                let coerced = this.value.coerce_into_value();
                if let ExpressionValue::Stream(_) = &coerced {
                    return span_range.execution_err("The stream could not be coerced into a single value");
                }
                // Re-run the cast operation on the coerced value
                context.operation.evaluate(coerced.into_owned(span_range))
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast {
                        target:
                            CastTarget::Boolean
                            | CastTarget::Char
                            | CastTarget::Integer(_)
                            | CastTarget::Float(_),
                        ..
                    } => unary_definitions::cast_to_value(),
                    _ => return None,
                })
            }
        }
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

impl ParseSource for StreamLiteral {
    fn parse(input: SourceParser) -> ParseResult<Self> {
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

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            StreamLiteral::Regular(lit) => lit.control_flow_pass(context),
            StreamLiteral::Raw(lit) => lit.control_flow_pass(context),
            StreamLiteral::Grouped(lit) => lit.control_flow_pass(context),
        }
    }
}

impl Interpret for StreamLiteral {
    fn interpret_into(
        &self,
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

impl Evaluate for StreamLiteral {
    type OutputValue = OutputStream;

    fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<Self::OutputValue> {
        match self {
            StreamLiteral::Regular(lit) => lit.evaluate(interpreter),
            StreamLiteral::Raw(lit) => Ok(lit.evaluate()),
            StreamLiteral::Grouped(lit) => lit.evaluate(interpreter),
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

impl ParseSource for RegularStreamLiteral {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = SourceStream::parse_with_span(&inner, brackets.span())?;
        Ok(Self {
            prefix,
            brackets,
            content,
        })
    }

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl Interpret for RegularStreamLiteral {
    fn interpret_into(
        &self,
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

impl Evaluate for RegularStreamLiteral {
    type OutputValue = OutputStream;

    fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<Self::OutputValue> {
        self.interpret_to_new_stream(interpreter)
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

impl ParseSource for RawStreamLiteral {
    fn parse(input: SourceParser) -> ParseResult<Self> {
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

    fn control_flow_pass(&self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl Interpret for RawStreamLiteral {
    fn interpret_into(
        &self,
        _interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        output.extend_raw_tokens(self.content.clone());
        Ok(())
    }
}

impl HasSpanRange for RawStreamLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.span())
    }
}

impl RawStreamLiteral {
    fn evaluate(&self) -> OutputStream {
        // Cloning a token stream is relatively cheap, but we could also
        // consider storing an Owned<TokenStream> and returning a Shared<TokenStream>
        OutputStream::raw(self.content.clone())
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

impl ParseSource for GroupedStreamLiteral {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let group = input.parse_ident_matching("group")?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = SourceStream::parse_with_span(&inner, brackets.span())?;
        Ok(Self {
            prefix,
            group,
            brackets,
            content,
        })
    }

    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl Interpret for GroupedStreamLiteral {
    fn interpret_into(
        &self,
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

impl Evaluate for GroupedStreamLiteral {
    type OutputValue = OutputStream;

    fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<OutputStream> {
        self.interpret_to_new_stream(interpreter)
    }
}
