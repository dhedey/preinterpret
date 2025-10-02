use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct OutputStream {
    /// Currently, even ~inside~ macro executions, round-tripping [`Delimiter::None`] groups to a TokenStream
    /// breaks in rust-analyzer. This causes various spurious errors in the IDE.
    /// See: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    ///
    /// So instead of just storing a TokenStream here, we store a list of TokenStreams and interpreted groups.
    /// This ensures any non-delimited groups are not round-tripped to a TokenStream.
    ///
    /// In future, we may wish to internally use some kind of `TokenBuffer` type, which I've started on below.
    segments: Vec<OutputSegment>,
    token_length: usize,
}

#[derive(Clone)]
// This was primarily implemented to avoid this issue: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
// But it doesn't actually help because the `syn::parse` mechanism only operates on a TokenStream,
// so we have to convert back into a TokenStream.
enum OutputSegment {
    TokenVec(Vec<TokenTree>), // Cheaper than a TokenStream (probably)
    OutputGroup(Delimiter, Span, OutputStream),
}

#[derive(Clone)]
pub(crate) enum OutputTokenTree {
    TokenTree(TokenTree),
    OutputGroup(Delimiter, Span, OutputStream),
}

impl From<OutputTokenTree> for OutputStream {
    fn from(value: OutputTokenTree) -> Self {
        let mut new = Self::new();
        new.push_interpreted_item(value);
        new
    }
}

impl HasSpan for OutputTokenTree {
    fn span(&self) -> Span {
        match self {
            OutputTokenTree::TokenTree(token_tree) => token_tree.span(),
            OutputTokenTree::OutputGroup(_, span, _) => *span,
        }
    }
}

impl OutputStream {
    pub(crate) fn new() -> Self {
        Self {
            segments: vec![],
            token_length: 0,
        }
    }

    pub(crate) fn new_with(
        appender: impl FnOnce(&mut Self) -> ExecutionResult<()>,
    ) -> ExecutionResult<Self> {
        let mut stream = Self::new();
        appender(&mut stream)?;
        Ok(stream)
    }

    #[allow(unused)]
    pub(crate) fn new_grouped(
        appender: impl FnOnce(&mut Self) -> ExecutionResult<()>,
        delimiter: Delimiter,
        span: Span,
    ) -> ExecutionResult<Self> {
        let mut stream = Self::new();
        stream.push_grouped(appender, delimiter, span)?;
        Ok(stream)
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
        inner_tokens: OutputStream,
        delimiter: Delimiter,
        span: Span,
    ) {
        self.segments
            .push(OutputSegment::OutputGroup(delimiter, span, inner_tokens));
        self.token_length += 1;
    }

    pub(crate) fn push_interpreted_item(&mut self, segment_item: OutputTokenTree) {
        match segment_item {
            OutputTokenTree::TokenTree(token_tree) => {
                self.push_raw_token_tree(token_tree);
            }
            OutputTokenTree::OutputGroup(delimiter, span, inner_tokens) => {
                self.push_new_group(inner_tokens, delimiter, span);
            }
        }
    }

