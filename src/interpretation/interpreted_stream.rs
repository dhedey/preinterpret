use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct InterpretedStream {
    /// Currently, even ~inside~ macro executions, round-tripping [`Delimiter::None`] groups to a TokenStream
    /// breaks in rust-analyzer. This causes various spurious errors in the IDE.
    /// See: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    ///
    /// So instead of just storing a TokenStream here, we store a list of TokenStreams and interpreted groups.
    /// This ensures any non-delimited groups are not round-tripped to a TokenStream.
    ///
    /// In future, we may wish to internally use some kind of `TokenBuffer` type, which I've started on below.
    segments: Vec<InterpretedSegment>,
    token_length: usize,
}

#[derive(Clone)]
// This was primarily implemented to avoid this issue: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
// But it doesn't actually help because the `syn::parse` mechanism only operates on a TokenStream,
// so we have to convert back into a TokenStream.
enum InterpretedSegment {
    TokenVec(Vec<TokenTree>), // Cheaper than a TokenStream (probably)
    InterpretedGroup(Delimiter, Span, InterpretedStream),
}

pub(crate) enum InterpretedTokenTree {
    TokenTree(TokenTree),
    InterpretedGroup(Delimiter, Span, InterpretedStream),
}

impl From<InterpretedTokenTree> for InterpretedStream {
    fn from(value: InterpretedTokenTree) -> Self {
        let mut new = Self::new();
        new.push_interpreted_item(value);
        new
    }
}

impl InterpretedStream {
    pub(crate) fn new() -> Self {
        Self {
            segments: vec![],
            token_length: 0,
        }
    }

    pub(crate) fn raw(token_stream: TokenStream) -> Self {
        let mut new = Self::new();
        new.extend_raw_tokens(token_stream);
        new
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
        appender: impl FnOnce(&mut Self) -> ExecutionResult<()>,
        delimiter: Delimiter,
        span: Span,
    ) -> ExecutionResult<()> {
        let mut inner = Self::new();
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
        self.segments.push(InterpretedSegment::InterpretedGroup(
            delimiter,
            span,
            inner_tokens,
        ));
        self.token_length += 1;
    }

    pub(crate) fn push_interpreted_item(&mut self, segment_item: InterpretedTokenTree) {
        match segment_item {
            InterpretedTokenTree::TokenTree(token_tree) => {
                self.push_raw_token_tree(token_tree);
            }
            InterpretedTokenTree::InterpretedGroup(delimiter, span, inner_tokens) => {
                self.push_new_group(inner_tokens, delimiter, span);
            }
        }
    }

