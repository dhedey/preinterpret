use std::ops::Deref;

use crate::internal_prelude::*;

pub(crate) trait ContextualParse: Sized {
    type Context;

    fn parse_with_context(input: ParseStream, context: Self::Context) -> ParseResult<Self>;
}

pub(crate) trait Parse: Sized {
    fn parse(input: ParseStream) -> ParseResult<Self>;
}

impl<T: SynParse> Parse for T {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(T::parse(&input.inner)?)
    }
}

pub(crate) type ParseStream<'a> = &'a ParseBuffer<'a>;

// We create our own ParseBuffer mostly so we can overwrite
// parse<T: Parse> to return ParseResult<T> instead of syn::Result<T>
#[repr(transparent)]
pub(crate) struct ParseBuffer<'a> {
    inner: SynParseBuffer<'a>,
}

impl<'a> From<syn::parse::ParseBuffer<'a>> for ParseBuffer<'a> {
    fn from(inner: syn::parse::ParseBuffer<'a>) -> Self {
        Self { inner }
    }
}

// This is From<&'a SynParseBuffer<'a>> for &'a ParseBuffer<'a>
impl<'a> From<SynParseStream<'a>> for ParseStream<'a> {
    fn from(syn_parse_stream: SynParseStream<'a>) -> Self {
        unsafe {
            // SAFETY: This is safe because [Syn]ParseStream<'a> = &'a [Syn]ParseBuffer<'a>
            // And ParseBuffer<'a> is marked as #[repr(transparent)] so has identical layout to SynParseBuffer<'a>
            // So this is a transmute between compound types with identical layouts which is safe.
            core::mem::transmute::<SynParseStream<'a>, ParseStream<'a>>(syn_parse_stream)
        }
    }
}

impl<'a> ParseBuffer<'a> {
    // Methods on SynParseBuffer are available courtesy of Deref below
    // But the following methods are replaced, for ease of use:

    pub(crate) fn parse<T: Parse>(&self) -> ParseResult<T> {
        T::parse(self)
    }

    pub(crate) fn fork(&self) -> ParseBuffer<'a> {
        ParseBuffer {
            inner: self.inner.fork(),
        }
    }
}

impl<'a> Deref for ParseBuffer<'a> {
    type Target = SynParseBuffer<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
