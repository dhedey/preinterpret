use crate::internal_prelude::*;

// ===============================================
// How syn features fits with preinterpret parsing
// ===============================================
//
// There are a few places where we parse in preinterpret:
// * Parse the initial input from the macro, into an interpretable structure
// * Parsing as part of interpretation
//   * e.g. of raw tokens, either from the preinterpret declaration, or from a variable
//   * e.g. of a variable, as part of incremental parsing (while_parse style loops)
//
// I spent quite a while considering whether this could be wrapping a
// `syn::parse::ParseBuffer<'a>` or `syn::buffer::Cursor<'a>`...
//
// Parsing the initial input
// -------------------------
//
// This is where InterpreterParseStream comes in.
//
// Now that I've changed how preinterpret works to be a two-pass approach
// (first parsing, then interpreting), we could consider swapping out the
// InterpreterParseStream to wrap a syn::ParseStream instead.
//
// This probably could work, but has a little more overhead to swap to a
// TokenBuffer (and so ParseStream) and back.
//
// Parsing in the context of a command execution
// ---------------------------------------------
//
// For parse/destructuring operations, we could temporarily create parse
// streams in scope of a command execution.
//
// Parsing a variable or other token stream
// ----------------------------------------
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

#[derive(Clone)]
pub(crate) struct InterpreterParseStream {
    tokens: iter::Peekable<<TokenStream as IntoIterator>::IntoIter>,
    /// The span range of the original stream, before tokens were consumed
    full_span_range: SpanRange,
    /// The span of the last item consumed (or the full span range if no items have been consumed yet)
    latest_item_span_range: SpanRange,
}

impl InterpreterParseStream {
    pub(crate) fn new(token_stream: TokenStream, span_range: SpanRange) -> Self {
        Self {
            tokens: token_stream.into_iter().peekable(),
            full_span_range: span_range,
            latest_item_span_range: span_range,
        }
    }

    pub(crate) fn full_span_range(&self) -> SpanRange {
        self.full_span_range
    }

    pub(super) fn peek_token_tree(&mut self) -> Option<&TokenTree> {
        self.tokens.peek()
    }

    pub(super) fn next_token_tree_or_end(&mut self) -> Option<TokenTree> {
        self.tokens.next()
    }

    #[allow(unused)]
    pub(super) fn next_token_tree(&mut self, error_message: &str) -> Result<TokenTree> {
        match self.next_token_tree_or_end() {
            Some(token_tree) => Ok(token_tree),
            None => self
                .latest_item_span_range
                .err(format!("Unexpected end: {error_message}")),
        }
    }

    fn next_item_or_end(&mut self) -> Result<Option<InterpretationItem>> {
        let next_item = InterpretationItem::parse(self)?;
        Ok(match next_item {
            Some(next_item) => {
                self.latest_item_span_range = next_item.span_range();
                Some(next_item)
            }
            None => None,
        })
    }

    pub(crate) fn next_item(&mut self, error_message: &str) -> Result<InterpretationItem> {
        match self.next_item_or_end()? {
            Some(item) => Ok(item),
            None => self
                .latest_item_span_range
                .err(format!("Unexpected end: {error_message}")),
        }
    }

    pub(crate) fn next_as_ident(&mut self, error_message: &'static str) -> Result<Ident> {
        match self.next_item(error_message)? {
            InterpretationItem::Ident(ident) => Ok(ident),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_ident_matching(
        &mut self,
        ident_name: &str,
        error_message: &'static str,
    ) -> Result<Ident> {
        match self.next_item(error_message)? {
            InterpretationItem::Ident(ident) if ident == ident_name => Ok(ident),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_punct(&mut self, error_message: &'static str) -> Result<Punct> {
        match self.next_item(error_message)? {
            InterpretationItem::Punct(punct) => Ok(punct),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_punct_matching(
        &mut self,
        char: char,
        error_message: &'static str,
    ) -> Result<Punct> {
        match self.next_item(error_message)? {
            InterpretationItem::Punct(punct) if punct.as_char() == char => Ok(punct),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_kinded_group(
        &mut self,
        delimiter: Delimiter,
        error_message: &'static str,
    ) -> Result<InterpretationGroup> {
        match self.next_item(error_message)? {
            InterpretationItem::Group(group) if group.delimiter() == delimiter => Ok(group),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_variable(&mut self, error_message: &'static str) -> Result<Variable> {
        match self.next_item(error_message)? {
            InterpretationItem::Variable(variable_substitution) => Ok(variable_substitution),
            other => other.err(error_message),
        }
    }

    pub(crate) fn is_empty(&mut self) -> bool {
        self.peek_token_tree().is_none()
    }

    pub(crate) fn assert_end(&mut self, error_message: &'static str) -> Result<()> {
        match self.next_token_tree_or_end() {
            Some(token) => token.span_range().err(error_message),
            None => Ok(()),
        }
    }

    pub(crate) fn parse_all_for_interpretation(&mut self) -> Result<InterpretationStream> {
        InterpretationStream::parse(self, self.full_span_range)
    }

    pub(crate) fn read_all_as_raw_token_stream(&mut self) -> TokenStream {
        core::mem::replace(&mut self.tokens, TokenStream::new().into_iter().peekable()).collect()
    }
}
