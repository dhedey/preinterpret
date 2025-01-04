use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct InterpretedStream {
    token_stream: TokenStream,
}

impl InterpretedStream {
    pub(crate) fn new() -> Self {
        Self {
            token_stream: TokenStream::new(),
        }
    }

    pub(crate) fn raw(token_stream: TokenStream) -> Self {
        Self { token_stream }
    }

    pub(crate) fn of_ident(ident: Ident) -> Self {
        TokenTree::Ident(ident).into()
    }

    pub(crate) fn of_literal(literal: Literal) -> Self {
        TokenTree::Literal(literal).into()
    }

    pub(crate) fn extend(&mut self, interpreted_stream: InterpretedStream) {
        self.token_stream.extend(interpreted_stream.token_stream);
    }

    pub(crate) fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
        self.token_stream.extend(iter::once(token_tree));
    }

    pub(crate) fn push_new_group(
        &mut self,
        inner_tokens: InterpretedStream,
        delimiter: Delimiter,
        span_range: SpanRange,
    ) {
        self.push_raw_token_tree(TokenTree::group(
            inner_tokens.token_stream,
            delimiter,
            span_range.span(),
        ));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.token_stream.is_empty()
    }

    pub(crate) fn into_token_stream(self) -> TokenStream {
        self.token_stream
    }
}

impl From<TokenTree> for InterpretedStream {
    fn from(value: TokenTree) -> Self {
        InterpretedStream {
            token_stream: value.into(),
        }
    }
}

impl HasSpanRange for InterpretedStream {
    fn span_range(&self) -> SpanRange {
        self.token_stream.span_range()
    }
}
