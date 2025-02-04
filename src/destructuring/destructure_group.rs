use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct DestructureGroup {
    delimiter: Delimiter,
    inner: DestructureRemaining,
}

impl Parse<Source> for DestructureGroup {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
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
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        let (_, inner) = input.parse_specific_group(self.delimiter)?;
        self.inner.handle_destructure(&inner, interpreter)
    }
}