    pub(crate) fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
        self.extend_raw_tokens(iter::once(token_tree));
    }

    pub(crate) fn extend_raw_tokens(&mut self, tokens: impl IntoIterator<Item = TokenTree>) {
        if !matches!(self.segments.last(), Some(InterpretedSegment::TokenVec(_))) {
            self.segments.push(InterpretedSegment::TokenVec(vec![]));
        }

        match self.segments.last_mut() {
            Some(InterpretedSegment::TokenVec(token_vec)) => {
                let before_length = token_vec.len();
                token_vec.extend(tokens);
                self.token_length += token_vec.len() - before_length;
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.token_length
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.token_length == 0
    }

    /// For a type `T` which implements `syn::Parse`, you can call this as `syn_parse(self, T::parse)`.
    /// For more complicated parsers, just pass the parsing function to this function.
    ///
    /// WARNING: With rust-analyzer, this loses transparent groups which have been inserted.
    /// Use only where that doesn't matter: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    ///
    /// Annotate usages with // RUST-ANALYZER SAFETY: ... to explain why the use of this function is OK.
    pub(crate) unsafe fn syn_parse<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(ParseStream) -> Result<T, E>,
    ) -> Result<T, E> {
        self.into_token_stream().parse_with(parser)
    }

    pub(crate) fn append_into(self, output: &mut InterpretedStream) {
        output.segments.extend(self.segments);
        output.token_length += self.token_length;
    }

    pub(crate) fn append_cloned_into(&self, output: &mut InterpretedStream) {
        self.clone().append_into(output);
    }

    /// WARNING: With rust-analyzer, this loses transparent groups which have been inserted.
    /// Use only where that doesn't matter: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    ///
    /// Annotate usages with // RUST-ANALYZER SAFETY: ... to explain why the use of this function is OK.
    pub(crate) unsafe fn into_token_stream(self) -> TokenStream {
        let mut output = TokenStream::new();
        self.append_to_token_stream(&mut output);
        output
    }

    unsafe fn append_to_token_stream(self, output: &mut TokenStream) {
        for segment in self.segments {
            match segment {
                InterpretedSegment::TokenVec(vec) => {
                    output.extend(vec);
                }
                InterpretedSegment::InterpretedGroup(delimiter, span, inner) => {
                    output.extend(iter::once(TokenTree::Group(
                        Group::new(delimiter, inner.into_token_stream()).with_span(span),
                    )))
                }
            }
        }
    }

    pub(crate) fn into_token_stream_removing_any_transparent_groups(self) -> TokenStream {
        let mut output = TokenStream::new();
        self.append_to_token_stream_without_transparent_groups(&mut output);
        output
    }

    fn append_to_token_stream_without_transparent_groups(self, output: &mut TokenStream) {
        for segment in self.segments {
            match segment {
                InterpretedSegment::TokenVec(vec) => {
                    for token in vec {
                        match token {
                            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                                output.extend(group.stream().flatten_transparent_groups());
                            }
                            other => output.extend(iter::once(other)),
                        }
                    }
                }
                InterpretedSegment::InterpretedGroup(delimiter, span, interpreted_stream) => {
                    if delimiter == Delimiter::None {
                        interpreted_stream
                            .append_to_token_stream_without_transparent_groups(output);
                    } else {
                        let mut inner = TokenStream::new();
                        interpreted_stream
                            .append_to_token_stream_without_transparent_groups(&mut inner);
                        output.extend(iter::once(TokenTree::Group(
                            Group::new(delimiter, inner).with_span(span),
                        )));
                    }
                }
            }
        }
    }

    pub(crate) fn unwrap_singleton_group(
        self,
        check_group: impl FnOnce(Delimiter) -> bool,
        create_error: impl FnOnce() -> SynError,
    ) -> ParseResult<InterpretedStream> {
        let mut item_vec = self.into_item_vec();
        if item_vec.len() == 1 {
            match item_vec.pop().unwrap() {
                InterpretedTokenTree::InterpretedGroup(delimiter, _, inner) => {
                    if check_group(delimiter) {
                        return Ok(inner);
                    }
                }
                InterpretedTokenTree::TokenTree(TokenTree::Group(group)) => {
                    if check_group(group.delimiter()) {
                        return Ok(Self::raw(group.stream()));
                    }
                }
                _ => {}
            }
        }
        Err(create_error().into())
    }

    pub(crate) fn into_item_vec(self) -> Vec<InterpretedTokenTree> {
        let mut output = Vec::with_capacity(self.token_length);
        for segment in self.segments {
            match segment {
                InterpretedSegment::TokenVec(vec) => {
                    output.extend(vec.into_iter().map(InterpretedTokenTree::TokenTree));
                }
                InterpretedSegment::InterpretedGroup(delimiter, span, interpreted_stream) => {
                    output.push(InterpretedTokenTree::InterpretedGroup(
                        delimiter,
                        span,
                        interpreted_stream,
                    ));
                }
            }
        }
        output
    }

    pub(crate) fn into_raw_destructure_stream(self) -> RawDestructureStream {
        let mut output = RawDestructureStream::empty();
        for segment in self.segments {
            match segment {
                InterpretedSegment::TokenVec(vec) => {
                    output.append_from_token_stream(vec);
                }
                InterpretedSegment::InterpretedGroup(delimiter, _, inner) => {
                    output.push_item(RawDestructureItem::Group(RawDestructureGroup::new(
                        delimiter,
                        inner.into_raw_destructure_stream(),
                    )));
                }
            }
        }
        output
    }

    pub(crate) fn concat_recursive(self, behaviour: &ConcatBehaviour) -> String {
        fn concat_recursive_interpreted_stream(
            behaviour: &ConcatBehaviour,
            output: &mut String,
            prefix_spacing: Spacing,
            stream: InterpretedStream,
        ) {
            let mut spacing = prefix_spacing;
            for segment in stream.segments {
                spacing = match segment {
                    InterpretedSegment::TokenVec(vec) => {
                        concat_recursive_token_stream(behaviour, output, spacing, vec)
                    }
                    InterpretedSegment::InterpretedGroup(delimiter, _, interpreted_stream) => {
                        behaviour.before_token_tree(output, spacing);
                        behaviour.wrap_delimiters(
                            output,
                            delimiter,
                            interpreted_stream.is_empty(),
                            |output| {
                                concat_recursive_interpreted_stream(
                                    behaviour,
                                    output,
                                    Spacing::Joint,
                                    interpreted_stream,
                                );
                            },
                        );
                        Spacing::Alone
                    }
                }
            }
        }

        fn concat_recursive_token_stream(
            behaviour: &ConcatBehaviour,
            output: &mut String,
            prefix_spacing: Spacing,
            token_stream: impl IntoIterator<Item = TokenTree>,
        ) -> Spacing {
            let mut spacing = prefix_spacing;
            for token_tree in token_stream.into_iter() {
                behaviour.before_token_tree(output, spacing);
                spacing = match token_tree {
                    TokenTree::Literal(literal) => {
                        behaviour.handle_literal(output, literal);
                        Spacing::Alone
                    }
                    TokenTree::Group(group) => {
                        let inner = group.stream();
                        behaviour.wrap_delimiters(
                            output,
                            group.delimiter(),
                            inner.is_empty(),
                            |output| {
                                concat_recursive_token_stream(
                                    behaviour,
                                    output,
                                    Spacing::Joint,
                                    inner,
                                );
                            },
                        );
                        Spacing::Alone
                    }
                    TokenTree::Punct(punct) => {
                        output.push(punct.as_char());
                        punct.spacing()
                    }
                    TokenTree::Ident(ident) => {
                        output.push_str(&ident.to_string());
                        Spacing::Alone
                    }
                }
            }
            spacing
        }

        let mut output = String::new();
        concat_recursive_interpreted_stream(behaviour, &mut output, Spacing::Joint, self);
        output
    }
}

