use crate::internal_prelude::*;

pub(crate) trait Interpret: Sized {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()>;
}
