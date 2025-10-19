use crate::internal_prelude::*;

pub(crate) trait TokenStreamParseExt: Sized {
    fn source_parse_and_analyze<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(SourceParser) -> Result<T, E>,
        control_flow_analysis: impl FnOnce(&mut T, FlowCapturer) -> Result<(), E>,
    ) -> Result<(T, ScopeDefinitions), E>;

    fn interpreted_parse_with<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(ParseStream<Output>) -> Result<T, E>,
    ) -> Result<T, E>;
}

impl TokenStreamParseExt for TokenStream {
    fn source_parse_and_analyze<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(SourceParser) -> Result<T, E>,
        control_flow_analysis: impl FnOnce(&mut T, FlowCapturer) -> Result<(), E>,
    ) -> Result<(T, ScopeDefinitions), E> {
        parse_with(self, parse_and_analyze(parser, control_flow_analysis))
    }

    fn interpreted_parse_with<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(ParseStream<Output>) -> Result<T, E>,
    ) -> Result<T, E> {
        parse_with(self, parser)
    }
}

pub(crate) fn parse_with<T, K, E: From<syn::Error>>(
    stream: TokenStream,
    parser: impl FnOnce(ParseStream<K>) -> Result<T, E>,
) -> Result<T, E> {
    let mut result = None;
    let parse_result = (|input: SynParseStream| -> SynResult<()> {
        result = Some(parser(input.into()));
        match &result {
            // Some fallback error to ensure that we don't go down the unexpected branch inside parse2
            Some(Err(_)) => Err(SynError::new(Span::call_site(), "")),
            _ => Ok(()),
        }
    })
    .parse2(stream);

    match (result, parse_result) {
        (Some(Ok(value)), Ok(())) => Ok(value),
        (Some(Err(error)), _) => Err(error),
        // If the inner result was Ok, but the parse result was an error, this indicates that the parse2
        // hit the "unexpected" path, indicating that some parse buffer (i.e. group) wasn't fully consumed.
        // So we propagate this error.
        (Some(Ok(_)), Err(error)) => Err(error.into()),
        (None, _) => unreachable!(),
    }
}

pub(crate) trait CursorExt: Sized {
    /// Because syn doesn't parse ' as a punct (not aligned with the TokenTree abstraction)
    fn any_punct(self) -> Option<(Punct, Self)>;
    fn ident_matching(self, content: &str) -> Option<(Ident, Self)>;
    fn punct_matching(self, char: char) -> Option<(Punct, Self)>;
    fn literal_matching(self, content: &str) -> Option<(Literal, Self)>;
    fn group_matching(self, expected_delimiter: Delimiter) -> Option<(DelimSpan, Self, Self)>;
}

impl CursorExt for Cursor<'_> {
    fn any_punct(self) -> Option<(Punct, Self)> {
        match self.token_tree() {
            Some((TokenTree::Punct(punct), next)) => Some((punct, next)),
            _ => None,
        }
    }

    fn ident_matching(self, content: &str) -> Option<(Ident, Self)> {
        match self.ident() {
            Some((ident, next)) if ident == content => Some((ident, next)),
            _ => None,
        }
    }

    fn punct_matching(self, char: char) -> Option<(Punct, Self)> {
        // self.punct() is a little more efficient, but can't match '
        let matcher = if char == '\'' {
            self.any_punct()
        } else {
            self.punct()
        };
        match matcher {
            Some((punct, next)) if punct.as_char() == char => Some((punct, next)),
            _ => None,
        }
    }

    fn literal_matching(self, content: &str) -> Option<(Literal, Self)> {
        match self.literal() {
            Some((literal, next)) if literal.to_string() == content => Some((literal, next)),
            _ => None,
        }
    }

    fn group_matching(self, expected_delimiter: Delimiter) -> Option<(DelimSpan, Self, Self)> {
        match self.any_group() {
            Some((inner_cursor, delimiter, delim_span, next_outer_cursor))
                if delimiter == expected_delimiter =>
            {
                Some((delim_span, inner_cursor, next_outer_cursor))
            }
            _ => None,
        }
    }
}

pub(crate) trait DelimiterExt {
    fn description_of_open(&self) -> &'static str;
    #[allow(unused)]
    fn description_of_group(&self) -> &'static str;
}

