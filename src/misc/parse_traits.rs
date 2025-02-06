use std::ops::Deref;

use crate::internal_prelude::*;

// Parsing of source code tokens
// =============================

pub(crate) struct Source;

impl ParseBuffer<'_, Source> {
    pub(crate) fn peek_grammar(&self) -> SourcePeekMatch {
        detect_preinterpret_grammar(self.cursor())
    }
}

#[allow(unused)]
pub(crate) enum SourcePeekMatch {
    Command(Option<CommandOutputKind>),
    GroupedVariable,
    FlattenedVariable,
    AppendVariableBinding,
    ExplicitTransformStream,
    Transformer(Option<TransformerKind>),
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
    End,
}

fn detect_preinterpret_grammar(cursor: syn::buffer::Cursor) -> SourcePeekMatch {
    // We have to check groups first, so that we handle transparent groups
    // and avoid the self.ignore_none() calls inside cursor
    if let Some((next, delimiter, _, _)) = cursor.any_group() {
        if delimiter == Delimiter::Bracket {
            if let Some((_, next)) = next.punct_matching('!') {
                if let Some((ident, next)) = next.ident() {
                    if next.punct_matching('!').is_some() {
                        let output_kind =
                            CommandKind::for_ident(&ident).map(|kind| kind.standard_output_kind());
                        return SourcePeekMatch::Command(output_kind);
                    }
                }
                if let Some((first, next)) = next.punct_matching('.') {
                    if let Some((_, next)) = next.punct_matching('.') {
                        if let Some((ident, next)) = next.ident() {
                            if next.punct_matching('!').is_some() {
                                let output_kind = CommandKind::for_ident(&ident).and_then(|kind| {
                                    kind.flattened_output_kind(first.span_range()).ok()
                                });
                                return SourcePeekMatch::Command(output_kind);
                            }
                        }
                    }
                }
            }
        }

        // Ideally we'd like to detect $($tt)* substitutions from macros and interpret them as
        // a Raw (uninterpreted) group, because typically that's what a user would typically intend.
        //
        // You'd think mapping a Delimiter::None to a GrammarPeekMatch::RawGroup would be a good way
        // of doing this, but unfortunately this behaviour is very arbitrary and not in a helpful way:
        // => A $tt or $($tt)* is not grouped...
        // => A $literal or $($literal)* _is_ outputted in a group...
        //
        // So this isn't possible. It's unlikely to matter much, and a user can always do:
        // [!raw! $($tt)*] anyway.

        return SourcePeekMatch::Group(delimiter);
    }
    if let Some((_, next)) = cursor.punct_matching('#') {
        if next.ident().is_some() {
            return SourcePeekMatch::GroupedVariable;
        }
        if let Some((_, next)) = next.punct_matching('.') {
            if let Some((_, next)) = next.punct_matching('.') {
                if next.ident().is_some() {
                    return SourcePeekMatch::FlattenedVariable;
                }
                if let Some((_, next)) = next.punct_matching('>') {
                    if next.punct_matching('>').is_some() {
                        return SourcePeekMatch::AppendVariableBinding;
                    }
                }
            }
        }
        if let Some((_, next)) = next.punct_matching('>') {
            if next.punct_matching('>').is_some() {
                return SourcePeekMatch::AppendVariableBinding;
            }
        }
    }

    if let Some((_, next)) = cursor.punct_matching('@') {
        if let Some((_, _, _)) = next.group_matching(Delimiter::Parenthesis) {
            // @(...) or @(_ = ...) or @(#x = ...)
            return SourcePeekMatch::ExplicitTransformStream;
        }
        if let Some((ident, _)) = next.ident() {
            let name = ident.to_string();
            if name.to_uppercase() == name {
                return SourcePeekMatch::Transformer(TransformerKind::for_ident(&ident));
            }
        }
        if let Some((_, next, _)) = next.group_matching(Delimiter::Bracket) {
            if let Some((ident, _)) = next.ident() {
                let name = ident.to_string();
                if name.to_uppercase() == name {
                    return SourcePeekMatch::Transformer(TransformerKind::for_ident(&ident));
                }
            }
        }
    }

    match cursor.token_tree() {
        Some((TokenTree::Ident(ident), _)) => SourcePeekMatch::Ident(ident),
        Some((TokenTree::Punct(punct), _)) => SourcePeekMatch::Punct(punct),
        Some((TokenTree::Literal(literal), _)) => SourcePeekMatch::Literal(literal),
        Some((TokenTree::Group(_), _)) => unreachable!("Already covered above"),
        None => SourcePeekMatch::End,
    }
}

