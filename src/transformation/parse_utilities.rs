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

impl Parse<Output> for ParseUntil {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        let next: TokenTree = input.parse()?;
        let until = match next {
            TokenTree::Group(group) => {
                if !group.stream().is_empty() {
                    return group
                        .span()
                        .parse_err(format!(
                            "Until will only read up until the start of the group '{}'. The group must be empty '{}{}' to indicate this.",
                            group.delimiter().description_of_open(),
                            group.delimiter().description_of_open(),
                            group.delimiter().description_of_close(),
                        ));
                }
                ParseUntil::Group(group.delimiter())
            }
            TokenTree::Ident(ident) => ParseUntil::Ident(ident),
            TokenTree::Punct(punct) => ParseUntil::Punct(punct),
            TokenTree::Literal(literal) => ParseUntil::Literal(literal),
        };
        if !input.is_empty() {
            return input.parse_err("Until only takes a single token.");
        }
        Ok(until)
    }
}

impl ParseUntil {
    pub(crate) fn handle_parse_into(
        &self,
        input: OutputParseStream,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            ParseUntil::End => {
                let remaining = input.parse::<TokenStream>()?;
                output.extend_raw_tokens(remaining);
            }
            ParseUntil::Group(delimiter) => {
                while !input.is_empty() {
                    if input.peek_specific_group(*delimiter) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    output.push_raw_token_tree(next);
                }
            }
            ParseUntil::Ident(ident) => {
                let content = ident.to_string();
                while !input.is_empty() {
                    if input.peek_ident_matching(&content) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    output.push_raw_token_tree(next);
                }
            }
            ParseUntil::Punct(punct) => {
                let punct_char = punct.as_char();
                while !input.is_empty() {
                    if input.peek_punct_matching(punct_char) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    output.push_raw_token_tree(next);
                }
            }
            ParseUntil::Literal(literal) => {
                let content = literal.to_string();
                while !input.is_empty() {
                    if input.peek_literal_matching(&content) {
                        return Ok(());
                    }
                    let next = input.parse()?;
                    output.push_raw_token_tree(next);
                }
            }
        }
        Ok(())
    }
}