impl IntoIterator for InterpretedStream {
    type IntoIter = std::vec::IntoIter<InterpretedTokenTree>;
    type Item = InterpretedTokenTree;

    fn into_iter(self) -> Self::IntoIter {
        self.into_item_vec().into_iter()
    }
}

pub(crate) struct ConcatBehaviour {
    pub(crate) add_space_between_token_trees: bool,
    pub(crate) output_transparent_group_as_command: bool,
    pub(crate) unwrap_contents_of_string_like_literals: bool,
}

impl ConcatBehaviour {
    pub(crate) fn standard() -> Self {
        Self {
            add_space_between_token_trees: false,
            output_transparent_group_as_command: false,
            unwrap_contents_of_string_like_literals: true,
        }
    }

    pub(crate) fn debug() -> Self {
        Self {
            add_space_between_token_trees: true,
            output_transparent_group_as_command: true,
            unwrap_contents_of_string_like_literals: false,
        }
    }

    fn before_token_tree(&self, output: &mut String, spacing: Spacing) {
        if self.add_space_between_token_trees && spacing == Spacing::Alone {
            output.push(' ');
        }
    }

    fn handle_literal(&self, output: &mut String, literal: Literal) {
        match literal.content_if_string_like() {
            Some(content) if self.unwrap_contents_of_string_like_literals => {
                output.push_str(&content)
            }
            _ => output.push_str(&literal.to_string()),
        }
    }

