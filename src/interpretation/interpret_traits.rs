use crate::internal_prelude::*;

// This trait isn't so important any more. It can probably be removed in future.
// It is typically used for things which output directly to the interpreter's output
// stream, rather than returning values.
pub(crate) trait Interpret: Sized {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()>;
}
