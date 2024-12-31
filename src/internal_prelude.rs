pub(crate) use core::iter;
use extra::DelimSpan;
pub(crate) use proc_macro2::*;
pub(crate) use quote::ToTokens;
pub(crate) use std::{collections::HashMap, str::FromStr};
pub(crate) use syn::{parse_str, Expr, ExprLit, Lit, LitBool, LitFloat, LitInt, Result, UnOp};

pub(crate) use crate::command::*;
pub(crate) use crate::commands::*;
pub(crate) use crate::expressions::*;
pub(crate) use crate::interpreter::*;
pub(crate) use crate::string_conversion::*;

pub(crate) trait TokenStreamExt: Sized {
    fn push_token_tree(&mut self, token_tree: TokenTree);
    fn push_new_group(
        &mut self,
        span_range: SpanRange,
        delimiter: Delimiter,
        inner_tokens: TokenStream,
    );
}

impl TokenStreamExt for TokenStream {
    fn push_token_tree(&mut self, token_tree: TokenTree) {
        self.extend(iter::once(token_tree));
    }

    fn push_new_group(
        &mut self,
        span_range: SpanRange,
        delimiter: Delimiter,
        inner_tokens: TokenStream,
    ) {
        let group = Group::new(delimiter, inner_tokens).with_span(span_range.span());
        self.push_token_tree(TokenTree::Group(group));
    }
}

pub(crate) trait SpanErrorExt: Sized {
    fn err<T>(self, message: impl std::fmt::Display) -> syn::Result<T> {
        Err(self.error(message))
    }

    fn error(self, message: impl std::fmt::Display) -> syn::Error;
}

impl SpanErrorExt for Span {
    fn error(self, message: impl std::fmt::Display) -> syn::Error {
        syn::Error::new(self, message)
    }
}

impl SpanErrorExt for SpanRange {
    fn error(self, message: impl std::fmt::Display) -> syn::Error {
        syn::Error::new_spanned(self, message)
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

#[allow(unused)]
impl SpanRange {
    pub(crate) fn new_between(start: Span, end: Span) -> Self {
        Self { start, end }
    }

    /// * On nightly, this gives a span covering the full range (the same result as `Span::join` would)
    /// * On stable, this gives the span of the first token of the group (because [`proc_macro::Span::join`] is not supported)
    pub(crate) fn span(&self) -> Span {
        <Self as syn::spanned::Spanned>::span(self)
    }

    pub(crate) fn start(&self) -> Span {
        self.start
    }

    pub(crate) fn end(&self) -> Span {
        self.end
    }

    pub(crate) fn replace_start(self, start: Span) -> Self {
        Self { start, ..self }
    }

    pub(crate) fn replace_end(self, end: Span) -> Self {
        Self { end, ..self }
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
        SpanRange::new_between(self.open(), self.close())
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
trait AutoSpanRange {}

macro_rules! impl_auto_span_range {
    ($($ty:ty),* $(,)?) => {
        $(
            impl AutoSpanRange for $ty {}
        )*
    };
}

impl_auto_span_range! {
    syn::Expr,
    syn::ExprBinary,
    syn::ExprUnary,
    syn::BinOp,
    syn::UnOp,
    syn::Type,
    syn::TypePath,
}
