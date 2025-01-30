use crate::internal_prelude::*;

pub(crate) trait TokenStreamParseExt: Sized {
    fn parse_with<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(ParseStream) -> Result<T, E>,
    ) -> Result<T, E>;
}

impl TokenStreamParseExt for TokenStream {
    fn parse_with<T, E: From<syn::Error>>(
        self,
        parser: impl FnOnce(ParseStream) -> Result<T, E>,
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
        .parse2(self);

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

pub(crate) trait ParserBufferExt {
    fn parse_with<T: ContextualParse>(&self, context: T::Context) -> ParseResult<T>;
    fn parse_all_for_interpretation(
        &self,
        span_range: SpanRange,
    ) -> ParseResult<InterpretationStream>;
    fn try_parse_or_message<T, F: FnOnce(&Self) -> ParseResult<T>, M: std::fmt::Display>(
        &self,
        func: F,
        message: M,
    ) -> ParseResult<T>;
    fn parse_any_ident(&self) -> ParseResult<Ident>;
    fn parse_any_punct(&self) -> ParseResult<Punct>;
    fn peek_ident_matching(&self, content: &str) -> bool;
    fn parse_ident_matching(&self, content: &str) -> ParseResult<Ident>;
    fn peek_punct_matching(&self, punct: char) -> bool;
    fn parse_punct_matching(&self, content: char) -> ParseResult<Punct>;
    fn peek_literal_matching(&self, content: &str) -> bool;
    fn parse_literal_matching(&self, content: &str) -> ParseResult<Literal>;
    fn parse_any_group(&self) -> ParseResult<(Delimiter, DelimSpan, ParseBuffer)>;
    fn peek_specific_group(&self, delimiter: Delimiter) -> bool;
    fn parse_group_matching(
        &self,
        matching: impl FnOnce(Delimiter) -> bool,
        expected_message: impl FnOnce() -> String,
    ) -> ParseResult<(DelimSpan, ParseBuffer)>;
    fn parse_specific_group(&self, delimiter: Delimiter) -> ParseResult<(DelimSpan, ParseBuffer)>;
    fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T>;
    fn parse_error(&self, message: impl std::fmt::Display) -> ParseError;
}

impl ParserBufferExt for ParseBuffer<'_> {
    fn parse_with<T: ContextualParse>(&self, context: T::Context) -> ParseResult<T> {
        T::parse_with_context(self, context)
    }

    fn parse_all_for_interpretation(
        &self,
        span_range: SpanRange,
    ) -> ParseResult<InterpretationStream> {
        self.parse_with(span_range)
    }

    fn try_parse_or_message<T, F: FnOnce(&Self) -> ParseResult<T>, M: std::fmt::Display>(
        &self,
        parse: F,
        message: M,
    ) -> ParseResult<T> {
        let error_span = self.span();
        parse(self).map_err(|_| error_span.error(message).into())
    }

    fn parse_any_ident(&self) -> ParseResult<Ident> {
        Ok(self.call(Ident::parse_any)?)
    }

    fn parse_any_punct(&self) -> ParseResult<Punct> {
        // Annoyingly, ' behaves weirdly in syn, so we need to handle it
        match self.parse::<TokenTree>()? {
            TokenTree::Punct(punct) => Ok(punct),
            _ => self.span().parse_err("expected punctuation"),
        }
    }

    fn peek_ident_matching(&self, content: &str) -> bool {
        self.cursor().ident_matching(content).is_some()
    }

    fn parse_ident_matching(&self, content: &str) -> ParseResult<Ident> {
        Ok(self.step(|cursor| {
            cursor
                .ident_matching(content)
                .ok_or_else(|| cursor.span().error(format!("expected {}", content)))
        })?)
    }

    fn peek_punct_matching(&self, punct: char) -> bool {
        self.cursor().punct_matching(punct).is_some()
    }

    fn parse_punct_matching(&self, punct: char) -> ParseResult<Punct> {
        Ok(self.step(|cursor| {
            cursor
                .punct_matching(punct)
                .ok_or_else(|| cursor.span().error(format!("expected {}", punct)))
        })?)
    }

    fn peek_literal_matching(&self, content: &str) -> bool {
        self.cursor().literal_matching(content).is_some()
    }

    fn parse_literal_matching(&self, content: &str) -> ParseResult<Literal> {
        Ok(self.step(|cursor| {
            cursor
                .literal_matching(content)
                .ok_or_else(|| cursor.span().error(format!("expected {}", content)))
        })?)
    }

    fn parse_any_group(&self) -> ParseResult<(Delimiter, DelimSpan, ParseBuffer)> {
        use syn::parse::discouraged::AnyDelimiter;
        let (delimiter, delim_span, parse_buffer) = self.parse_any_delimiter()?;
        Ok((delimiter, delim_span, parse_buffer.into()))
    }

    fn peek_specific_group(&self, delimiter: Delimiter) -> bool {
        self.cursor().group_matching(delimiter).is_some()
    }

    fn parse_group_matching(
        &self,
        matching: impl FnOnce(Delimiter) -> bool,
        expected_message: impl FnOnce() -> String,
    ) -> ParseResult<(DelimSpan, ParseBuffer)> {
        use syn::parse::discouraged::AnyDelimiter;
        let error_span = match self.parse_any_delimiter() {
            Ok((delimiter, delim_span, inner)) if matching(delimiter) => {
                return Ok((delim_span, inner.into()));
            }
            Ok((_, delim_span, _)) => delim_span.open(),
            Err(error) => error.span(),
        };
        error_span.parse_err(expected_message())
    }

    fn parse_specific_group(
        &self,
        expected_delimiter: Delimiter,
    ) -> ParseResult<(DelimSpan, ParseBuffer)> {
        self.parse_group_matching(
            |delimiter| delimiter == expected_delimiter,
            || format!("Expected {}", expected_delimiter.description_of_open()),
        )
    }

    fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        Err(self.parse_error(message))
    }

    fn parse_error(&self, message: impl std::fmt::Display) -> ParseError {
        self.span().parse_error(message)
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
            Delimiter::None => "start of transparent group, from a grouped #variable substitution or stream-based command such as [!group! ...]",
        }
    }

    fn description_of_group(&self) -> &'static str {
        match self {
            Delimiter::Parenthesis => "(...)",
            Delimiter::Brace => "{ ... }",
            Delimiter::Bracket => "[...]",
            Delimiter::None => "transparent group, from a grouped #variable substitution or stream-based command such as [!group! ...]",
        }
    }
}

/// Allows storing a stack of parse buffers for certain parse strategies which require
/// handling multiple groups in parallel.
pub(crate) struct ParseStreamStack<'a> {
    base: ParseStream<'a>,
    group_stack: Vec<ParseBuffer<'a>>,
}

impl<'a> ParseStreamStack<'a> {
    pub(crate) fn new(base: ParseStream<'a>) -> Self {
        Self {
            base,
            group_stack: Vec::new(),
        }
    }

    fn current(&self) -> ParseStream<'_> {
        self.group_stack.last().unwrap_or(self.base)
    }

    pub(crate) fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        self.current().parse_err(message)
    }

    pub(crate) fn peek_grammar(&mut self) -> PeekMatch {
        detect_preinterpret_grammar(self.current().cursor())
    }

    pub(crate) fn parse<T: Parse>(&mut self) -> ParseResult<T> {
        self.current().parse()
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
            std::mem::transmute::<ParseBuffer<'_>, ParseBuffer<'a>>(inner)
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
