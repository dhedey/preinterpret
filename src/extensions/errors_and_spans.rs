use crate::internal_prelude::*;

pub(crate) trait SynErrorExt: Sized {
    fn concat(self, extra: &str) -> Self;
}

impl SynErrorExt for syn::Error {
    fn concat(self, extra: &str) -> Self {
        let mut message = self.to_string();
        message.push_str(extra);
        Self::new(self.span(), message)
    }
}

pub(crate) trait SpanErrorExt: Sized {
    fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        Err(self.error(message).into())
    }

    fn execution_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.error(message).into())
    }

    fn err<T>(&self, message: impl std::fmt::Display) -> syn::Result<T> {
        Err(self.error(message))
    }

    fn error(&self, message: impl std::fmt::Display) -> syn::Error;

    fn execution_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::Error(self.error(message))
    }

    #[allow(unused)]
    fn parse_error(&self, message: impl std::fmt::Display) -> ParseError {
        ParseError::Standard(self.error(message))
    }
}

impl<T: HasSpanRange> SpanErrorExt for T {
    fn error(&self, message: impl std::fmt::Display) -> syn::Error {
        self.span_range().create_error(message)
    }
}

pub(crate) trait HasSpanRange {
    fn span_range(&self) -> SpanRange;

    fn span(&self) -> Span {
        self.span_range().span()
    }
}

/// [`syn::spanned`] has the limitation that it uses [`proc_macro::Span::join`]
/// and falls back to the span of the first token when not available.
///
/// Instead, [`syn::Error`] uses a trick involving a span range. This effectively
/// allows capturing this trick when we're not immediately creating an error.
///
/// When [`proc_macro::Span::join`] is stabilised and [`syn::spanned`] works,
/// we can swap [`SpanRange`] contents for [`Span`] (or even remove it and [`HasSpanRange`]).
#[derive(Copy, Clone)]
pub(crate) struct SpanRange {
    start: Span,
    end: Span,
}

impl SpanRange {
    pub(crate) fn new_single(span: Span) -> Self {
        Self {
            start: span,
            end: span,
        }
    }

    pub(crate) fn new_between(start: Span, end: Span) -> Self {
        Self { start, end }
    }

    fn create_error(&self, message: impl std::fmt::Display) -> syn::Error {
        syn::Error::new_spanned(self, message)
    }

    /// * On nightly, this gives a span covering the full range (the same result as `Span::join` would)
    /// * On stable, this gives the span of the first token of the group (because [`proc_macro::Span::join`] is not supported)
    pub(crate) fn span(&self) -> Span {
        <Self as syn::spanned::Spanned>::span(self)
    }

    pub(crate) fn set_end(&mut self, end: Span) {
        self.end = end;
    }

    #[allow(unused)]
    pub(crate) fn start(&self) -> Span {
        self.start
    }

    #[allow(unused)]
    pub(crate) fn end(&self) -> Span {
        self.end
    }
}

// This is implemented so we can create an error from it using `Error::new_spanned(..)`
impl ToTokens for SpanRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend([
            TokenTree::Punct(Punct::new('<', Spacing::Alone).with_span(self.start)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone).with_span(self.end)),
        ]);
    }
}

impl HasSpanRange for SpanRange {
    fn span_range(&self) -> SpanRange {
        *self
    }
}

impl HasSpanRange for Span {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(*self, *self)
    }
}

impl HasSpanRange for TokenTree {
    fn span_range(&self) -> SpanRange {
        self.span().span_range()
    }
}

impl HasSpanRange for Group {
    fn span_range(&self) -> SpanRange {
        self.span().span_range()
    }
}

impl HasSpanRange for DelimSpan {
    fn span_range(&self) -> SpanRange {
        // We could use self.open() => self.close() here, but using
        // self.join() is better as it can be round-tripped to a span
        // as the whole span, rather than just the start or end.
        self.join().span_range()
    }
}

pub(crate) trait SlowSpanRange {
    /// This name is purposefully very long to discourage use, as it can cause nasty performance issues
    fn span_range_from_iterating_over_all_tokens(&self) -> SpanRange;
}

impl<T: ToTokens> SlowSpanRange for T {
    fn span_range_from_iterating_over_all_tokens(&self) -> SpanRange {
        let mut iter = self.into_token_stream().into_iter();
        let start = iter.next().map_or_else(Span::call_site, |t| t.span());
        let end = iter.last().map_or(start, |t| t.span());
        SpanRange { start, end }
    }
}

/// This should only be used for syn built-ins or when there isn't a better
/// span range available
pub(crate) trait AutoSpanRange {}

macro_rules! impl_auto_span_range {
    ($($ty:ty),* $(,)?) => {
        $(
            impl AutoSpanRange for $ty {}
        )*
    };
}

impl<T: ToTokens + AutoSpanRange> HasSpanRange for T {
    fn span_range(&self) -> SpanRange {
        // AutoSpanRange should only be used for tokens with a small number of tokens
        SlowSpanRange::span_range_from_iterating_over_all_tokens(&self)
    }
}

// This should only be used for types with a bounded number of tokens
// otherwise, span_range_from_iterating_over_all_tokens() can be used
// directly with a longer name to make the performance hit clearer, so
// it's only used in error cases.
impl_auto_span_range! {
    Ident,
    Punct,
    Literal,
    syn::BinOp,
    syn::UnOp,
    syn::token::As,
    syn::token::DotDot,
    syn::token::In,
}
