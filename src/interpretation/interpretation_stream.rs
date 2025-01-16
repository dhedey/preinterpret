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

    fn create_parser(context: SpanRange) -> impl FnOnce(ParseStream) -> Result<Self> {
        move |input: ParseStream| Self::parse_with_context(input, context)
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
}

impl Express for InterpretationStream {
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
    content: InterpretationGroupContent,
}

#[derive(Clone)]
enum InterpretationGroupContent {
    Interpeted(InterpretationStream),
    Raw(TokenStream),
}

impl Parse for InterpretationGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let (delimiter, delim_span, content) = input.parse_any_delimiter()?;
        let span_range = delim_span.span_range();
        let content = match delimiter {
            // This is likely from a macro variable or macro expansion.
            // Either way, we shouldn't be interpreting it.
            Delimiter::None => InterpretationGroupContent::Raw(content.parse()?),
            _ => InterpretationGroupContent::Interpeted(content.parse_with(span_range)?),
        };
        Ok(Self {
            source_delimeter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }
}

impl Interpret for InterpretationGroup {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let inner = match self.content {
            InterpretationGroupContent::Interpeted(stream) => {
                stream.interpret_as_tokens(interpreter)?
            }
            InterpretationGroupContent::Raw(token_stream) => {
                InterpretedStream::raw(self.source_delim_span.span_range(), token_stream)
            }
        };
        output.push_new_group(inner, self.source_delimeter, self.source_delim_span.join());
        Ok(())
    }
}

impl Express for InterpretationGroup {
    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        match self.content {
            InterpretationGroupContent::Interpeted(stream) => expression_stream
                .push_expression_group(
                    stream.interpret_as_expression(interpreter)?,
                    self.source_delimeter,
                    self.source_delim_span.join(),
                ),
            InterpretationGroupContent::Raw(token_stream) => expression_stream
                .push_grouped_interpreted_stream(
                    InterpretedStream::raw(self.source_delim_span.span_range(), token_stream),
                    self.source_delim_span.join(),
                ),
        }
        Ok(())
    }
}

impl HasSpanRange for InterpretationGroup {
    fn span_range(&self) -> SpanRange {
        self.source_delim_span.span_range()
    }
}
