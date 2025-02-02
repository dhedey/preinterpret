use std::ops::Deref;

use crate::internal_prelude::*;

// Parsing of source code tokens
// =============================

pub(crate) trait ContextualParseFromSource: Sized {
    type Context;

    fn parse_from_source(input: SourceParseStream, context: Self::Context) -> ParseResult<Self>;
}

pub(crate) trait ParseFromSource: Sized {
    fn parse_from_source(input: SourceParseStream) -> ParseResult<Self>;
}

impl<T: SynParse> ParseFromSource for T {
    fn parse_from_source(input: SourceParseStream) -> ParseResult<Self> {
        Ok(T::parse(&input.inner)?)
    }
}

impl<T: ParseFromSource> Parse<Source> for T {
    fn parse(input: SourceParseStream) -> ParseResult<Self> {
        T::parse_from_source(input)
    }
}

pub(crate) struct Source;
pub(crate) type SourceParseStream<'a> = &'a SourceParseBuffer<'a>;
pub(crate) type SourceParseBuffer<'a> = KindedParseBuffer<'a, Source>;

impl SourceParseBuffer<'_> {
    pub(crate) fn peek_grammar(&self) -> GrammarPeekMatch {
        detect_preinterpret_grammar(self.cursor())
    }
}

#[allow(unused)]
pub(crate) enum GrammarPeekMatch {
    Command(Option<CommandOutputKind>),
    GroupedVariable,
    FlattenedVariable,
    AppendVariableDestructuring,
    Destructurer(Option<DestructurerKind>),
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
    End,
}

