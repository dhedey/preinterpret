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

impl OutputStream {
    pub(crate) fn new() -> Self {
        Self {
            segments: vec![],
            token_length: 0,
        }
    }

    pub(crate) fn new_with(appender: impl FnOnce(&mut Self)) -> Self {
        let mut stream = Self::new();
        appender(&mut stream);
        stream
    }

    #[allow(unused)]
    pub(crate) fn new_try_with(
        appender: impl FnOnce(&mut Self) -> ExecutionResult<()>,
    ) -> ExecutionResult<Self> {
        let mut stream = Self::new();
        appender(&mut stream)?;
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

    pub(crate) fn push_tokens(&mut self, tokens: impl ToTokens) {
        self.extend_raw_tokens(tokens.into_token_stream());
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

    pub(crate) fn coerce_into_value(self) -> Value {
        let parse_result = self.clone().parse_as::<syn::Lit>();
        match parse_result {
            Ok(syn_lit) => Value::for_syn_lit(syn_lit).into_inner(),
            // Keep as stream otherwise
            Err(_) => self.into_value(),
        }
    }

    /// WARNING: With rust-analyzer, this loses transparent groups which have been inserted.
    /// Use only where that doesn't matter: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    pub(crate) fn parse_with<T>(
        self,
        parser: impl FnOnce(ParseStream<Output>) -> ExecutionResult<T>,
    ) -> ExecutionResult<T> {
        self.into_token_stream().interpreted_parse_with(parser)
    }

    /// WARNING: With rust-analyzer, this loses transparent groups which have been inserted.
    /// Use only where that doesn't matter: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
    pub(crate) fn parse_as<T: Parse<Output>>(self) -> ParseResult<T> {
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
    pub(crate) fn into_token_stream(self) -> TokenStream {
        let mut output = TokenStream::new();
        self.append_to_token_stream(&mut output);
        output
    }

    fn append_to_token_stream(self, output: &mut TokenStream) {
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

    pub(crate) fn replace_first_level_spans(&mut self, new_span: Span) {
        for segment in self.segments.iter_mut() {
            match segment {
                OutputSegment::TokenVec(vec) => {
                    for token in vec.iter_mut() {
                        match token {
                            TokenTree::Group(group) => group.set_span(new_span),
                            TokenTree::Ident(ident) => ident.set_span(new_span),
                            TokenTree::Punct(punct) => punct.set_span(new_span),
                            TokenTree::Literal(literal) => literal.set_span(new_span),
                        }
                    }
                }
                OutputSegment::OutputGroup(_, span, _) => {
                    *span = new_span;
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

    pub(crate) fn concat_content(&self, behaviour: &ConcatBehaviour) -> String {
        let mut output = String::new();
        self.concat_content_into(&mut output, behaviour);
        output
    }

    pub(crate) fn concat_content_into(&self, output: &mut String, behaviour: &ConcatBehaviour) {
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
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        handle_parsing_exact_output_match(input, self, output)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = OutputTokenTreeRef<'_>> {
        self.segments.iter().flat_map(|segment| match segment {
            OutputSegment::TokenVec(vec) => {
                EitherIterator::Left(vec.iter().map(OutputTokenTreeRef::TokenTree))
            }
            OutputSegment::OutputGroup(delimiter, span, inner) => EitherIterator::Right(
                core::iter::once(OutputTokenTreeRef::OutputGroup(*delimiter, *span, inner)),
            ),
        })
    }
}

pub(crate) enum OutputTokenTreeRef<'a> {
    TokenTree(&'a TokenTree),
    #[allow(unused)]
    OutputGroup(Delimiter, Span, &'a OutputStream),
}

#[derive(Clone)]
pub(crate) struct OutputStreamIntoIter {
    segments: std::vec::IntoIter<OutputSegment>,
    current_segment_iter: Option<OutputSegmentIntoIter>,
    count_remaining: usize,
}

impl OutputStreamIntoIter {
    fn new(stream: OutputStream) -> Self {
        let mut segments = stream.segments.into_iter();
        let current_segment_iter = segments.next().map(OutputSegment::into_iter);
        let count_remaining = stream.token_length;
        Self {
            segments,
            current_segment_iter,
            count_remaining,
        }
    }
}

impl Iterator for OutputStreamIntoIter {
    type Item = OutputTokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count_remaining == 0 {
            return None;
        }

        loop {
            if let Some(current_segment_iter) = &mut self.current_segment_iter {
                if let Some(item) = current_segment_iter.next() {
                    self.count_remaining -= 1;
                    return Some(item);
                }
            }

            match self.segments.next() {
                Some(next_segment) => {
                    self.current_segment_iter = Some(next_segment.into_iter());
                }
                None => {
                    self.current_segment_iter = None;
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count_remaining, Some(self.count_remaining))
    }
}

impl IntoIterator for OutputStream {
    type IntoIter = OutputStreamIntoIter;
    type Item = OutputTokenTree;

    fn into_iter(self) -> Self::IntoIter {
        OutputStreamIntoIter::new(self)
    }
}

pub(crate) struct ConcatBehaviour {
    pub(crate) use_debug_literal_syntax: bool,
    pub(crate) add_space_between_token_trees: bool,
    pub(crate) use_stream_literal_syntax: bool,
    pub(crate) output_literal_structure: bool,
    pub(crate) unwrap_contents_of_string_like_literals: bool,
    pub(crate) show_none_values: bool,
    pub(crate) iterator_limit: usize,
    pub(crate) error_after_iterator_limit: bool,
    pub(crate) error_span_range: SpanRange,
}

impl ConcatBehaviour {
    pub(crate) fn standard(error_span_range: SpanRange) -> Self {
        Self {
            add_space_between_token_trees: false,
            use_stream_literal_syntax: false,
            use_debug_literal_syntax: false,
            output_literal_structure: false,
            unwrap_contents_of_string_like_literals: true,
            show_none_values: false,
            iterator_limit: 1000,
            error_after_iterator_limit: true,
            error_span_range,
        }
    }

    pub(crate) fn literal(error_span_range: SpanRange) -> Self {
        Self {
            add_space_between_token_trees: false,
            use_stream_literal_syntax: false,
            use_debug_literal_syntax: true,
            output_literal_structure: false,
            unwrap_contents_of_string_like_literals: true,
            show_none_values: false,
            iterator_limit: 1000,
            error_after_iterator_limit: true,
            error_span_range,
        }
    }

    pub(crate) fn debug(error_span_range: SpanRange) -> Self {
        Self {
            add_space_between_token_trees: true,
            use_stream_literal_syntax: true,
            use_debug_literal_syntax: true,
            output_literal_structure: true,
            unwrap_contents_of_string_like_literals: false,
            show_none_values: true,
            iterator_limit: 20,
            error_after_iterator_limit: false,
            error_span_range,
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
                if self.use_debug_literal_syntax {
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

#[derive(Clone)]
// This was primarily implemented to avoid this issue: https://github.com/rust-lang/rust-analyzer/issues/18211#issuecomment-2604547032
// But it doesn't actually help because the `syn::parse` mechanism only operates on a TokenStream,
// so we have to convert back into a TokenStream.
enum OutputSegment {
    TokenVec(Vec<TokenTree>), // Cheaper than a TokenStream (probably)
    OutputGroup(Delimiter, Span, OutputStream),
}

#[derive(Clone)]
enum OutputSegmentIntoIter {
    Vec(std::vec::IntoIter<TokenTree>),
    Single(Option<OutputTokenTree>),
}

impl Iterator for OutputSegmentIntoIter {
    type Item = OutputTokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OutputSegmentIntoIter::Vec(iter) => iter.next().map(OutputTokenTree::TokenTree),
            OutputSegmentIntoIter::Single(option) => option.take(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OutputSegmentIntoIter::Vec(iter) => iter.size_hint(),
            OutputSegmentIntoIter::Single(option) => {
                let len = if option.is_some() { 1 } else { 0 };
                (len, Some(len))
            }
        }
    }
}

impl IntoIterator for OutputSegment {
    type IntoIter = OutputSegmentIntoIter;
    type Item = OutputTokenTree;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OutputSegment::TokenVec(vec) => OutputSegmentIntoIter::Vec(vec.into_iter()),
            OutputSegment::OutputGroup(delimiter, span, interpreted_stream) => {
                OutputSegmentIntoIter::Single(Some(OutputTokenTree::OutputGroup(
                    delimiter,
                    span,
                    interpreted_stream,
                )))
            }
        }
    }
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
