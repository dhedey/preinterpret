use crate::internal_prelude::*;

/// A parsed stream ready for interpretation
#[derive(Clone)]
pub(crate) struct SourceStream {
    items: Vec<SourceItem>,
    span: Span,
}

impl ContextualParse<Source> for SourceStream {
    type Context = Span;

    fn parse(input: ParseStream<Source>, span: Self::Context) -> ParseResult<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items, span })
    }
}

impl Interpret for SourceStream {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        for item in self.items {
            item.interpret_into(interpreter, output)?;
        }
        Ok(())
    }
}

impl HasSpan for SourceStream {
    fn span(&self) -> Span {
        self.span
    }
}

/// A parsed group ready for interpretation
#[derive(Clone)]
pub(crate) struct SourceGroup {
    source_delimiter: Delimiter,
    source_delim_span: DelimSpan,
    content: SourceStream,
}

impl SourceGroup {
    pub(crate) fn into_content(self) -> SourceStream {
        self.content
    }
}

impl Parse<Source> for SourceGroup {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_group()?;
        let content = content.parse_with_context(delim_span.join())?;
        Ok(Self {
            source_delimiter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }
}

impl Interpret for SourceGroup {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let inner = self.content.interpret_to_new_stream(interpreter)?;
        output.push_new_group(inner, self.source_delimiter, self.source_delim_span.join());
        Ok(())
    }
}

impl HasSpan for SourceGroup {
    fn span(&self) -> Span {
        self.source_delim_span.join()
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

impl Parse<Source> for RawGroup {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_group()?;
        let content = content.parse()?;
        Ok(Self {
            source_delimeter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }
}

impl Interpret for RawGroup {
    fn interpret_into(self, _: &mut Interpreter, output: &mut OutputStream) -> ExecutionResult<()> {
        output.push_new_group(
            OutputStream::raw(self.content),
            self.source_delimeter,
            self.source_delim_span.join(),
        );
        Ok(())
    }
}

impl HasSpan for RawGroup {
    fn span(&self) -> Span {
        self.source_delim_span.join()
    }
}
