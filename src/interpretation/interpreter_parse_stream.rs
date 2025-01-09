use crate::internal_prelude::*;

// ===============================================
// How syn features fits with preinterpret parsing
// ===============================================
//
// I spent quite a while considering whether this could be wrapping a
// `syn::parse::ParseBuffer<'a>` or `syn::buffer::Cursor<'a>`, instead
// of a custom Peekable<TokenIter>
//
// I came to the conclusion it doesn't make much sense for now, but
// could be explored in future:
//
// * ParseBuffer / ParseStream is powerful but restrictive
//   > Due to how it works, we'd need to use it to parse the initial input
//     in a single initial pass.
//   > We currently have an Interpreter with us as we parse, which the `Parse`
//     trait doesn't allow.
//   > We could consider splitting into a two-pass approach, where we start
//     with a Parse step, and then we interpret after, but it would be quite
//     a big change internally
// * Cursor needs to reference into some TokenBuffer
//   > We would convert the input TokenStream into a TokenBuffer and
//     Cursor into that
//   > But this can't be converted into a ParseBuffer outside of the syn crate,
//     and so it doesn't get much benefit
//
// Either of these approaches appear disjoint from the parse/destructuring
// operations...
//
// * For parse/destructuring operations, we may need to temporarily
//   create parse streams in scope of a command execution
// * For #VARIABLES which can be incrementally consumed / parsed,
//   it would be nice to be able to store a Cursor into a TokenBuffer,
//   but annoyingly it isn't possible to convert this to a ParseStream
//   outside of syn.

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

    pub(super) fn peek_token_tree(&mut self) -> Option<&TokenTree> {
        self.tokens.peek()
    }

    pub(super) fn next_token_tree_or_end(&mut self) -> Option<TokenTree> {
        self.tokens.next()
    }

    fn next_item_or_end(&mut self) -> Result<Option<NextItem>> {
        let next_item = NextItem::parse(self)?;
        Ok(match next_item {
            Some(next_item) => {
                self.latest_item_span_range = next_item.span_range();
                Some(next_item)
            },
            None => None,
        })
    }

    pub(crate) fn next_item(&mut self, error_message: &'static str) -> Result<NextItem> {
        match self.next_item_or_end()? {
            Some(item) => Ok(item),
            None => self.latest_item_span_range.err(format!("Unexpected end: {error_message}")),
        }
    }

    pub(crate) fn next_as_ident(&mut self, error_message: &'static str) -> Result<Ident> {
        match self.next_item(error_message)? {
            NextItem::Ident(ident) => Ok(ident),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_ident_matching(&mut self, ident_name: &str, error_message: &'static str) -> Result<Ident> {
        match self.next_item(error_message)? {
            NextItem::Ident(ident) if &ident.to_string() == ident_name => Ok(ident),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_punct(&mut self, error_message: &'static str) -> Result<Punct> {
        match self.next_item(error_message)? {
            NextItem::Punct(punct) => Ok(punct),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_punct_matching(&mut self, char: char, error_message: &'static str) -> Result<Punct> {
        match self.next_item(error_message)? {
            NextItem::Punct(punct) if punct.as_char() == char => Ok(punct),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_kinded_group(&mut self, delimiter: Delimiter, error_message: &'static str) -> Result<InterpretationGroup> {
        match self.next_item(error_message)? {
            NextItem::Group(group) if group.delimiter() == delimiter => Ok(group),
            other => other.err(error_message),
        }
    }

    pub(crate) fn next_as_variable(
        &mut self,
        error_message: &'static str,
    ) -> Result<Variable> {
        match self.next_item(error_message)? {
            NextItem::Variable(variable_substitution) => Ok(variable_substitution),
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
