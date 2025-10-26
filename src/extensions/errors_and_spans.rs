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

pub(crate) trait SpanErrorExt {
    fn syn_error(&self, message: impl std::fmt::Display) -> syn::Error;

    fn parse_error(&self, message: impl std::fmt::Display) -> ParseError {
        ParseError::Standard(self.syn_error(message))
    }

    fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        Err(self.syn_error(message).into())
    }

    fn syntax_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.syntax_error(message))
    }

    fn syntax_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::syntax_error(self.syn_error(message))
    }

    fn type_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.type_error(message))
    }

    fn type_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::type_error(self.syn_error(message))
    }

    fn control_flow_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.control_flow_error(message))
    }

    fn control_flow_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::control_flow_error(self.syn_error(message))
    }

    fn ownership_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.ownership_error(message))
    }

    fn ownership_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::ownership_error(self.syn_error(message))
    }

    fn assertion_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.assertion_error(message))
    }

    fn debug_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::debug_error(self.syn_error(message))
    }

    fn debug_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.debug_error(message))
    }

    fn assertion_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::assertion_error(self.syn_error(message))
    }

    fn value_err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        Err(self.value_error(message))
    }

    fn value_error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        ExecutionInterrupt::value_error(self.syn_error(message))
    }
}

impl<T: HasSpanRange + ?Sized> SpanErrorExt for T {
    fn syn_error(&self, message: impl std::fmt::Display) -> syn::Error {
        self.span_range().create_error(message)
    }
}

/// This is intended to be implemented only for types which have a cheap span.
/// It is cheaper than [`syn::spanned`], which requires streaming the whole type,
/// and can be very slow for e.g. large expressions.
pub(crate) trait HasSpan {
    fn span(&self) -> Span;
}

/// This is intended to be implemented only for types which have a cheap SpanRange.
/// It is cheaper than [`syn::spanned`], which requires streaming the whole type,
/// and can be very slow for e.g. large expressions.
///
/// See also [`SlowSpanRange`] for the equivalent of [`syn::spanned`].
pub(crate) trait HasSpanRange {
    fn span_range(&self) -> SpanRange;

    fn span_from_join_else_start(&self) -> Span {
        self.span_range().join_into_span_else_start()
    }
}

impl<T: HasSpan + ?Sized> HasSpanRange for T {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_single(self.span())
    }
}

impl HasSpanRange for &SpanRange {
    fn span_range(&self) -> SpanRange {
        **self
    }
}

