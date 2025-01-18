use crate::internal_prelude::*;

// ======================================
// How syn fits with preinterpret parsing
// ======================================
//
// TLDR: This is discussed on this syn issue, where David Tolnay suggested
// forking syn to get what we want (i.e. a more versatile TokenBuffer):
// ==> https://github.com/dtolnay/syn/issues/1842
//
// There are a few places where we support (or might wish to support) parsing
// as part of interpretation:
// * e.g. of a token stream in `CommandValueInput`
// * e.g. as part of a PARSER, from an InterpretedStream
// * e.g. of a variable, as part of incremental parsing (while_parse style loops)
//
// I spent quite a while considering whether this could be wrapping a
// `syn::parse::ParseBuffer<'a>` or `syn::buffer::Cursor<'a>`...
//
// Some commands want to performantly parse a variable or other token stream.
//
// Here we want variables to support:
// * Easy appending of tokens
// * Incremental parsing
//
// Ideally we'd want to be able to store a syn::TokenBuffer, and be able to
// append to it, and freely convert it to a syn::ParseStream, possibly even storing
// a cursor position into it.
//
// Unfortunately this isn't at all possible:
// * TokenBuffer appending isn't a thing, you can only create one (recursively) from
//   a TokenStream
// * TokenBuffer can't be converted to a ParseStream outside of the syn crate
// * For performance, a cursor stores a pointer into a TokenBuffer, so it can only be
//   used against a fixed buffer.
//
// We could probably work around these limitations by sacrificing performance and transforming
// to TokenStream and back, but probably there's a better way.
//
// What we probably want is our own abstraction, likely a fork from `syn`, which supports
// converting a Cursor into an indexed based cursor, which can safely be stored separately
// from the TokenBuffer.
//
// We could use this abstraction for InterpretedStream; and our variables could store a
// tuple of (IndexCursor, PreinterpretTokenBuffer)

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

    pub(crate) fn push_grouped(
        &mut self,
        appender: impl FnOnce(&mut Self) -> Result<()>,
        delimiter: Delimiter,
        span: Span,
    ) -> Result<()> {
        let mut inner = Self::new(span.span_range());
        appender(&mut inner)?;
        self.push_new_group(inner, delimiter, span);
        Ok(())
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

    pub(crate) fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
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
