use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct InterpretedStream {
    /// Currently, even ~inside~ macro executions, round-tripping [`Delimiter::None`] groups to a TokenStream
    /// breaks in rust-analyzer. This causes various spurious errors in the IDE.
    /// See: https://github.com/dtolnay/syn/issues/1464 and https://github.com/rust-lang/rust-analyzer/issues/18211
    ///
    /// In future, we may wish to internally use some kind of `TokenBuffer` type, which I've started on below.
    token_stream: TokenStream,
}

impl InterpretedStream {
    pub(crate) fn new() -> Self {
        Self {
            token_stream: TokenStream::new(),
        }
    }

    pub(crate) fn raw(token_stream: TokenStream) -> Self {
        Self { token_stream }
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
        appender: impl FnOnce(&mut Self) -> Result<()>,
        delimiter: Delimiter,
        span: Span,
    ) -> Result<()> {
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
        self.push_raw_token_tree(TokenTree::group(inner_tokens.token_stream, delimiter, span));
    }

    pub(crate) fn extend_raw_tokens(&mut self, tokens: impl ToTokens) {
        tokens.to_tokens(&mut self.token_stream);
    }

    pub(crate) fn extend_raw_token_iter(&mut self, tokens: impl IntoIterator<Item = TokenTree>) {
        self.token_stream.extend(tokens);
    }

    pub(crate) fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
        self.token_stream.extend(iter::once(token_tree));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.token_stream.is_empty()
    }

    #[allow(unused)]
    pub(crate) fn parse_into_fields<T: 'static>(
        self,
        parser: FieldsParseDefinition<T>,
        error_span_range: SpanRange,
    ) -> Result<T> {
        self.syn_parse(parser.create_syn_parser(error_span_range))
    }

    /// For a type `T` which implements `syn::Parse`, you can call this as `syn_parse(self, T::parse)`.
    /// For more complicated parsers, just pass the parsing function to this function.
    pub(crate) fn syn_parse<P: syn::parse::Parser>(self, parser: P) -> Result<P::Output> {
        parser.parse2(self.token_stream)
    }

    pub(crate) fn into_token_stream(self) -> TokenStream {
        self.token_stream
    }

    pub(crate) fn append_cloned_into(&self, output: &mut InterpretedStream) {
        output.token_stream.extend(self.token_stream.clone())
    }

    pub(crate) fn concat_recursive(self) -> String {
        fn concat_recursive_internal(output: &mut String, token_stream: TokenStream) {
            for token_tree in token_stream {
                match token_tree {
                    TokenTree::Literal(literal) => match literal.content_if_string_like() {
                        Some(content) => output.push_str(&content),
                        None => output.push_str(&literal.to_string()),
                    },
                    TokenTree::Group(group) => match group.delimiter() {
                        Delimiter::Parenthesis => {
                            output.push('(');
                            concat_recursive_internal(output, group.stream());
                            output.push(')');
                        }
                        Delimiter::Brace => {
                            output.push('{');
                            concat_recursive_internal(output, group.stream());
                            output.push('}');
                        }
                        Delimiter::Bracket => {
                            output.push('[');
                            concat_recursive_internal(output, group.stream());
                            output.push(']');
                        }
                        Delimiter::None => {
                            concat_recursive_internal(output, group.stream());
                        }
                    },
                    TokenTree::Punct(punct) => {
                        output.push(punct.as_char());
                    }
                    TokenTree::Ident(ident) => output.push_str(&ident.to_string()),
                }
            }
        }

        let mut output = String::new();
        concat_recursive_internal(&mut output, self.into_token_stream());
        output
    }
}

impl From<TokenTree> for InterpretedStream {
    fn from(value: TokenTree) -> Self {
        InterpretedStream {
            token_stream: value.into(),
        }
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
