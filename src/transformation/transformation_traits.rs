use crate::internal_prelude::*;

pub(crate) trait HandleTransformation {
    /// Handles a transformation by pushing an input stream and calling `handle_transform`.
    ///
    /// This is the primary entry point for transformations that start with an `OutputStream`.
    fn handle_transform_from_stream(
        &self,
        input: OutputStream,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        unsafe {
            // RUST-ANALYZER-SAFETY: ...this isn't generally safe...
            // We should only do this when we know that either the input or parser doesn't require
            // analysis of nested None-delimited groups.
            input.parse_with(|input| {
                interpreter.with_input(input, |interpreter| self.handle_transform(interpreter))
            })
        }
    }

    /// Handles a transformation, reading input from the interpreter's input handler.
    ///
    /// Implementations should use `interpreter.input(span_source)` to access the current
    /// input stream, and `interpreter.input_stack_mut(span_source)` for operations
    /// that need to enter/exit groups.
    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()>;
}
