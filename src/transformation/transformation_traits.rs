use crate::internal_prelude::*;

pub(crate) trait HandleTransformation {
    fn handle_transform_from_stream(
        &self,
        input: OutputStream,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        unsafe {
            // RUST-ANALYZER-SAFETY: ...this isn't generally safe...
            // We should only do this when we know that either the input or parser doesn't require
            // analysis of nested None-delimited groups.
            input.parse_with(|input| self.handle_transform(input, interpreter, output))
        }
    }

    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;
}
