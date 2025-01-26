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

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::InterpretedValue>;
}

impl<T: ToTokens> InterpretValue for T {
    type InterpretedValue = Self;

    fn interpret_to_value(
        self,
        _interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::InterpretedValue> {
        Ok(self)
    }
}
