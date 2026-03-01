use crate::internal_prelude::*;

/// Designed to automatically discover (via peeking) the next token, to limit the extent of a
/// match such as `rest()`.
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
    ) -> FunctionResult<()> {
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

pub(crate) fn handle_parsing_exact_output_match(
    input: ParseStream<Output>,
    expected: &OutputStream,
    output: &mut OutputStream,
) -> FunctionResult<()> {
    for item in expected.iter() {
        match item {
            OutputTokenTreeRef::TokenTree(token_tree) => {
                handle_parsing_exact_token_tree(input, token_tree, output)?;
            }
            OutputTokenTreeRef::OutputGroup(delimiter, _, inner_expected) => {
                handle_parsing_exact_group(
                    input,
                    delimiter,
                    |inner_input, inner_output| {
                        handle_parsing_exact_output_match(inner_input, inner_expected, inner_output)
                    },
                    output,
                )?;
            }
        }
    }
    Ok(())
}

fn handle_parsing_exact_stream_match(
    input: ParseStream<Output>,
    expected: TokenStream,
    output: &mut OutputStream,
) -> FunctionResult<()> {
    for item in expected.into_iter() {
        handle_parsing_exact_token_tree(input, &item, output)?;
    }
    Ok(())
}

fn handle_parsing_exact_token_tree(
    input: ParseStream<Output>,
    expected: &TokenTree,
    output: &mut OutputStream,
) -> FunctionResult<()> {
    match expected {
        TokenTree::Group(group) => {
            handle_parsing_exact_group(
                input,
                group.delimiter(),
                |inner_input, inner_output| {
                    handle_parsing_exact_stream_match(inner_input, group.stream(), inner_output)
                },
                output,
            )?;
        }
        TokenTree::Ident(ident) => {
            output.push_ident(input.parse_ident_matching(&ident.to_string())?);
        }
        TokenTree::Punct(punct) => {
            output.push_punct(input.parse_punct_matching(punct.as_char())?);
        }
        TokenTree::Literal(literal) => {
            output.push_literal(input.parse_literal_matching(&literal.to_string())?);
        }
    }
    Ok(())
}

fn handle_parsing_exact_group(
    input: ParseStream<Output>,
    delimiter: Delimiter,
    parse_inner: impl FnOnce(ParseStream<Output>, &mut OutputStream) -> FunctionResult<()>,
    output: &mut OutputStream,
) -> FunctionResult<()> {
    // Because `None` is ignored by Syn at parsing time, we can effectively be most permissive by ignoring them.
    // This removes a bit of a footgun for users.
    // If they really want to check for a None group, they can embed `@[GROUP @[EXACT ...]]` transformer.
    if delimiter == Delimiter::None {
        parse_inner(input, output)
    } else {
        let (source_span, inner_source) = input.parse_specific_group(delimiter)?;
        output.push_grouped(
            |output| parse_inner(&inner_source, output),
            delimiter,
            source_span.join(),
        )
    }
}
