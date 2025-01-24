use crate::internal_prelude::*;

pub(crate) trait Interpret: Sized {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()>;

    fn interpret_to_new_stream(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<InterpretedStream> {
        let mut output = InterpretedStream::new();
        self.interpret_into(interpreter, &mut output)?;
        Ok(output)
    }
}

pub(crate) trait InterpretValue: Sized {
    type InterpretedValue;

    fn interpret(self, interpreter: &mut Interpreter) -> ExecutionResult<Self::InterpretedValue>;
}

impl<T: InterpretValue<InterpretedValue = I> + HasSpanRange, I: ToTokens> Interpret for T {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        output.extend_with_raw_tokens_from(self.interpret(interpreter)?);
        Ok(())
    }
}

impl<T: ToTokens> InterpretValue for T {
    type InterpretedValue = Self;

    fn interpret(self, _interpreter: &mut Interpreter) -> ExecutionResult<Self::InterpretedValue> {
        Ok(self)
    }
}
