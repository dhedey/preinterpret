use super::*;

#[derive(Clone)]
pub(crate) struct StreamValue {
    pub(crate) value: OutputStream,
}

impl StreamValue {
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

impl HasValueKind for StreamValue {
    type SpecificKind = ValueKind;

    fn kind(&self) -> ValueKind {
        ValueKind::Stream
    }
}

impl Debug for StreamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_string = String::new();
        self.concat_recursive_into(
            &mut debug_string,
            &ConcatBehaviour::debug(Span::call_site().span_range()),
        );
        write!(f, "{}", debug_string)
    }
}

impl ValuesEqual for StreamValue {
    /// Compares two streams by their debug string representation, ignoring spans.
    /// Transparent groups (none-delimited groups) are preserved in comparison.
    /// Use `remove_transparent_groups()` before comparison if you want to ignore them.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        // Use debug concat_recursive which preserves transparent group structure
        let lhs = self
            .value
            .concat_recursive(&ConcatBehaviour::debug(Span::call_site().span_range()));
        let rhs = other
            .value
            .concat_recursive(&ConcatBehaviour::debug(Span::call_site().span_range()));
        if lhs == rhs {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

impl IntoValue for StreamValue {
    fn into_value(self) -> Value {
        Value::Stream(self)
    }
}

impl IntoValue for OutputStream {
    fn into_value(self) -> Value {
        StreamValue { value: self }.into_value()
    }
}

impl IntoValue for TokenStream {
    fn into_value(self) -> Value {
        OutputStream::raw(self).into_value()
    }
}

impl_resolvable_argument_for! {
    StreamTypeData,
    (value, context) -> StreamValue {
        match value {
            Value::Stream(value) => Ok(value),
            _ => context.err("a stream", value),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    (value: StreamValue) -> OutputStream { value.value }
);

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

            // Removes transparent (none-delimited) groups from the stream.
            // Useful before equality comparison if you want to ignore them.
            fn remove_transparent_groups(this: OutputStream) -> OutputStream {
                OutputStream::raw(this.to_token_stream_removing_any_transparent_groups())
            }

            fn infer(this: OutputStream) -> ExecutionResult<Value> {
                Ok(this.coerce_into_value())
            }

            fn split(this: OutputStream, separator: AnyRef<OutputStream>, settings: Option<SplitSettings>) -> ExecutionResult<ArrayValue> {
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

            // Some literals become Value::UnsupportedLiteral but can still be round-tripped back to a stream
            [context] fn to_literal(this: SpannedAnyRef<OutputStream>) -> ExecutionResult<Value> {
                let string = this.concat_recursive(&ConcatBehaviour::literal(this.span_range()));
                let literal = string_interface::methods::to_literal(context, string.as_str().into_spanned_ref(this.span_range()))?;
                Ok(Value::for_literal(literal).into_value())
            }

            // CORE METHODS
            // ============

            // NOTE: with_span() exists on all values, this is just a specialized mutable version for streams
            fn set_span(mut this: Mutable<StreamValue>, span_source: Shared<StreamValue>) -> ExecutionResult<()> {
                let span_range = span_source.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                this.value.replace_first_level_spans(span_range.join_into_span_else_start());
                Ok(())
            }

            fn error(this: Shared<StreamValue>, message: Shared<String>) -> ExecutionResult<Never> {
                let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                error_span_range.assertion_err(message.as_str())
            }

            fn assert(this: Shared<StreamValue>, condition: bool, message: Option<AnyRef<str>>) -> ExecutionResult<()> {
                if condition {
                    Ok(())
                } else {
                    let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                    let message = match message {
                        Some(ref m) => m,
                        None => "Assertion failed",
                    };
                    error_span_range.assertion_err(message)
                }
            }

            fn assert_eq(this: Shared<StreamValue>, lhs: SpannedAnyRef<Value>, rhs: SpannedAnyRef<Value>, message: Option<AnyRef<str>>) -> ExecutionResult<()> {
                let lhs_value: &Value = &lhs;
                let rhs_value: &Value = &rhs;
                match Value::debug_eq(lhs_value, rhs_value) {
                    Ok(()) => Ok(()),
                    Err(debug_error) => {
                        let error_span_range = this.resolve_content_span_range().unwrap_or(Span::call_site().span_range());
                        let message = match message {
                            Some(ref m) => m.to_string(),
                            None => format!(
                                "Assertion failed: {}\n  lhs = {}\n  rhs = {}",
                                debug_error.format_message(),
                                lhs.concat_recursive(&ConcatBehaviour::debug(lhs.span_range()))?,
                                rhs.concat_recursive(&ConcatBehaviour::debug(rhs.span_range()))?,
                            ),
                        };
                        error_span_range.assertion_err(message)
                    }
                }
            }

            [context] fn reinterpret_as_run(this: Owned<StreamValue>) -> ExecutionResult<OwnedValue> {
                let source = this.into_inner().value.into_token_stream();
                let (reparsed, scope_definitions) = source.source_parse_and_analyze(ExpressionBlockContent::parse, ExpressionBlockContent::control_flow_pass)?;
                let mut inner_interpreter = Interpreter::new(scope_definitions);
                let return_value = reparsed.evaluate_spanned(&mut inner_interpreter, context.output_span_range, RequestedOwnership::owned())?.expect_owned();
                if !inner_interpreter.complete().is_empty() {
                    return context.control_flow_err("reinterpret_as_run does not allow non-empty stream output")
                }
                Ok(return_value.0)
            }

            [context] fn reinterpret_as_stream(this: Owned<StreamValue>) -> ExecutionResult<OutputStream> {
                let source = this.into_inner().value.into_token_stream();
                let (reparsed, scope_definitions) = source.source_parse_and_analyze(
                    |input| SourceStream::parse_with_span(input, context.output_span_range.span_from_join_else_start()),
                    SourceStream::control_flow_pass,
                )?;
                let mut inner_interpreter = Interpreter::new(scope_definitions);
                reparsed.interpret(&mut inner_interpreter)?;
                Ok(inner_interpreter.complete())
            }
        }
        pub(crate) mod unary_operations {
            [context] fn cast_to_value(this: Owned<StreamValue>) -> ExecutionResult<ReturnedValue> {
                let this = this.into_inner();
                let coerced = this.value.coerce_into_value();
                if let Value::Stream(_) = &coerced {
                    return context.output_span_range.value_err("The stream could not be coerced into a single value");
                }
                // Re-run the cast operation on the coerced value
                Ok(context.operation.evaluate(Spanned(coerced.into_owned(), context.output_span_range))?.0)
            }
        }
        pub(crate) mod binary_operations {
            fn add(mut lhs: OutputStream, rhs: OutputStream) -> OutputStream {
                rhs.append_into(&mut lhs);
                lhs
            }

            fn add_assign(mut lhs: Assignee<OutputStream>, rhs: OutputStream) {
                rhs.append_into(&mut lhs);
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

            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    BinaryOperation::Addition { .. } => binary_definitions::add(),
                    BinaryOperation::AddAssign { .. } => binary_definitions::add_assign(),
                    _ => return None,
                })
            }
        }
    }
}

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

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            StreamLiteral::Regular(lit) => lit.control_flow_pass(context),
            StreamLiteral::Raw(lit) => lit.control_flow_pass(context),
            StreamLiteral::Grouped(lit) => lit.control_flow_pass(context),
        }
    }
}

