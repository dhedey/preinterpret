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
// This is where CommandArguments comes in.
//
// Now that I've changed how preinterpret works to be a two-pass approach
// (first parsing, then interpreting), we could consider swapping out the
// CommandArguments to wrap a syn::ParseStream instead.
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
pub(crate) struct CommandArguments<'a> {
    parse_stream: ParseStream<'a>,
    /// The span range of the original stream, before tokens were consumed
    full_span_range: SpanRange,
    /// The span of the last item consumed (or the full span range if no items have been consumed yet)
    latest_item_span_range: SpanRange,
}

impl<'a> CommandArguments<'a> {
    pub(crate) fn new(parse_stream: ParseStream<'a>, span_range: SpanRange) -> Self {
        Self {
            parse_stream,
            full_span_range: span_range,
            latest_item_span_range: span_range,
        }
    }

    pub(crate) fn full_span_range(&self) -> SpanRange {
        self.full_span_range
    }

    /// We use this instead of the "unexpected / drop glue" pattern in order to give a better error message
    pub(crate) fn assert_empty(&self, error_message: &'static str) -> Result<()> {
        if self.parse_stream.is_empty() {
            Ok(())
        } else {
            self.latest_item_span_range.err(error_message)
        }
    }

    pub(crate) fn fully_parse_or_error<T>(
        &self,
        parse_function: impl FnOnce(ParseStream) -> Result<T>,
        error_message: &'static str
    ) -> Result<T> {
        let parsed = parse_function(self.parse_stream)
            .or_else(|_| self.full_span_range.err(error_message))?;

        self.assert_empty(error_message)?;

        Ok(parsed)
    }

    pub(crate) fn parse_all_for_interpretation(&self) -> Result<InterpretationStream> {
        self.parse_stream.parse_all_for_interpretation(self.full_span_range)
    }

    pub(crate) fn read_all_as_raw_token_stream(&self) -> TokenStream {
        self.parse_stream.parse::<TokenStream>().unwrap()
    }
}