// Parsing of already interpreted tokens
// (e.g. transforming / destructuring)
// =====================================

pub(crate) struct Output;

impl ParseBuffer<'_, Output> {
    pub(crate) fn peek_grammar(&self) -> OutputPeekMatch {
        match self.cursor().token_tree() {
            Some((TokenTree::Ident(ident), _)) => OutputPeekMatch::Ident(ident),
            Some((TokenTree::Punct(punct), _)) => OutputPeekMatch::Punct(punct),
            Some((TokenTree::Literal(literal), _)) => OutputPeekMatch::Literal(literal),
            Some((TokenTree::Group(group), _)) => OutputPeekMatch::Group(group.delimiter()),
            None => OutputPeekMatch::End,
        }
    }
}

#[allow(unused)]
pub(crate) enum OutputPeekMatch {
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
    End,
}

// Generic parsing
// ===============

pub(crate) trait Parse<K>: Sized {
    fn parse(input: ParseStream<K>) -> ParseResult<Self>;
}

pub(crate) trait ContextualParse<K>: Sized {
    type Context;

    fn parse(input: ParseStream<K>, context: Self::Context) -> ParseResult<Self>;
}

impl<T: SynParse, K> Parse<K> for T {
    fn parse(input: ParseStream<K>) -> ParseResult<Self> {
        Ok(T::parse(&input.inner)?)
    }
}

pub(crate) type ParseStream<'a, K> = &'a ParseBuffer<'a, K>;

// We create our own ParseBuffer mostly so we can overwrite
// parse<T: Parse> to return ParseResult<T> instead of syn::Result<T>
#[repr(transparent)]
pub(crate) struct ParseBuffer<'a, K> {
    inner: SynParseBuffer<'a>,
    _kind: PhantomData<K>,
}

impl<'a, K> From<syn::parse::ParseBuffer<'a>> for ParseBuffer<'a, K> {
    fn from(inner: syn::parse::ParseBuffer<'a>) -> Self {
        Self {
            inner,
            _kind: PhantomData,
        }
    }
}

// This is From<&'a SynParseBuffer<'a>> for &'a ParseBuffer<'a>
impl<'a, K> From<SynParseStream<'a>> for ParseStream<'a, K> {
    fn from(syn_parse_stream: SynParseStream<'a>) -> Self {
        unsafe {
            // SAFETY: This is safe because [Syn]ParseStream<'a> = &'a [Syn]ParseBuffer<'a>
            // And ParseBuffer<'a> is marked as #[repr(transparent)] so has identical layout to SynParseBuffer<'a>
            // So this is a transmute between compound types with identical layouts which is safe.
            core::mem::transmute::<SynParseStream<'a>, ParseStream<'a, K>>(syn_parse_stream)
        }
    }
}

