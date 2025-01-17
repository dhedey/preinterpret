use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct InterpretedStream {
    source_span_range: SpanRange,
    token_stream: TokenStream,
}

impl InterpretedStream {
    pub(crate) fn new(source_span_range: SpanRange) -> Self {
        Self {
            source_span_range,
            token_stream: TokenStream::new(),
        }
    }

    pub(crate) fn raw(empty_span_range: SpanRange, token_stream: TokenStream) -> Self {
        Self {
            source_span_range: empty_span_range,
            token_stream,
        }
    }

    pub(crate) fn extend(&mut self, interpreted_stream: InterpretedStream) {
        self.token_stream.extend(interpreted_stream.token_stream);
    }

    pub(crate) fn push_literal(&mut self, literal: Literal) {
        self.push_raw_token_tree(literal.into());
    }

    pub(crate) fn push_ident(&mut self, ident: Ident) {
        self.push_raw_token_tree(ident.into());
    }

    pub(crate) fn push_punct(&mut self, punct: Punct) {
        self.push_raw_token_tree(punct.into());
    }

    pub(crate) fn push_new_group(
        &mut self,
        inner_tokens: InterpretedStream,
        delimiter: Delimiter,
        span: Span,
    ) {
        self.push_raw_token_tree(TokenTree::group(inner_tokens.token_stream, delimiter, span));
    }

    pub(crate) fn extend_raw_tokens(&mut self, tokens: impl ToTokens) {
        tokens.to_tokens(&mut self.token_stream);
    }

    pub(crate) fn extend_raw_token_iter(&mut self, tokens: impl IntoIterator<Item = TokenTree>) {
        self.token_stream.extend(tokens);
    }

    fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
        self.token_stream.extend(iter::once(token_tree));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.token_stream.is_empty()
    }

    #[allow(unused)]
    pub(crate) fn parse_into_fields<T: 'static>(
        self,
        parser: FieldsParseDefinition<T>,
    ) -> Result<T> {
        let source_span_range = self.source_span_range;
        self.syn_parse(parser.create_syn_parser(source_span_range))
    }

    /// For a type `T` which implements `syn::Parse`, you can call this as `syn_parse(self, T::parse)`.
    /// For more complicated parsers, just pass the parsing function to this function.
    pub(crate) fn syn_parse<P: syn::parse::Parser>(self, parser: P) -> Result<P::Output> {
        parser.parse2(self.token_stream)
    }

    #[allow(unused)]
    pub(crate) fn into_singleton(self, error_message: &str) -> Result<TokenTree> {
        if self.is_empty() {
            return self.source_span_range.err(error_message);
        }
        let mut iter = self.token_stream.into_iter();
        let first = iter.next().unwrap(); // No panic because we're not empty
        match iter.next() {
            Some(_) => self.source_span_range.err(error_message),
            None => Ok(first),
        }
    }

    pub(crate) fn set_span_range(&mut self, span_range: SpanRange) {
        self.source_span_range = span_range;
    }

    pub(crate) fn into_token_stream(self) -> TokenStream {
        self.token_stream
    }

    pub(crate) fn append_cloned_into(&self, output: &mut InterpretedStream) {
        output.token_stream.extend(self.token_stream.clone())
    }
}

impl From<TokenTree> for InterpretedStream {
    fn from(value: TokenTree) -> Self {
        InterpretedStream {
            source_span_range: value.span_range(),
            token_stream: value.into(),
        }
    }
}

impl HasSpanRange for InterpretedStream {
    fn span_range(&self) -> SpanRange {
        if self.token_stream.is_empty() {
            self.source_span_range
        } else {
            self.token_stream.span_range()
        }
    }
}
