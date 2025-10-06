use crate::internal_prelude::*;

/// Designed to automatically discover (via peeking) the next token, to limit the extent of a
/// match such as `@REST`.
#[derive(Clone)]
pub(crate) enum ParseUntil {
    End,
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl ParseUntil {
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
