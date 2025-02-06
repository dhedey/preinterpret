use crate::internal_prelude::*;

/// Designed to instruct the parser to stop parsing when a certain condition is met
pub(crate) trait StopCondition<K>: Clone {
    fn should_stop(input: ParseStream<K>) -> bool;
}

#[derive(Clone)]
pub(crate) struct UntilEnd;
impl<K> StopCondition<K> for UntilEnd {
    fn should_stop(input: ParseStream<K>) -> bool {
        input.is_empty()
    }
}

pub(crate) trait PeekableToken<K>: Clone {
    fn peek(input: ParseStream<K>) -> bool;
}

// This is going through such pain to ensure we stay in the public API of syn
// We'd like to be able to use `input.peek::<T>()` for some syn::token::Token
// but that's just not an API they support for some reason
macro_rules! impl_peekable_token {
    ($(Token![$token:tt]),* $(,)?) => {
        $(
            impl<K> PeekableToken<K> for Token![$token] {
                fn peek(input: ParseStream<K>) -> bool {
                    input.peek(Token![$token])
                }
            }
        )*
    };
}

impl_peekable_token! {
    Token![=],
    Token![in],
}

#[derive(Clone)]
pub(crate) struct UntilToken<T> {
    token: PhantomData<T>,
}

impl<T: PeekableToken<K>, K> StopCondition<K> for UntilToken<T> {
    fn should_stop(input: ParseStream<K>) -> bool {
        input.is_empty() || T::peek(input)
    }
}

/// Designed to automatically discover (via peeking) the next token, to limit the extent of a
/// match such as `#..x`.
#[derive(Clone)]
pub(crate) enum ParseUntil {
    End,
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl ParseUntil {
    /// Peeks the next token, to discover what we should parse next
    pub(crate) fn peek_flatten_limit<C: StopCondition<Source>>(
        input: ParseStream<Source>,
    ) -> ParseResult<ParseUntil> {
        if C::should_stop(input) {
            return Ok(ParseUntil::End);
        }
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_)
            | SourcePeekMatch::GroupedVariable
            | SourcePeekMatch::FlattenedVariable
            | SourcePeekMatch::Transformer(_)
            | SourcePeekMatch::ExplicitTransformStream
            | SourcePeekMatch::AppendVariableBinding => {
                return input
                    .span()
                    .parse_err("This cannot follow a flattened variable binding");
            }
            SourcePeekMatch::Group(delimiter) => ParseUntil::Group(delimiter),
            SourcePeekMatch::Ident(ident) => ParseUntil::Ident(ident),
            SourcePeekMatch::Literal(literal) => ParseUntil::Literal(literal),
            SourcePeekMatch::Punct(punct) => ParseUntil::Punct(punct),
            SourcePeekMatch::End => ParseUntil::End,
        })
    }

    pub(crate) fn handle_parse_into(
        &self,
        input: ParseStream<Output>,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            ParseUntil::End => output.extend_raw_tokens(input.parse::<TokenStream>()?),
            ParseUntil::Group(delimiter) => {
                while !input.is_empty() {
                    if input.peek_specific_group(*delimiter) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
            ParseUntil::Ident(ident) => {
                let content = ident.to_string();
                while !input.is_empty() {
                    if input.peek_ident_matching(&content) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
            ParseUntil::Punct(punct) => {
                let punct = punct.as_char();
                while !input.is_empty() {
                    if input.peek_punct_matching(punct) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
            ParseUntil::Literal(literal) => {
                let content = literal.to_string();
                while !input.is_empty() {
                    if input.peek_literal_matching(&content) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
        }
        Ok(())
    }
}
