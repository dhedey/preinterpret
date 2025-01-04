use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionStream {
    interpreted_stream: InterpretedStream,
}

impl ExpressionStream {
    pub(crate) fn new() -> Self {
        Self {
            interpreted_stream: InterpretedStream::new(),
        }
    }

    pub(crate) fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
        self.interpreted_stream.push_raw_token_tree(token_tree);
    }

    pub(crate) fn push_interpreted_group(&mut self, contents: InterpretedStream, span_range: SpanRange) {
        self.interpreted_stream.push_new_group(contents, Delimiter::None, span_range);
    }

    pub(crate) fn push_expression_group(&mut self, contents: Self, delimiter: Delimiter, span_range: SpanRange) {
        self.interpreted_stream.push_new_group(contents.interpreted_stream, delimiter, span_range);
    }

    pub(crate) fn evaluate(self) -> Result<EvaluationOutput> {
        use syn::parse::{Parse, Parser};

        // Parsing into a rust expression is overkill here.
        //
        // In future we could choose to implement a subset of the grammar which we actually can use/need.
        //
        // That said, it's useful for now for two reasons:
        // * Aligning with rust syntax
        // * Saving implementation work, particularly given that syn is likely pulled into most code bases anyway
        //
        // Because of the kind of expressions we're parsing (i.e. no {} allowed),
        // we can get by with parsing it as `Expr::parse` rather than with
        // `Expr::parse_without_eager_brace` or `Expr::parse_with_earlier_boundary_rule`.
        let expression = Expr::parse.parse2(self.interpreted_stream.into_token_stream())?;

        EvaluationTree::build_from(&expression)?
            .evaluate()
    }
}
