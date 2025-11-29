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
        interpreter: &mut Interpreter,
        error_span_range: &SpanRange,
    ) -> ExecutionResult<()> {
        match self {
            ParseUntil::End => {
                let remaining = interpreter
                    .input(error_span_range)?
                    .parse::<TokenStream>()?;
                interpreter
                    .output(error_span_range)?
                    .extend_raw_tokens(remaining);
            }
            ParseUntil::Group(delimiter) => {
                let (input, output) = interpreter.input_and_output(error_span_range)?;
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
                let (input, output) = interpreter.input_and_output(error_span_range)?;
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
                let (input, output) = interpreter.input_and_output(error_span_range)?;
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
                let (input, output) = interpreter.input_and_output(error_span_range)?;
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
