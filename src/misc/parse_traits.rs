use crate::internal_prelude::*;

pub(crate) trait ContextualParse: Sized {
    type Context;

    fn parse_with_context(input: ParseStream, context: Self::Context) -> ParseResult<Self>;
}

pub(crate) trait Parse: Sized {
    fn parse(input: ParseStream) -> ParseResult<Self>;
}

impl<T: SynParse> Parse for T {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(T::parse(input)?)
    }
}