impl<'a, K> ParseBuffer<'a, K> {
    pub(crate) fn fork(&self) -> ParseBuffer<'a, K> {
        ParseBuffer {
            inner: self.inner.fork(),
            _kind: PhantomData,
        }
    }

    pub(crate) fn parse<T: Parse<K>>(&self) -> ParseResult<T> {
        T::parse(self)
    }

    pub(crate) fn parse_with_context<T: ContextualParse<K>>(
        &self,
        context: T::Context,
    ) -> ParseResult<T> {
        T::parse(self, context)
    }

    pub(crate) fn try_parse_or_error<
        T,
        F: FnOnce(&Self) -> ParseResult<T>,
        M: std::fmt::Display,
    >(
        &self,
        parse: F,
        message: M,
    ) -> ParseResult<T> {
        let error_span = self.span();
        parse(self).map_err(|_| error_span.error(message).into())
    }

    pub(crate) fn parse_any_punct(&self) -> ParseResult<Punct> {
        // Annoyingly, ' behaves weirdly in syn, so we need to handle it
        match self.inner.parse::<TokenTree>()? {
            TokenTree::Punct(punct) => Ok(punct),
            _ => self.span().parse_err("expected punctuation"),
        }
    }

    pub(crate) fn parse_any_ident(&self) -> ParseResult<Ident> {
        Ok(self.call(Ident::parse_any)?)
    }

    pub(crate) fn peek_ident_matching(&self, content: &str) -> bool {
        self.cursor().ident_matching(content).is_some()
    }

    pub(crate) fn parse_ident_matching(&self, content: &str) -> ParseResult<Ident> {
        Ok(self.step(|cursor| {
            cursor
                .ident_matching(content)
                .ok_or_else(|| cursor.span().error(format!("expected {}", content)))
        })?)
    }

    pub(crate) fn peek_punct_matching(&self, punct: char) -> bool {
        self.cursor().punct_matching(punct).is_some()
    }

    pub(crate) fn parse_punct_matching(&self, punct: char) -> ParseResult<Punct> {
        Ok(self.step(|cursor| {
            cursor
                .punct_matching(punct)
                .ok_or_else(|| cursor.span().error(format!("expected {}", punct)))
        })?)
    }

    pub(crate) fn peek_literal_matching(&self, content: &str) -> bool {
        self.cursor().literal_matching(content).is_some()
    }

    pub(crate) fn parse_literal_matching(&self, content: &str) -> ParseResult<Literal> {
        Ok(self.step(|cursor| {
            cursor
                .literal_matching(content)
                .ok_or_else(|| cursor.span().error(format!("expected {}", content)))
        })?)
    }

    pub(crate) fn parse_any_group(&self) -> ParseResult<(Delimiter, DelimSpan, ParseBuffer<K>)> {
        use syn::parse::discouraged::AnyDelimiter;
        let (delimiter, delim_span, parse_buffer) = self.parse_any_delimiter()?;
        Ok((delimiter, delim_span, parse_buffer.into()))
    }

    pub(crate) fn peek_specific_group(&self, delimiter: Delimiter) -> bool {
        self.cursor().group_matching(delimiter).is_some()
    }

    pub(crate) fn parse_group_matching(
        &self,
        matching: impl FnOnce(Delimiter) -> bool,
        expected_message: impl FnOnce() -> String,
    ) -> ParseResult<(DelimSpan, ParseBuffer<K>)> {
        let error_span = match self.parse_any_group() {
            Ok((delimiter, delim_span, inner)) if matching(delimiter) => {
                return Ok((delim_span, inner));
            }
            Ok((_, delim_span, _)) => delim_span.open(),
            Err(error) => error.span(),
        };
        error_span.parse_err(expected_message())
    }

    pub(crate) fn parse_specific_group(
        &self,
        expected_delimiter: Delimiter,
    ) -> ParseResult<(DelimSpan, ParseBuffer<K>)> {
        self.parse_group_matching(
            |delimiter| delimiter == expected_delimiter,
            || format!("Expected {}", expected_delimiter.description_of_open()),
        )
    }

    pub(crate) fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        Err(self.parse_error(message))
    }

    pub(crate) fn parse_error(&self, message: impl std::fmt::Display) -> ParseError {
        self.span().parse_error(message)
    }
}

impl<'a, K> Deref for ParseBuffer<'a, K> {
    type Target = SynParseBuffer<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