impl DelimiterExt for Delimiter {
    fn description_of_open(&self) -> &'static str {
        match self {
            Delimiter::Parenthesis => "(",
            Delimiter::Brace => "{",
            Delimiter::Bracket => "[",
            Delimiter::None => "start of transparent group, from a grouped macro $variable substitution or preinterpret %group[...] literal",
        }
    }

    fn description_of_group(&self) -> &'static str {
        match self {
            Delimiter::Parenthesis => "(...)",
            Delimiter::Brace => "{ ... }",
            Delimiter::Bracket => "[...]",
            Delimiter::None => "transparent group, from a grouped macro $variable substitution or preinterpret %group[...] literal",
        }
    }
}

/// Allows storing a stack of parse buffers for certain parse strategies which require
/// handling multiple groups in parallel.
pub(crate) struct ParseStreamStack<'a> {
    base: SourceParser<'a>,
    group_stack: Vec<SourceParseBuffer<'a>>,
}

impl<'a> ParseStreamStack<'a> {
    pub(crate) fn new(base: SourceParser<'a>) -> Self {
        Self {
            base,
            group_stack: Vec::new(),
        }
    }

    pub(crate) fn current(&self) -> SourceParser<'_> {
        self.group_stack.last().unwrap_or(self.base)
    }

    pub(crate) fn cursor(&self) -> Cursor<'_> {
        self.current().cursor()
    }

    pub(crate) fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        self.current().parse_err(message)
    }

    pub(crate) fn parse<T: ParseSource>(&mut self) -> ParseResult<T> {
        self.current().parse()
    }

    pub(crate) fn is_current_empty(&self) -> bool {
        self.current().is_empty()
    }

    pub(crate) fn peek_grammar(&mut self) -> SourcePeekMatch {
        self.current().peek_grammar()
    }

    #[allow(unused)]
    pub(crate) fn peek<T: syn::parse::Peek>(&mut self, token: T) -> bool {
        self.current().peek(token)
    }

    #[allow(unused)]
    pub(crate) fn peek2<T: syn::parse::Peek>(&mut self, token: T) -> bool {
        self.current().peek2(token)
    }

    pub(crate) fn parse_any_ident(&mut self) -> ParseResult<Ident> {
        self.current().parse_any_ident()
    }

    pub(crate) fn try_parse_or_revert<T: ParseSource>(&mut self) -> ParseResult<T> {
        let current = self.current();
        let fork = current.fork();
        match fork.parse::<T>() {
            Ok(output) => {
                current.advance_to(&fork);
                Ok(output)
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn parse_and_enter_group(&mut self) -> ParseResult<(Delimiter, DelimSpan)> {
        let (delimiter, delim_span, inner) = self.current().parse_any_group()?;
        let inner = unsafe {
            // SAFETY: This is safe because the lifetime is there for two reasons:
            // (A) Prevent mixing up different buffers from e.g. different groups,
            // (B) Ensure the buffers are dropped in the correct order so that the unexpected drop glue triggers
            // in the correct order.
            //
            // This invariant is maintained by this `ParseStreamStack` struct:
            // (A) Is enforced by the fact we're parsing the group from the top parse buffer current().
            // (B) Is enforced by a combination of:
            // ==> exit_group() ensures the parse buffers are dropped in the correct order
            // ==> If a user forgets to do it (or e.g. an error path or panic causes exit_group not to be called)
            //     Then the drop glue ensures the groups are dropped in the correct order.
            std::mem::transmute::<SourceParseBuffer<'_>, SourceParseBuffer<'a>>(inner)
        };
        self.group_stack.push(inner);
        Ok((delimiter, delim_span))
    }

    /// Should be paired with `parse_and_enter_group`.
    ///
    /// If the group is not finished, the next attempt to read from the parent will trigger an error,
    /// in accordance with the drop glue on `ParseBuffer`.
    ///
    /// ### Panics
    /// Panics if there is no group available.
    pub(crate) fn exit_group(&mut self) {
        self.group_stack
            .pop()
            .expect("finish_group must be paired with push_group");
    }
}

impl Drop for ParseStreamStack<'_> {
    fn drop(&mut self) {
        while !self.group_stack.is_empty() {
            self.exit_group();
        }
    }
}
