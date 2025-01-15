use crate::internal_prelude::*;

/// Parses a [..] block.
pub(crate) struct BracketedTokenStream {
    #[allow(unused)]
    pub(crate) brackets: token::Bracket,
    pub(crate) token_stream: TokenStream,
}

impl syn::parse::Parse for BracketedTokenStream {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            brackets: syn::bracketed!(content in input),
            token_stream: content.parse()?,
        })
    }
}
