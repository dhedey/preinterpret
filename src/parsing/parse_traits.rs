use crate::internal_prelude::*;

pub(crate) trait HandleParse {
    fn handle_parse_from_stream(
        &self,
        input: InterpretedStream,
        interpreter: &mut Interpreter,
    ) -> Result<()> {
        |input: ParseStream| -> Result<()> { self.handle_parse(input, interpreter) }
            .parse2(input.into_token_stream())
    }

    fn handle_parse(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()>;
}