    fn wrap_delimiters(
        &self,
        output: &mut String,
        delimiter: Delimiter,
        is_empty: bool,
        inner: impl FnOnce(&mut String),
    ) {
        match delimiter {
            Delimiter::Parenthesis => {
                output.push('(');
                inner(output);
                output.push(')');
            }
            Delimiter::Brace => {
                if is_empty {
                    output.push('{');
                    inner(output);
                    output.push('}');
                } else {
                    output.push_str("{ ");
                    inner(output);
                    output.push_str(" }");
                }
            }
            Delimiter::Bracket => {
                output.push('[');
                inner(output);
                output.push(']');
            }
            Delimiter::None => {
                if self.output_transparent_group_as_command {
                    if is_empty {
                        output.push_str("[!group!");
                    } else {
                        output.push_str("[!group! ");
                    }
                    inner(output);
                    output.push(']');
                } else {
                    inner(output);
                }
            }
        }
    }
}

impl From<TokenTree> for InterpretedStream {
    fn from(value: TokenTree) -> Self {
        InterpretedStream::raw(value.into())
    }
}

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

/// Inspired/ forked from [`syn::buffer::TokenBuffer`], in order to support appending tokens,
/// as per the issue here: https://github.com/dtolnay/syn/issues/1842
///
/// Syn is dual-licensed under MIT and Apache, and a subset of it is reproduced from version 2.0.96
/// of syn, and then further edited as a derivative work as part of preinterpret, which is released
/// under the same licenses.
///
/// LICENSE-MIT: https://github.com/dtolnay/syn/blob/2.0.96/LICENSE-MIT
/// LICENSE-APACHE: https://github.com/dtolnay/syn/blob/2.0.96/LICENSE-APACHE
#[allow(unused)]
mod token_buffer {
    use super::*;

    /// Inspired by [`syn::buffer::Entry`]
    /// Internal type which is used instead of `TokenTree` to represent a token tree
    /// within a `TokenBuffer`.
    enum TokenBufferEntry {
        // Mimicking types from proc-macro.
        // Group entries contain the offset to the matching End entry.
        Group(Group, usize),
        Ident(Ident),
        Punct(Punct),
        Literal(Literal),
        // End entries contain the offset (negative) to the start of the buffer, and
        // offset (negative) to the matching Group entry.
        End(isize, isize),
    }

    /// Inspired by [`syn::buffer::TokenBuffer`], but with the ability to append tokens.
    pub(super) struct TokenBuffer {
        entries: Vec<TokenBufferEntry>,
    }

    impl TokenBuffer {
        pub(crate) fn new(tokens: impl IntoIterator<Item = TokenTree>) -> Self {
            let mut entries = vec![];
            Self::recursive_new(&mut entries, tokens);
            entries.push(TokenBufferEntry::End(-(entries.len() as isize), 0));
            Self { entries }
        }

        pub(crate) fn append(&mut self, tokens: impl IntoIterator<Item = TokenTree>) {
            self.entries.pop();
            Self::recursive_new(&mut self.entries, tokens);
            self.entries
                .push(TokenBufferEntry::End(-(self.entries.len() as isize), 0));
        }

        fn recursive_new(
            entries: &mut Vec<TokenBufferEntry>,
            stream: impl IntoIterator<Item = TokenTree>,
        ) {
            for tt in stream {
                match tt {
                    TokenTree::Ident(ident) => entries.push(TokenBufferEntry::Ident(ident)),
                    TokenTree::Punct(punct) => entries.push(TokenBufferEntry::Punct(punct)),
                    TokenTree::Literal(literal) => entries.push(TokenBufferEntry::Literal(literal)),
                    TokenTree::Group(group) => {
                        let group_start_index = entries.len();
                        entries.push(TokenBufferEntry::End(0, 0)); // we replace this below
                        Self::recursive_new(entries, group.stream());
                        let group_end_index = entries.len();
                        let group_offset = group_end_index - group_start_index;
                        entries.push(TokenBufferEntry::End(
                            -(group_end_index as isize),
                            -(group_offset as isize),
                        ));
                        entries[group_start_index] = TokenBufferEntry::Group(group, group_offset);
                    }
                }
            }
        }
    }
}