/// [`syn::spanned`] is potentially unexpectedly expensive, and has the
/// limitation that it uses [`proc_macro::Span::join`] and falls back to the
/// span of the first token when not available.
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

    pub(crate) fn new_between(start: impl HasSpanRange, end: impl HasSpanRange) -> Self {
        Self {
            start: start.span_range().start,
            end: end.span_range().end,
        }
    }

    fn create_error(&self, message: impl std::fmt::Display) -> syn::Error {
        syn::Error::new_spanned(self, message)
    }

    /// * On nightly, this gives a span covering the full range (the same result as `Span::join` would)
    /// * On stable, this gives the span of the first token of the group (because [`proc_macro::Span::join`] is not supported)
    pub(crate) fn join_into_span_else_start(&self) -> Span {
        <Self as syn::spanned::Spanned>::span(self)
    }

    pub(crate) fn set_start(&mut self, start: Span) {
        self.start = start;
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

impl From<Span> for SpanRange {
    fn from(span: Span) -> Self {
        Self::new_single(span)
    }
}

impl HasSpanRange for SpanRange {
    fn span_range(&self) -> SpanRange {
        *self
    }
}

impl HasSpan for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl<'a> HasSpan for Cursor<'a> {
    fn span(&self) -> Span {
        Cursor::span(*self)
    }
}

impl HasSpan for TokenTree {
    fn span(&self) -> Span {
        TokenTree::span(self)
    }
}

impl HasSpan for Group {
    fn span(&self) -> Span {
        Group::span(self)
    }
}

impl HasSpan for DelimSpan {
    fn span(&self) -> Span {
        DelimSpan::join(self)
    }
}

impl HasSpan for Ident {
    fn span(&self) -> Span {
        Ident::span(self)
    }
}

impl HasSpan for Punct {
    fn span(&self) -> Span {
        Punct::span(self)
    }
}

impl HasSpan for Literal {
    fn span(&self) -> Span {
        Literal::span(self)
    }
}

/// [ ... ]
#[derive(Copy, Clone)]
pub(crate) struct Brackets {
    pub(crate) delim_span: DelimSpan,
}

impl core::ops::Deref for Brackets {
    type Target = DelimSpan;

    fn deref(&self) -> &Self::Target {
        &self.delim_span
    }
}

impl HasSpan for Brackets {
    fn span(&self) -> Span {
        self.delim_span.span()
    }
}

/// { ... }
#[derive(Copy, Clone)]
pub(crate) struct Braces {
    pub(crate) delim_span: DelimSpan,
}

impl core::ops::Deref for Braces {
    type Target = DelimSpan;

    fn deref(&self) -> &Self::Target {
        &self.delim_span
    }
}

impl HasSpan for Braces {
    fn span(&self) -> Span {
        self.delim_span.span()
    }
}

/// ( ... )
#[derive(Copy, Clone)]
pub(crate) struct Parentheses {
    pub(crate) delim_span: DelimSpan,
}

impl core::ops::Deref for Parentheses {
    type Target = DelimSpan;

    fn deref(&self) -> &Self::Target {
        &self.delim_span
    }
}

impl HasSpan for Parentheses {
    fn span(&self) -> Span {
        self.delim_span.span()
    }
}

impl HasSpan for LitFloat {
    fn span(&self) -> Span {
        LitFloat::span(self)
    }
}
impl HasSpan for LitInt {
    fn span(&self) -> Span {
        LitInt::span(self)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct TransparentDelimiters {
    pub(crate) delim_span: DelimSpan,
}

impl core::ops::Deref for TransparentDelimiters {
    type Target = DelimSpan;

    fn deref(&self) -> &Self::Target {
        &self.delim_span
    }
}

impl HasSpan for TransparentDelimiters {
    fn span(&self) -> Span {
        self.delim_span.span()
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

// This should only be used for types implementing ToTokens, which have
// a small, bounded number of tokens, with sensible performance.
macro_rules! impl_auto_span_range {
    ($($ty:ty),* $(,)?) => {
        $(
            impl HasSpanRange for $ty {
                fn span_range(&self) -> SpanRange {
                    SlowSpanRange::span_range_from_iterating_over_all_tokens(self)
                }
            }
        )*
    };
}

// This should only be used for types with a bounded number of tokens
// greater than one.
// If exactly one, implement HasSpan.
// Otherwise, span_range_from_iterating_over_all_tokens() can be used
// directly with a longer name to make the performance hit clearer, so
// it's only used in error cases.
impl_auto_span_range! {
    syn::BinOp,
    syn::UnOp,
    syn::token::Underscore,
    syn::token::Dot,
    syn::token::DotDot,
    syn::token::DotDotEq,
    syn::token::Shl,
    syn::token::Shr,
    syn::token::AndAnd,
    syn::token::OrOr,
    syn::token::EqEq,
    syn::token::Lt,
    syn::token::Le,
    syn::token::Ne,
    syn::token::Ge,
    syn::token::Gt,
    syn::token::Comma,
    syn::token::PlusEq,
    syn::token::MinusEq,
    syn::token::StarEq,
    syn::token::SlashEq,
    syn::token::PercentEq,
    syn::token::AndEq,
    syn::token::OrEq,
    syn::token::CaretEq,
    syn::token::ShlEq,
    syn::token::ShrEq,
}

macro_rules! single_span_token {
    ($(Token![$token:tt]),* $(,)?) => {
        $(
            impl HasSpan for Token![$token] {
                fn span(&self) -> Span {
                    self.span
                }
            }
        )*
    };
}

single_span_token! {
    Token![as],
    Token![in],
    Token![=],
    Token![!],
    Token![~],
    Token![?],
    Token![+],
    Token![-],
    Token![*],
    Token![/],
    Token![%],
    Token![^],
    Token![&],
    Token![|],
}

pub(crate) struct Spanned<T> {
    pub(crate) value: T,
    pub(crate) span_range: SpanRange,
}

impl<T> Spanned<T> {
    pub(crate) fn deconstruct(self) -> (T, SpanRange) {
        (self.value, self.span_range)
    }

    #[allow(unused)]
    pub(crate) fn map<U>(self, f: impl FnOnce(T, &SpanRange) -> U) -> Spanned<U> {
        Spanned {
            value: f(self.value, &self.span_range),
            span_range: self.span_range,
        }
    }

    pub(crate) fn try_map<U, E>(
        self,
        f: impl FnOnce(T, &SpanRange) -> Result<U, E>,
    ) -> Result<Spanned<U>, E> {
        Ok(Spanned {
            value: f(self.value, &self.span_range)?,
            span_range: self.span_range,
        })
    }
}

impl<T> HasSpanRange for Spanned<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<X, T: AsRef<X>> AsRef<X> for Spanned<T> {
    fn as_ref(&self) -> &X {
        self.value.as_ref()
    }
}

impl<X, T: AsMut<X>> AsMut<X> for Spanned<T> {
    fn as_mut(&mut self) -> &mut X {
        self.value.as_mut()
    }
}

pub(crate) trait ToSpanned: Sized {
    fn spanned(self, source: impl HasSpanRange) -> Spanned<Self>;
}

impl<T> ToSpanned for T {
    fn spanned(self, source: impl HasSpanRange) -> Spanned<Self> {
        Spanned {
            value: self,
            span_range: source.span_range(),
        }
    }
}