fn detect_preinterpret_grammar(cursor: syn::buffer::Cursor) -> GrammarPeekMatch {
    // We have to check groups first, so that we handle transparent groups
    // and avoid the self.ignore_none() calls inside cursor
    if let Some((next, delimiter, _, _)) = cursor.any_group() {
        if delimiter == Delimiter::Bracket {
            if let Some((_, next)) = next.punct_matching('!') {
                if let Some((ident, next)) = next.ident() {
                    if next.punct_matching('!').is_some() {
                        let output_kind =
                            CommandKind::for_ident(&ident).map(|kind| kind.standard_output_kind());
                        return GrammarPeekMatch::Command(output_kind);
                    }
                }
                if let Some((first, next)) = next.punct_matching('.') {
                    if let Some((_, next)) = next.punct_matching('.') {
                        if let Some((ident, next)) = next.ident() {
                            if next.punct_matching('!').is_some() {
                                let output_kind = CommandKind::for_ident(&ident).and_then(|kind| {
                                    kind.flattened_output_kind(first.span_range()).ok()
                                });
                                return GrammarPeekMatch::Command(output_kind);
                            }
                        }
                    }
                }
            }
        }
        if delimiter == Delimiter::Parenthesis {
            if let Some((_, next)) = next.punct_matching('!') {
                if let Some((ident, next)) = next.ident() {
                    if next.punct_matching('!').is_some() {
                        return GrammarPeekMatch::Destructurer(DestructurerKind::for_ident(&ident));
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

        return GrammarPeekMatch::Group(delimiter);
    }
    if let Some((_, next)) = cursor.punct_matching('#') {
        if next.ident().is_some() {
            return GrammarPeekMatch::GroupedVariable;
        }
        if let Some((_, next)) = next.punct_matching('.') {
            if let Some((_, next)) = next.punct_matching('.') {
                if next.ident().is_some() {
                    return GrammarPeekMatch::FlattenedVariable;
                }
                if let Some((_, next)) = next.punct_matching('>') {
                    if next.punct_matching('>').is_some() {
                        return GrammarPeekMatch::AppendVariableDestructuring;
                    }
                }
            }
        }
        if let Some((_, next)) = next.punct_matching('>') {
            if next.punct_matching('>').is_some() {
                return GrammarPeekMatch::AppendVariableDestructuring;
            }
        }
    }

    match cursor.token_tree() {
        Some((TokenTree::Ident(ident), _)) => GrammarPeekMatch::Ident(ident),
        Some((TokenTree::Punct(punct), _)) => GrammarPeekMatch::Punct(punct),
        Some((TokenTree::Literal(literal), _)) => GrammarPeekMatch::Literal(literal),
        Some((TokenTree::Group(_), _)) => unreachable!("Already covered above"),
        None => GrammarPeekMatch::End,
    }
}

// Parsing of already interpreted tokens
// (e.g. destructuring)
// =====================================

pub(crate) trait ParseFromInterpreted: Sized {
    fn parse_from_interpreted(input: InterpretedParseStream) -> ParseResult<Self>;
}

impl<T: SynParse> ParseFromInterpreted for T {
    fn parse_from_interpreted(input: InterpretedParseStream) -> ParseResult<Self> {
        Ok(T::parse(&input.inner)?)
    }
}

impl<T: ParseFromInterpreted> Parse<Interpreted> for T {
    fn parse(input: InterpretedParseStream) -> ParseResult<Self> {
        T::parse_from_interpreted(input)
    }
}

pub(crate) struct Interpreted;
pub(crate) type InterpretedParseStream<'a> = &'a InterpretedParseBuffer<'a>;
pub(crate) type InterpretedParseBuffer<'a> = KindedParseBuffer<'a, Interpreted>;

impl InterpretedParseBuffer<'_> {
    pub(crate) fn peek_token(&self) -> InterpretedPeekMatch {
        match self.cursor().token_tree() {
            Some((TokenTree::Ident(ident), _)) => InterpretedPeekMatch::Ident(ident),
            Some((TokenTree::Punct(punct), _)) => InterpretedPeekMatch::Punct(punct),
            Some((TokenTree::Literal(literal), _)) => InterpretedPeekMatch::Literal(literal),
            Some((TokenTree::Group(group), _)) => InterpretedPeekMatch::Group(group.delimiter()),
            None => InterpretedPeekMatch::End,
        }
    }
}

#[allow(unused)]
pub(crate) enum InterpretedPeekMatch {
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
    End,
}

// Generic parsing
// ===============

pub(crate) trait Parse<K>: Sized {
    fn parse(input: KindedParseStream<K>) -> ParseResult<Self>;
}

pub(crate) type KindedParseStream<'a, K> = &'a KindedParseBuffer<'a, K>;

// We create our own ParseBuffer mostly so we can overwrite
// parse<T: Parse> to return ParseResult<T> instead of syn::Result<T>
#[repr(transparent)]
pub(crate) struct KindedParseBuffer<'a, K> {
    inner: SynParseBuffer<'a>,
    _kind: PhantomData<K>,
}

impl<'a, K> From<syn::parse::ParseBuffer<'a>> for KindedParseBuffer<'a, K> {
    fn from(inner: syn::parse::ParseBuffer<'a>) -> Self {
        Self {
            inner,
            _kind: PhantomData,
        }
    }
}

// This is From<&'a SynParseBuffer<'a>> for &'a ParseBuffer<'a>
impl<'a, K> From<SynParseStream<'a>> for KindedParseStream<'a, K> {
    fn from(syn_parse_stream: SynParseStream<'a>) -> Self {
        unsafe {
            // SAFETY: This is safe because [Syn]ParseStream<'a> = &'a [Syn]ParseBuffer<'a>
            // And ParseBuffer<'a> is marked as #[repr(transparent)] so has identical layout to SynParseBuffer<'a>
            // So this is a transmute between compound types with identical layouts which is safe.
            core::mem::transmute::<SynParseStream<'a>, KindedParseStream<'a, K>>(syn_parse_stream)
        }
    }
}

impl<'a, K> KindedParseBuffer<'a, K> {
    pub(crate) fn fork(&self) -> KindedParseBuffer<'a, K> {
        KindedParseBuffer {
            inner: self.inner.fork(),
            _kind: PhantomData,
        }
    }

    pub(crate) fn parse<T: Parse<K>>(&self) -> ParseResult<T> {
        T::parse(self)
    }

    pub(crate) fn parse_any_punct(&self) -> ParseResult<Punct> {
        // Annoyingly, ' behaves weirdly in syn, so we need to handle it
        match self.inner.parse::<TokenTree>()? {
            TokenTree::Punct(punct) => Ok(punct),
            _ => self.span().parse_err("expected punctuation"),
        }
    }
}

impl<'a, K> Deref for KindedParseBuffer<'a, K> {
    type Target = SynParseBuffer<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
