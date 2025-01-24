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
        let parse_result = (|input: ParseStream| -> SynResult<()> {
            result = Some(parser(input));
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

pub(crate) trait ParserExt {
    fn parse_v2<T: Parse>(&self) -> ParseResult<T>;
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
    fn peek_group_matching(&self, delimiter: Delimiter) -> bool;
    fn parse_group_matching(&self, delimiter: Delimiter) -> ParseResult<(DelimSpan, ParseBuffer)>;
    fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T>;
    fn parse_error(&self, message: impl std::fmt::Display) -> ParseError;
}

impl ParserExt for ParseBuffer<'_> {
    fn parse_v2<T: Parse>(&self) -> ParseResult<T> {
        T::parse(self)
    }

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
        Ok(Ident::parse_any(self)?)
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

    fn peek_group_matching(&self, delimiter: Delimiter) -> bool {
        self.cursor().group_matching(delimiter).is_some()
    }

    fn parse_group_matching(
        &self,
        expected_delimiter: Delimiter,
    ) -> ParseResult<(DelimSpan, ParseBuffer)> {
        let (delimiter, delim_span, inner_stream) = self.parse_any_delimiter()?;
        if delimiter != expected_delimiter {
            return delim_span.open().parse_err(match expected_delimiter {
                Delimiter::Parenthesis => "Expected (",
                Delimiter::Brace => "Expected {",
                Delimiter::Bracket => "Expected [",
                Delimiter::None => "Expected start of transparent group",
            });
        }
        Ok((delim_span, inner_stream))
    }

    fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        Err(self.parse_error(message))
    }

    fn parse_error(&self, message: impl std::fmt::Display) -> ParseError {
        self.span().parse_error(message)
    }
}
