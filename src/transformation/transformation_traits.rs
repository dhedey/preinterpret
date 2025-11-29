use crate::internal_prelude::*;

pub(crate) trait HandleTransformation {
    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()>;
}
