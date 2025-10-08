use crate::internal_prelude::*;

pub(crate) trait Interpret: Sized {
    fn interpret_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;

    fn interpret_to_new_stream(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OutputStream> {
        let mut output = OutputStream::new();
        self.interpret_into(interpreter, &mut output)?;
        Ok(output)
    }
}

pub(crate) trait Evaluate: Sized {
    type OutputValue;

    fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<Self::OutputValue>;
}
