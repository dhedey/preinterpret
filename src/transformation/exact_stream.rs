use crate::internal_prelude::*;

pub(crate) fn handle_parsing_exact_output_match(
    input: ParseStream<Output>,
    interpreter: &mut Interpreter,
    expected: &OutputStream,
    output: &mut OutputStream,
) -> ExecutionResult<()> {
    for item in expected.iter() {
        match item {
            OutputTokenTreeRef::TokenTree(token_tree) => {
                handle_parsing_exact_token_tree(input, interpreter, token_tree, output)?;
            }
            OutputTokenTreeRef::OutputGroup(delimiter, _, inner_expected) => {
                handle_parsing_exact_group(
                    input,
                    delimiter,
                    |inner_input, inner_output| {
                        handle_parsing_exact_output_match(
                            inner_input,
                            interpreter,
                            inner_expected,
                            inner_output,
                        )
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
    interpreter: &mut Interpreter,
    expected: TokenStream,
    output: &mut OutputStream,
) -> ExecutionResult<()> {
    for item in expected.into_iter() {
        handle_parsing_exact_token_tree(input, interpreter, &item, output)?;
    }
    Ok(())
}

fn handle_parsing_exact_token_tree(
    input: ParseStream<Output>,
    interpreter: &mut Interpreter,
    expected: &TokenTree,
    output: &mut OutputStream,
) -> ExecutionResult<()> {
    match expected {
        TokenTree::Group(group) => {
            handle_parsing_exact_group(
                input,
                group.delimiter(),
                |inner_input, inner_output| {
                    handle_parsing_exact_stream_match(
                        inner_input,
                        interpreter,
                        group.stream(),
                        inner_output,
                    )
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
    parse_inner: impl FnOnce(ParseStream<Output>, &mut OutputStream) -> ExecutionResult<()>,
    output: &mut OutputStream,
) -> ExecutionResult<()> {
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
