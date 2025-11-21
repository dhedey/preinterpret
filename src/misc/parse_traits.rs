#![allow(unused)]
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
    EmbeddedExpression,
    EmbeddedStatements,
    EmbeddedVariable,
    ExplicitTransformStream,
    Transformer(Option<TransformerKind>),
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
    StreamLiteral(StreamLiteralKind),
    ObjectLiteral,
    End,
}

fn detect_preinterpret_grammar(cursor: syn::buffer::Cursor) -> SourcePeekMatch {
    if let Some((_, next)) = cursor.punct_matching('#') {
        if next.ident().is_some() {
            return SourcePeekMatch::EmbeddedVariable;
        }
        if next.group_matching(Delimiter::Parenthesis).is_some() {
            return SourcePeekMatch::EmbeddedExpression;
        }
        if next.group_matching(Delimiter::Brace).is_some() {
            return SourcePeekMatch::EmbeddedStatements;
        }
    }

    if let Some((_, next)) = cursor.punct_matching('%') {
        if next.group_matching(Delimiter::Bracket).is_some() {
            return SourcePeekMatch::StreamLiteral(StreamLiteralKind::Regular);
        }
        if let Some((ident, _)) = next.ident() {
            let ident_string = ident.to_string();
            match ident_string.as_str() {
                "raw" => return SourcePeekMatch::StreamLiteral(StreamLiteralKind::Raw),
                "group" => return SourcePeekMatch::StreamLiteral(StreamLiteralKind::Grouped),
                _ => {}
            }
        }
        if next.group_matching(Delimiter::Brace).is_some() {
            return SourcePeekMatch::ObjectLiteral;
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

    if let Some((next, delimiter, _, _)) = cursor.any_group() {
        // Ideally we'd like to detect $($tt)* substitutions from macros and interpret them as
        // a Raw (uninterpreted) group, because typically that's what a user would typically intend.
        //
        // You'd think mapping a Delimiter::None to a GrammarPeekMatch::RawGroup would be a good way
        // of doing this, but unfortunately this behaviour is very arbitrary and not in a helpful way:
        // => A $tt or $($tt)* is not grouped...
        // => A $literal or $($literal)* _is_ outputted in a group...
        //
        // So this isn't possible. It's unlikely to matter much, and a user can always do:
        // %raw[$($tt)*] anyway.
        return SourcePeekMatch::Group(delimiter);
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

// Source parsing
// ===============

/// This trait is slightly more restrict/powerful than Parse<Source>
/// as it gives access to the parse context.
pub(crate) trait ParseSource: Sized {
    fn parse(input: SourceParser) -> ParseResult<Self>;

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()>;
}

impl<T> ParseSource for T
where
    T: Parse<Source>,
{
    fn parse(input: SourceParser) -> ParseResult<Self> {
        <T as Parse<Source>>::parse(&input.buffer)
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

pub(crate) trait ParseSourceOptional: ParseSource {
    /// This may want to  be overwritten to have a peek/commit behaviour
    /// to give clearer errors.
    fn parse_optional(input: SourceParser) -> ParseResult<Option<Self>> {
        let fork = input.fork();
        match Self::parse(&fork) {
            Ok(value) => {
                input.advance_to(&fork.buffer);
                Ok(Some(value))
            }
            Err(_) => Ok(None),
        }
    }
}

pub(crate) fn parse_without_analysis<T>(
    parser: impl FnOnce(SourceParser) -> ParseResult<T>,
) -> impl FnOnce(ParseStream<Source>) -> ParseResult<T> {
    move |stream: ParseStream<Source>| {
        // To get access to an owned ParseBuffer we fork it... and advance later!
        let forked = stream.fork();
        let parse_buffer = SourceParseBuffer::new(forked);
        let mut output = parser(&parse_buffer)?;
        stream.advance_to(&parse_buffer.buffer);
        Ok(output)
    }
}

pub(crate) type SourceParser<'a> = &'a SourceParseBuffer<'a>;
pub(crate) type FlowCapturer<'a> = &'a mut ControlFlowContext;

pub(crate) struct ControlFlowContext {
    state: FlowAnalysisState,
}

impl ControlFlowContext {
    pub(crate) fn analyze<T>(
        parsed: &mut T,
        inner: impl FnOnce(&mut T, FlowCapturer) -> ParseResult<()>,
    ) -> ParseResult<ScopeDefinitions> {
        let mut context = Self {
            state: FlowAnalysisState::new(),
        };
        inner(parsed, &mut context)?;
        context.state.finish()
    }

    pub(crate) fn register_scope(&mut self, id: &mut ScopeId) {
        assert!(id.is_placeholder());
        *id = self.state.allocate_scope();
    }

    pub(crate) fn register_variable_definition(
        &mut self,
        ident: &Ident,
        id: &mut VariableDefinitionId,
    ) {
        assert!(id.is_placeholder());
        *id = self.state.allocate_variable_definition(ident);
    }

    pub(crate) fn register_variable_reference(
        &mut self,
        ident: &Ident,
        id: &mut VariableReferenceId,
    ) {
        assert!(id.is_placeholder());
        *id = self.state.allocate_variable_reference(ident);
    }

    pub(crate) fn enter_scope(&mut self, scope: ScopeId) {
        self.state.enter_scope(scope);
    }

    pub(crate) fn exit_scope(&mut self, scope: ScopeId) {
        self.state.exit_scope(scope);
    }

    pub(crate) fn define_variable(&mut self, id: VariableDefinitionId) {
        self.state.define_variable(id);
    }

    pub(crate) fn reference_variable(
        &mut self,
        id: VariableReferenceId,
        #[cfg(feature = "debug")] assertion: FinalUseAssertion,
    ) -> ParseResult<()> {
        self.state.reference_variable(
            id,
            #[cfg(feature = "debug")]
            assertion,
        )
    }

    pub(crate) fn enter_next_segment(&mut self, segment_kind: SegmentKind) -> ControlFlowSegmentId {
        self.state.enter_next_segment(segment_kind)
    }

    pub(crate) fn enter_path_segment(
        &mut self,
        previous_sibling_id: Option<ControlFlowSegmentId>,
        segment_kind: SegmentKind,
    ) -> ControlFlowSegmentId {
        self.state
            .enter_path_segment(previous_sibling_id, segment_kind)
    }

    pub(crate) fn exit_segment(&mut self, segment_id: ControlFlowSegmentId) {
        self.state.exit_segment(segment_id);
    }

    pub(crate) fn register_catch_location(&mut self, kind: CatchLocationKind) -> CatchLocationId {
        self.state.register_catch_location(kind)
    }

    pub(crate) fn register_labeled_catch_location(
        &mut self,
        label: &str,
        location_id: CatchLocationId,
    ) {
        self.state
            .register_labeled_catch_location(label, location_id);
    }

    pub(crate) fn resolve_label_to_catch_location(&self, label: &str) -> Option<CatchLocationId> {
        self.state.resolve_label_to_catch_location(label)
    }

    /// Helper method to register a catch location and optionally assign it to a label.
    /// This reduces boilerplate when loops/blocks need to register catch locations.
    pub(crate) fn register_catch_location_with_optional_label(
        &mut self,
        label: Option<&mut CatchLabel>,
        kind: CatchLocationKind,
    ) -> CatchLocationId {
        let id = self.register_catch_location(kind);
        if let Some(label) = label {
            label.catch_location_id = id;
            self.register_labeled_catch_location(&label.ident_string(), id);
        }
        id
    }

    pub(crate) fn enter_loop(&mut self, catch_location_id: CatchLocationId) {
        self.state.enter_loop(catch_location_id);
    }

    pub(crate) fn exit_loop(&mut self, catch_location_id: CatchLocationId) {
        self.state.exit_loop(catch_location_id);
    }

    pub(crate) fn current_loop_catch_location(&self) -> Option<CatchLocationId> {
        self.state.current_loop_catch_location()
    }
}

// This was originally created so that we could modify a stateful context
// during parsing, but this was later moved to the control_flow pass instead.
// We might be able to remove this and go back to ParseBuffer<'a, Source> in future.
pub(crate) struct SourceParseBuffer<'a> {
    pub(crate) buffer: ParseBuffer<'a, Source>,
}

impl<'a> Deref for SourceParseBuffer<'a> {
    type Target = ParseBuffer<'a, Source>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a> SourceParseBuffer<'a> {
    fn new(buffer: ParseBuffer<'a, Source>) -> Self {
        Self { buffer }
    }

    pub(crate) fn fork(&self) -> SourceParseBuffer<'a> {
        SourceParseBuffer {
            buffer: self.buffer.fork(),
        }
    }

    pub(crate) fn parse_virtual_empty_stream<T>(
        &self,
        parser: impl FnOnce(SourceParser) -> ParseResult<T>,
    ) -> ParseResult<T> {
        parse_with(TokenStream::new(), |stream| -> ParseResult<T> {
            let forked = stream.fork();
            let parse_buffer = SourceParseBuffer { buffer: forked };
            let output = parser(&parse_buffer)?;
            stream.advance_to(&parse_buffer.buffer);
            Ok(output)
        })
    }

    fn child_from_buffer<'c>(&self, buffer: ParseBuffer<'c, Source>) -> SourceParseBuffer<'c> {
        SourceParseBuffer { buffer }
    }

    pub(crate) fn parse<T: ParseSource>(&self) -> ParseResult<T> {
        T::parse(self)
    }

    pub(crate) fn parse_optional<T: ParseSourceOptional>(&self) -> ParseResult<Option<T>> {
        T::parse_optional(self)
    }

    pub fn parse_terminated<T: ParseSource, P: ParseSource>(
        &'a self,
    ) -> ParseResult<Punctuated<T, P>> {
        Punctuated::parse_terminated_using(self, T::parse, P::parse)
    }

    pub(crate) fn call<T, F: FnOnce(SourceParser) -> ParseResult<T>>(
        &self,
        f: F,
    ) -> ParseResult<T> {
        f(self)
    }

    pub(crate) fn parse_any_group(
        &self,
    ) -> ParseResult<(Delimiter, DelimSpan, SourceParseBuffer<'_>)> {
        self.buffer
            .parse_any_group()
            .map(move |(d, s, b)| (d, s, self.child_from_buffer(b)))
    }

    pub(crate) fn parse_group_matching(
        &self,
        matching: impl FnOnce(Delimiter) -> bool,
        expected_message: impl FnOnce() -> String,
    ) -> ParseResult<(DelimSpan, SourceParseBuffer<'_>)> {
        self.buffer
            .parse_group_matching(matching, expected_message)
            .map(|(s, b)| (s, self.child_from_buffer(b)))
    }

    pub(crate) fn parse_specific_group(
        &self,
        expected_delimiter: Delimiter,
    ) -> ParseResult<(DelimSpan, SourceParseBuffer<'_>)> {
        self.parse_group_matching(
            |delimiter| delimiter == expected_delimiter,
            || format!("Expected {}", expected_delimiter.description_of_open()),
        )
    }

    pub(crate) fn parse_braces(&self) -> ParseResult<(Braces, SourceParseBuffer<'_>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::Brace)?;
        Ok((Braces { delim_span }, inner))
    }

    pub(crate) fn parse_brackets(&self) -> ParseResult<(Brackets, SourceParseBuffer<'_>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::Bracket)?;
        Ok((Brackets { delim_span }, inner))
    }

    pub(crate) fn parse_parentheses(&self) -> ParseResult<(Parentheses, SourceParseBuffer<'_>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::Parenthesis)?;
        Ok((Parentheses { delim_span }, inner))
    }

    pub(crate) fn parse_transparent_group(
        &self,
    ) -> ParseResult<(TransparentDelimiters, SourceParseBuffer<'_>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::None)?;
        Ok((TransparentDelimiters { delim_span }, inner))
    }
}

// Generic parsing
// ===============

pub(crate) trait Parse<K>: Sized {
    fn parse(input: ParseStream<K>) -> ParseResult<Self>;
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

    pub fn parse_terminated<T: Parse<K>, P: Parse<K>>(&'a self) -> ParseResult<Punctuated<T, P>> {
        Punctuated::parse_terminated_using(self, T::parse, P::parse)
    }

    pub(crate) fn call<T, F: FnOnce(ParseStream<K>) -> ParseResult<T>>(
        &self,
        f: F,
    ) -> ParseResult<T> {
        f(self)
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
        parse(self).map_err(|_| error_span.parse_error(message))
    }

    pub(crate) fn parse_any_punct(&self) -> ParseResult<Punct> {
        // Annoyingly, ' doesn't count as a `Punct` in syn
        // So to make sure we can capture it, we handle it as parsing TokenTree rather than a Punct
        match self.inner.parse::<TokenTree>()? {
            TokenTree::Punct(punct) => Ok(punct),
            _ => self.span().parse_err("expected punctuation"),
        }
    }

    pub(crate) fn parse_any_ident(&self) -> ParseResult<Ident> {
        self.call(|stream| Ok(Ident::parse_any(&stream.inner)?))
    }

    pub(crate) fn peek_ident_matching(&self, content: &str) -> bool {
        self.cursor().ident_matching(content).is_some()
    }

    pub(crate) fn parse_ident_matching(&self, content: &str) -> ParseResult<Ident> {
        Ok(self.inner.step(|cursor| {
            cursor
                .ident_matching(content)
                .ok_or_else(|| cursor.syn_error(format!("expected {}", content)))
        })?)
    }

    pub(crate) fn peek_punct_matching(&self, punct: char) -> bool {
        self.cursor().punct_matching(punct).is_some()
    }

    pub(crate) fn parse_punct_matching(&self, punct: char) -> ParseResult<Punct> {
        Ok(self.inner.step(|cursor| {
            cursor
                .punct_matching(punct)
                .ok_or_else(|| cursor.syn_error(format!("expected {}", punct)))
        })?)
    }

    pub(crate) fn peek_literal_matching(&self, content: &str) -> bool {
        self.cursor().literal_matching(content).is_some()
    }

    pub(crate) fn parse_literal_matching(&self, content: &str) -> ParseResult<Literal> {
        Ok(self.inner.step(|cursor| {
            cursor
                .literal_matching(content)
                .ok_or_else(|| cursor.syn_error(format!("expected {}", content)))
        })?)
    }

    pub(crate) fn parse_any_group(
        &self,
    ) -> ParseResult<(Delimiter, DelimSpan, ParseBuffer<'_, K>)> {
        use syn::parse::discouraged::AnyDelimiter;
        let (delimiter, delim_span, parse_buffer) = self.inner.parse_any_delimiter()?;
        Ok((delimiter, delim_span, parse_buffer.into()))
    }

    pub(crate) fn peek_specific_group(&self, delimiter: Delimiter) -> bool {
        self.cursor().group_matching(delimiter).is_some()
    }

    pub(crate) fn parse_group_matching(
        &self,
        matching: impl FnOnce(Delimiter) -> bool,
        expected_message: impl FnOnce() -> String,
    ) -> ParseResult<(DelimSpan, ParseBuffer<'_, K>)> {
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
    ) -> ParseResult<(DelimSpan, ParseBuffer<'_, K>)> {
        self.parse_group_matching(
            |delimiter| delimiter == expected_delimiter,
            || format!("Expected {}", expected_delimiter.description_of_open()),
        )
    }

    pub(crate) fn parse_braces(&self) -> ParseResult<(Braces, ParseBuffer<'_, K>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::Brace)?;
        Ok((Braces { delim_span }, inner))
    }

    pub(crate) fn parse_brackets(&self) -> ParseResult<(Brackets, ParseBuffer<'_, K>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::Bracket)?;
        Ok((Brackets { delim_span }, inner))
    }

    pub(crate) fn parse_parentheses(&self) -> ParseResult<(Parentheses, ParseBuffer<'_, K>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::Parenthesis)?;
        Ok((Parentheses { delim_span }, inner))
    }

    pub(crate) fn parse_transparent_group(
        &self,
    ) -> ParseResult<(TransparentDelimiters, ParseBuffer<'_, K>)> {
        let (delim_span, inner) = self.parse_specific_group(Delimiter::None)?;
        Ok((TransparentDelimiters { delim_span }, inner))
    }

    pub(crate) fn advance_to(&self, fork: &Self) {
        self.inner.advance_to(&fork.inner)
    }

    // ERRORS
    // ======

    pub(crate) fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        Err(self.parse_error(message))
    }

    pub(crate) fn parse_error(&self, message: impl std::fmt::Display) -> ParseError {
        self.span().parse_error(message)
    }

    // Pass-throughs to SynParseBuffer
    // ===============================

    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub(crate) fn cursor(&self) -> syn::buffer::Cursor<'a> {
        self.inner.cursor()
    }

    pub(crate) fn span(&self) -> Span {
        self.inner.span()
    }

    pub(crate) fn peek(&self, token: impl syn::parse::Peek) -> bool {
        self.inner.peek(token)
    }

    pub(crate) fn peek2(&self, token: impl syn::parse::Peek) -> bool {
        self.inner.peek2(token)
    }

    #[allow(unused)]
    pub(crate) fn peek3(&self, token: impl syn::parse::Peek) -> bool {
        self.inner.peek3(token)
    }

    /// End with `Err(lookahead.error())?`
    pub(crate) fn lookahead1(&self) -> syn::parse::Lookahead1<'a> {
        self.inner.lookahead1()
    }
}

