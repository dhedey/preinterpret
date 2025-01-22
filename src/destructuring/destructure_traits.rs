use crate::internal_prelude::*;

pub(crate) trait HandleDestructure {
    fn handle_destructure_from_stream(
        &self,
        input: InterpretedStream,
        interpreter: &mut Interpreter,
    ) -> Result<()> {
        unsafe {
            // RUST-ANALYZER-SAFETY: ...this isn't generally safe...
            // We should only do this when we know that either the input or parser doesn't require
            // analysis of nested None-delimited groups.
            input.syn_parse(|input: ParseStream| -> Result<()> {
                self.handle_destructure(input, interpreter)
            })
        }
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()>;
}