    pub(crate) fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
        self.extend_raw_tokens(iter::once(token_tree));
    }

    pub(crate) fn extend_raw_tokens(&mut self, tokens: impl IntoIterator<Item = TokenTree>) {
        if !matches!(self.segments.last(), Some(OutputSegment::TokenVec(_))) {
            self.segments.push(OutputSegment::TokenVec(vec![]));
        }

        match self.segments.last_mut() {
            Some(OutputSegment::TokenVec(token_vec)) => {
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

    pub(crate) fn coerce_into_value(self, stream_span_range: SpanRange) -> ExpressionValue {
        let parse_result = unsafe {
            // RUST-ANALYZER SAFETY: This is actually safe.
            self.clone().parse_as::<syn::Lit>()
        };
        match parse_result {
            Ok(syn_lit) => ExpressionValue::for_syn_lit(syn_lit),
            Err(_) => self.to_value(stream_span_range),
        }
    }

    /// WARNING: With rust-analyzer, this loses transparent groups which have been inserted.
    /// Use only where that doesn't matter: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    ///
    /// Annotate usages with // RUST-ANALYZER SAFETY: ... to explain why the use of this function is OK.
    pub(crate) unsafe fn parse_with<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(ParseStream<Output>) -> Result<T, E>,
    ) -> Result<T, E> {
        self.into_token_stream().interpreted_parse_with(parser)
    }

    /// WARNING: With rust-analyzer, this loses transparent groups which have been inserted.
    /// Use only where that doesn't matter: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    ///
    /// Annotate usages with // RUST-ANALYZER SAFETY: ... to explain why the use of this function is OK.
    pub(crate) unsafe fn parse_as<T: Parse<Output>>(self) -> ParseResult<T> {
        self.into_token_stream().interpreted_parse_with(T::parse)
    }

    pub(crate) fn append_into(self, output: &mut OutputStream) {
        output.segments.extend(self.segments);
        output.token_length += self.token_length;
    }

    pub(crate) fn append_cloned_into(&self, output: &mut OutputStream) {
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
                OutputSegment::TokenVec(vec) => {
                    output.extend(vec);
                }
                OutputSegment::OutputGroup(delimiter, span, inner) => {
                    output.extend(iter::once(TokenTree::Group(
                        Group::new(delimiter, inner.into_token_stream()).with_span(span),
                    )))
                }
            }
        }
    }

    pub(crate) fn to_token_stream_removing_any_transparent_groups(&self) -> TokenStream {
        let mut output = TokenStream::new();
        self.append_to_token_stream_without_transparent_groups(&mut output);
        output
    }

    fn append_to_token_stream_without_transparent_groups(&self, output: &mut TokenStream) {
        for segment in self.segments.iter() {
            match segment {
                OutputSegment::TokenVec(vec) => {
                    for token in vec {
                        match token {
                            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                                output.extend(group.stream().flatten_transparent_groups());
                            }
                            other => output.extend(iter::once(other.clone())),
                        }
                    }
                }
                OutputSegment::OutputGroup(delimiter, span, interpreted_stream) => {
                    let delimiter = *delimiter;
                    let span = *span;
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

    pub(crate) fn into_item_vec(self) -> Vec<OutputTokenTree> {
        let mut output = Vec::with_capacity(self.token_length);
        for segment in self.segments {
            match segment {
                OutputSegment::TokenVec(vec) => {
                    output.extend(vec.into_iter().map(OutputTokenTree::TokenTree));
                }
                OutputSegment::OutputGroup(delimiter, span, interpreted_stream) => {
                    output.push(OutputTokenTree::OutputGroup(
                        delimiter,
                        span,
                        interpreted_stream,
                    ));
                }
            }
        }
        output
    }

    pub(crate) fn concat_recursive(&self, behaviour: &ConcatBehaviour) -> String {
        let mut output = String::new();
        self.concat_recursive_into(&mut output, behaviour);
        output
    }

    pub(crate) fn concat_recursive_into(&self, output: &mut String, behaviour: &ConcatBehaviour) {
        fn concat_recursive_interpreted_stream(
            behaviour: &ConcatBehaviour,
            output: &mut String,
            prefix_spacing: Spacing,
            stream: &OutputStream,
        ) {
            let mut spacing = prefix_spacing;
            for segment in stream.segments.iter() {
                spacing = match segment {
                    OutputSegment::TokenVec(vec) => {
                        concat_recursive_token_stream(behaviour, output, spacing, vec)
                    }
                    OutputSegment::OutputGroup(delimiter, _, interpreted_stream) => {
                        behaviour.before_token_tree(output, spacing);
                        behaviour.wrap_delimiters(
                            output,
                            *delimiter,
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

        fn concat_recursive_token_stream<T: core::borrow::Borrow<TokenTree>>(
            behaviour: &ConcatBehaviour,
            output: &mut String,
            prefix_spacing: Spacing,
            token_stream: impl IntoIterator<Item = T>,
        ) -> Spacing {
            let mut spacing = prefix_spacing;
            for token_tree in token_stream.into_iter() {
                let token_tree = token_tree.borrow();
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
                        let char = punct.as_char();
                        // This conversion captures ~#% as ~%raw[#]%raw[%] rather than the
                        // more accurate %raw[~#%] which also captures the correct punct spacing.
                        // To do this properly would require lookahead which require quite a big
                        // logic change! So I've opted to ignore it for now...
                        // * Debug string isn't expected to be used for re-parsing
                        // * Even if it were, the specific spacing isn't used/relevant
                        //   for any known grammar
                        if behaviour.use_stream_literal_syntax && (char == '#' || char == '%') {
                            output.push_str("%raw[");
                            output.push(char);
                            output.push(']');
                        } else {
                            output.push(char);
                        }
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

        concat_recursive_interpreted_stream(behaviour, output, Spacing::Joint, self);
    }

    pub(crate) fn parse_exact_match(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        handle_parsing_exact_output_match(input, interpreter, self, output)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = OutputTokenTreeRef<'_>> {
        self.segments.iter().flat_map(|segment| match segment {
            OutputSegment::TokenVec(vec) => {
                EitherIterator::Left(vec.iter().map(OutputTokenTreeRef::TokenTree))
            }
            OutputSegment::OutputGroup(delimiter, span, inner) => EitherIterator::Right(
                [OutputTokenTreeRef::OutputGroup(*delimiter, *span, inner)].into_iter(),
            ),
        })
    }
}

pub(crate) enum OutputTokenTreeRef<'a> {
    TokenTree(&'a TokenTree),
    #[allow(unused)]
    OutputGroup(Delimiter, Span, &'a OutputStream),
}

impl IntoIterator for OutputStream {
    type IntoIter = std::vec::IntoIter<OutputTokenTree>;
    type Item = OutputTokenTree;

    fn into_iter(self) -> Self::IntoIter {
        self.into_item_vec().into_iter()
    }
}

pub(crate) struct ConcatBehaviour {
    pub(crate) add_space_between_token_trees: bool,
    pub(crate) use_stream_literal_syntax: bool,
    pub(crate) use_rust_literal_syntax: bool,
    pub(crate) output_array_structure: bool,
    pub(crate) unwrap_contents_of_string_like_literals: bool,
    pub(crate) show_none_values: bool,
    pub(crate) iterator_limit: usize,
    pub(crate) error_after_iterator_limit: bool,
}

impl ConcatBehaviour {
    pub(crate) fn standard() -> Self {
        Self {
            add_space_between_token_trees: false,
            use_stream_literal_syntax: false,
            use_rust_literal_syntax: false,
            output_array_structure: false,
            unwrap_contents_of_string_like_literals: true,
            show_none_values: false,
            iterator_limit: 1000,
            error_after_iterator_limit: true,
        }
    }

    pub(crate) fn literal() -> Self {
        Self {
            add_space_between_token_trees: false,
            use_stream_literal_syntax: false,
            use_rust_literal_syntax: true,
            output_array_structure: false,
            unwrap_contents_of_string_like_literals: true,
            show_none_values: false,
            iterator_limit: 1000,
            error_after_iterator_limit: true,
        }
    }

    pub(crate) fn debug() -> Self {
        Self {
            add_space_between_token_trees: true,
            use_stream_literal_syntax: true,
            use_rust_literal_syntax: true,
            output_array_structure: true,
            unwrap_contents_of_string_like_literals: false,
            show_none_values: true,
            iterator_limit: 20,
            error_after_iterator_limit: false,
        }
    }

    pub(crate) fn before_token_tree(&self, output: &mut String, spacing: Spacing) {
        if self.add_space_between_token_trees && spacing == Spacing::Alone {
            output.push(' ');
        }
    }

    pub(crate) fn handle_literal(&self, output: &mut String, literal: &Literal) {
        match literal.content_if_string_like() {
            Some(content) if self.unwrap_contents_of_string_like_literals => {
                output.push_str(&content)
            }
            _ => {
                if self.use_rust_literal_syntax {
                    output.push_str(&literal.to_string())
                } else {
                    output.push_str(&literal.inner_value_to_string())
                }
            }
        }
    }

    pub(crate) fn wrap_delimiters(
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
                if self.use_stream_literal_syntax {
                    output.push_str("%group[");
                    inner(output);
                    output.push(']');
                } else {
                    inner(output);
                }
            }
        }
    }
}

impl From<TokenTree> for OutputStream {
    fn from(value: TokenTree) -> Self {
        OutputStream::raw(value.into())
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
// * e.g. of a token stream in `SourceValue`
// * e.g. as part of a PARSER, from an OutputStream
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
// We could use this abstraction for OutputStream; and our variables could store a
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
