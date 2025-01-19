use crate::internal_prelude::*;

pub(crate) trait IdentExt: Sized {
    fn new_bool(value: bool, span: Span) -> Self;
}

impl IdentExt for Ident {
    fn new_bool(value: bool, span: Span) -> Self {
        Ident::new(&value.to_string(), span)
    }
}

pub(crate) trait CursorExt: Sized {
    fn punct_matching(self, char: char) -> Option<(Punct, Self)>;
}

impl CursorExt for Cursor<'_> {
    fn punct_matching(self, char: char) -> Option<(Punct, Self)> {
        match self.punct() {
            Some((punct, next)) if punct.as_char() == char => Some((punct, next)),
            _ => None,
        }
    }
}

pub(crate) trait LiteralExt: Sized {
    #[allow(unused)]
    fn content_if_string(&self) -> Option<String>;
    fn content_if_string_like(&self) -> Option<String>;
}

impl LiteralExt for Literal {
    fn content_if_string(&self) -> Option<String> {
        match parse_str::<Lit>(&self.to_string()).unwrap() {
            Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        }
    }

    fn content_if_string_like(&self) -> Option<String> {
        match parse_str::<Lit>(&self.to_string()).unwrap() {
            Lit::Str(lit_str) => Some(lit_str.value()),
            Lit::Char(lit_char) => Some(lit_char.value().to_string()),
            Lit::CStr(lit_cstr) => Some(lit_cstr.value().to_string_lossy().to_string()),
            _ => None,
        }
    }
}

pub(crate) trait TokenStreamExt: Sized {
    #[allow(unused)]
    fn push(&mut self, token: TokenTree);
    fn flatten_transparent_groups(self) -> Self;
}

impl TokenStreamExt for TokenStream {
    fn push(&mut self, token: TokenTree) {
        self.extend(iter::once(token));
    }

    fn flatten_transparent_groups(self) -> Self {
        let mut output = TokenStream::new();
        for token in self {
            match token {
                TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                    output.extend(group.stream().flatten_transparent_groups());
                }
                other => output.extend(iter::once(other)),
            }
        }
        output
    }
}

pub(crate) trait TokenTreeExt: Sized {
    fn group(tokens: TokenStream, delimeter: Delimiter, span: Span) -> Self;
    fn into_singleton_group(self, delimiter: Delimiter) -> Self;
}

impl TokenTreeExt for TokenTree {
    fn group(inner_tokens: TokenStream, delimeter: Delimiter, span: Span) -> Self {
        TokenTree::Group(Group::new(delimeter, inner_tokens).with_span(span))
    }

    fn into_singleton_group(self, delimiter: Delimiter) -> Self {
        let span = self.span();
        Self::group(self.into_token_stream(), delimiter, span)
    }
}

pub(crate) trait ParserExt {
    fn parse_with<T: ContextualParse>(&self, context: T::Context) -> Result<T>;
    fn parse_all_for_interpretation(&self, span_range: SpanRange) -> Result<InterpretationStream>;
    fn try_parse_or_message<T, F: FnOnce(&Self) -> Result<T>, M: std::fmt::Display>(
        &self,
        func: F,
        message: M,
    ) -> Result<T>;
}

impl ParserExt for ParseBuffer<'_> {
    fn parse_with<T: ContextualParse>(&self, context: T::Context) -> Result<T> {
        T::parse_with_context(self, context)
    }

    fn parse_all_for_interpretation(&self, span_range: SpanRange) -> Result<InterpretationStream> {
        self.parse_with(span_range)
    }

    fn try_parse_or_message<T, F: FnOnce(&Self) -> Result<T>, M: std::fmt::Display>(
        &self,
        parse: F,
        message: M,
    ) -> Result<T> {
        let error_span = self.span();
        parse(self).map_err(|_| error_span.error(message))
    }
}

pub(crate) trait ContextualParse: Sized {
    type Context;

    fn parse_with_context(input: ParseStream, context: Self::Context) -> Result<Self>;
}

#[allow(unused)]
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
    fn err<T>(&self, message: impl std::fmt::Display) -> syn::Result<T> {
        Err(self.error(message))
    }

    fn error(&self, message: impl std::fmt::Display) -> syn::Error;
}

impl<T: HasSpanRange> SpanErrorExt for T {
    fn error(&self, message: impl std::fmt::Display) -> syn::Error {
        self.span_range().create_error(message)
    }
}

pub(crate) trait WithSpanExt {
    fn with_span(self, span: Span) -> Self;
}

impl WithSpanExt for Literal {
    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}

impl WithSpanExt for Punct {
    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}

impl WithSpanExt for Group {
    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
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

impl<T: ToTokens + AutoSpanRange> HasSpanRange for T {
    fn span_range(&self) -> SpanRange {
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

impl_auto_span_range! {
    TokenStream,
    Ident,
    Punct,
    Literal,
    syn::Expr,
    syn::ExprBinary,
    syn::ExprUnary,
    syn::BinOp,
    syn::UnOp,
    syn::Type,
    syn::TypePath,
    syn::token::DotDot,
}