impl Interpret for StreamLiteral {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        match self {
            StreamLiteral::Regular(lit) => lit.interpret(interpreter),
            StreamLiteral::Raw(lit) => lit.interpret(interpreter),
            StreamLiteral::Grouped(lit) => lit.interpret(interpreter),
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

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl Interpret for RegularStreamLiteral {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        self.content.interpret(interpreter)
    }
}

impl HasSpanRange for RegularStreamLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.span())
    }
}

pub(crate) struct RawStreamLiteral {
    prefix: Token![%],
    _raw: Unused<RawKeyword>,
    brackets: Brackets,
    content: TokenStream,
}

impl ParseSource for RawStreamLiteral {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let _raw = input.parse()?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = inner.parse()?;
        Ok(Self {
            prefix,
            _raw,
            brackets,
            content,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl Interpret for RawStreamLiteral {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        interpreter
            .output(self)?
            .extend_raw_tokens(self.content.clone());
        Ok(())
    }
}

impl HasSpanRange for RawStreamLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.span())
    }
}

pub(crate) struct GroupedStreamLiteral {
    prefix: Token![%],
    _group: Unused<GroupKeyword>,
    brackets: Brackets,
    content: SourceStream,
}

impl ParseSource for GroupedStreamLiteral {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let _group = input.parse()?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = SourceStream::parse_with_span(&inner, brackets.span())?;
        Ok(Self {
            prefix,
            _group,
            brackets,
            content,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl Interpret for GroupedStreamLiteral {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        interpreter.in_output_group(Delimiter::None, self.brackets.span(), |interpreter| {
            self.content.interpret(interpreter)
        })
    }
}

impl HasSpanRange for GroupedStreamLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.span())
    }
}