// Punctuated extensions
// =======================

pub(crate) trait PunctuatedExtensions<T, P>: Sized {
    fn parse_terminated_using<I: AnyParseStream>(
        input: I,
        value_parser: impl Fn(I) -> ParseResult<T>,
        punct_parser: impl Fn(I) -> ParseResult<P>,
    ) -> ParseResult<Self>;
}

impl<T, P> PunctuatedExtensions<T, P> for Punctuated<T, P> {
    // More flexible than syn's built-in parse_terminated_with
    fn parse_terminated_using<I: AnyParseStream>(
        input: I,
        value_parser: impl Fn(I) -> ParseResult<T>,
        punct_parser: impl Fn(I) -> ParseResult<P>,
    ) -> ParseResult<Self> {
        let mut punctuated = Punctuated::new();

        loop {
            if input.is_empty() {
                break;
            }
            let value = value_parser(input)?;
            punctuated.push_value(value);
            if input.is_empty() {
                break;
            }
            let punct = punct_parser(input)?;
            punctuated.push_punct(punct);
        }

        Ok(punctuated)
    }
}

pub(crate) trait AnyParseStream: Copy {
    fn is_empty(&self) -> bool;
}

impl<'a, K> AnyParseStream for ParseStream<'a, K> {
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<'a> AnyParseStream for SourceParser<'a> {
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// This allows parsing of a type, but discarding the result.
///
/// This is useful for keywords or syntax which we'd like to parse
/// in a uniform way, but don't need to keep around bloating the
/// size of our types.
pub(crate) struct Unused<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: ParseSource> ParseSource for Unused<T> {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let _ = input.parse::<T>()?;
        Ok(Self {
            _marker: std::marker::PhantomData,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        unreachable!("An unused value should not have a control flow pass")
    }
}
