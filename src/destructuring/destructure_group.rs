use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct DestructureGroup {
    delimiter: Delimiter,
    inner: DestructureRemaining,
}

impl Parse for DestructureGroup {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (delimiter, _, content) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            inner: content.parse()?,
        })
    }
}

impl HandleDestructure for DestructureGroup {
    fn handle_destructure(
        &self,
        input: ParseStream,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        let (_, inner) = input.parse_specific_group(self.delimiter)?;
        self.inner.handle_destructure(&inner, interpreter)
    }
}
