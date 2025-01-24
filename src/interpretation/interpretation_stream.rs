use crate::internal_prelude::*;

/// A parsed stream ready for interpretation
#[derive(Clone)]
pub(crate) struct InterpretationStream {
    items: Vec<InterpretationItem>,
    span_range: SpanRange,
}

impl ContextualParse for InterpretationStream {
    type Context = SpanRange;

    fn parse_with_context(input: ParseStream, span_range: Self::Context) -> ParseResult<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse_v2()?);
        }
        Ok(Self { items, span_range })
    }
}

impl Interpret for InterpretationStream {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        for item in self.items {
            item.interpret_into(interpreter, output)?;
        }
        Ok(())
    }
}

impl HasSpanRange for InterpretationStream {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

/// A parsed group ready for interpretation
#[derive(Clone)]
pub(crate) struct InterpretationGroup {
    source_delimiter: Delimiter,
    source_delim_span: DelimSpan,
    content: InterpretationStream,
}

impl InterpretationGroup {
    pub(crate) fn into_content(self) -> InterpretationStream {
        self.content
    }
}

impl Parse for InterpretationGroup {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_delimiter()?;
        let content = content.parse_with(delim_span.span_range())?;
        Ok(Self {
            source_delimiter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }
}

impl Interpret for InterpretationGroup {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        let inner = self.content.interpret_to_new_stream(interpreter)?;
        output.push_new_group(inner, self.source_delimiter, self.source_delim_span.join());
        Ok(())
    }
}

impl HasSpanRange for InterpretationGroup {
    fn span_range(&self) -> SpanRange {
        self.source_delim_span.span_range()
    }
}

/// A parsed group intended to be raw tokens
#[derive(Clone)]
pub(crate) struct RawGroup {
    source_delimeter: Delimiter,
    source_delim_span: DelimSpan,
    content: TokenStream,
}

#[allow(unused)]
impl RawGroup {
    pub(crate) fn into_content(self) -> TokenStream {
        self.content
    }
}

impl Parse for RawGroup {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_delimiter()?;
        let content = content.parse()?;
        Ok(Self {
            source_delimeter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }
}

impl Interpret for RawGroup {
    fn interpret_into(
        self,
        _: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        output.push_new_group(
            InterpretedStream::raw(self.content),
            self.source_delimeter,
            self.source_delim_span.join(),
        );
        Ok(())
    }
}

impl Express for RawGroup {
    fn add_to_expression(
        self,
        _: &mut Interpreter,
        expression_stream: &mut ExpressionBuilder,
    ) -> ExecutionResult<()> {
        expression_stream.push_grouped(
            |inner| {
                inner.extend_raw_tokens(self.content);
                Ok(())
            },
            self.source_delim_span.join(),
        )
    }
}

impl HasSpanRange for RawGroup {
    fn span_range(&self) -> SpanRange {
        self.source_delim_span.span_range()
    }
}
