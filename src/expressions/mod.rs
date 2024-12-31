mod boolean;
mod evaluation_tree;
mod float;
mod integer;
mod operations;
mod value;

use crate::internal_prelude::*;
use boolean::*;
use evaluation_tree::*;
use float::*;
use integer::*;
use operations::*;
use value::*;

#[allow(unused)]
pub(crate) enum ExpressionParsingMode {
    Standard,
    BeforeCurlyBraces,
    StartOfStatement,
}

pub(crate) fn evaluate_expression(
    token_stream: TokenStream,
    mode: ExpressionParsingMode,
) -> Result<EvaluationOutput> {
    use syn::parse::{Parse, Parser};

    // Parsing into a rust expression is overkill here.
    // In future we could choose to implement a subset of the grammar which we actually can use/need.
    //
    // That said, it's useful for now for two reasons:
    // * Aligning with rust syntax
    // * Saving implementation work, particularly given that syn is likely pulled into most code bases anyway
    let expression = match mode {
        ExpressionParsingMode::Standard => Expr::parse.parse2(token_stream)?,
        ExpressionParsingMode::BeforeCurlyBraces => {
            (Expr::parse_without_eager_brace).parse2(token_stream)?
        }
        ExpressionParsingMode::StartOfStatement => {
            (Expr::parse_with_earlier_boundary_rule).parse2(token_stream)?
        }
    };

    EvaluationTree::build_from(&expression)?.evaluate()
}
