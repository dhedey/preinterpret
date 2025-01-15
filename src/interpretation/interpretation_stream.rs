use crate::internal_prelude::*;

/// A parsed stream ready for interpretation
#[derive(Clone)]
pub(crate) struct InterpretationStream {
    items: Vec<InterpretationItem>,
    span_range: SpanRange,
}

impl InterpretationStream {
    pub(crate) fn parse_from_token_stream(
        token_stream: TokenStream,
        span_range: SpanRange,
    ) -> Result<Self> {
        Self::create_parser(span_range).parse2(token_stream)
    }
}

impl ContextualParse for InterpretationStream {
    type Context = SpanRange;

    fn parse_with_context(input: ParseStream, span_range: Self::Context) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items, span_range })
    }
}

impl Interpret for InterpretationStream {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        for item in self.items {
            item.interpret_as_tokens_into(interpreter, output)?;
        }
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        let mut inner_expression_stream = ExpressionStream::new(self.span_range);
        for item in self.items {
            item.interpret_as_expression_into(interpreter, &mut inner_expression_stream)?;
        }
        expression_stream.push_expression_group(
            inner_expression_stream,
            Delimiter::None,
            self.span_range.span(),
        );
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
    source_delimeter: Delimiter,
    source_delim_span: DelimSpan,
    interpretation_stream: InterpretationStream,
}

impl Parse for InterpretationGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let (delimeter, delim_span, content) = input.parse_any_delimiter()?;
        let span_range = delim_span.span_range();
        Ok(Self {
            source_delimeter: delimeter,
            source_delim_span: delim_span,
            interpretation_stream: content.parse_with(span_range)?,
        })
    }
}

impl InterpretationGroup {
    pub(crate) fn delimiter(&self) -> Delimiter {
        self.source_delimeter
    }

    pub(crate) fn into_inner_stream(self) -> InterpretationStream {
        self.interpretation_stream
    }
}

impl Interpret for InterpretationGroup {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.push_new_group(
            self.interpretation_stream
                .interpret_as_tokens(interpreter)?,
            self.source_delimeter,
            self.source_delim_span.join(),
        );
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        expression_stream.push_expression_group(
            self.interpretation_stream
                .interpret_as_expression(interpreter)?,
            self.source_delimeter,
            self.source_delim_span.join(),
        );
        Ok(())
    }
}

impl HasSpanRange for InterpretationGroup {
    fn span_range(&self) -> SpanRange {
        self.source_delim_span.span_range()
    }
}
