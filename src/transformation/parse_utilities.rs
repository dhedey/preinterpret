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
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        match self {
            ParseUntil::End => {
                let remaining = input.parse::<TokenStream>()?;
                interpreter.output()?.extend_raw_tokens(remaining);
            }
            ParseUntil::Group(delimiter) => {
                while !input.is_empty() {
                    if input.peek_specific_group(*delimiter) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    interpreter.output()?.push_raw_token_tree(next);
                }
            }
            ParseUntil::Ident(ident) => {
                let content = ident.to_string();
                while !input.is_empty() {
                    if input.peek_ident_matching(&content) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    interpreter.output()?.push_raw_token_tree(next);
                }
            }
            ParseUntil::Punct(punct) => {
                let punct = punct.as_char();
                while !input.is_empty() {
                    if input.peek_punct_matching(punct) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    interpreter.output()?.push_raw_token_tree(next);
                }
            }
            ParseUntil::Literal(literal) => {
                let content = literal.to_string();
                while !input.is_empty() {
                    if input.peek_literal_matching(&content) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    interpreter.output()?.push_raw_token_tree(next);
                }
            }
        }
        Ok(())
    }
}
