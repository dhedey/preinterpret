use crate::internal_prelude::*;

pub(crate) trait OutputToStream: Sized {
    fn output_to_stream(&self, output: &mut OutputInterpreter) -> ExecutionResult<()>;
}

/// A wrapper around `Interpreter` that is intended to make outputting to the
/// top-most output stream more convenient, performant and less error-prone.
pub(crate) struct OutputInterpreter<'a> {
    interpreter: &'a mut Interpreter,
}

impl<'a> OutputInterpreter<'a> {
    /// Should only be used with a new Interpreter
    pub(crate) fn new_unchecked(interpreter: &'a mut Interpreter) -> Self {
        Self { interpreter }
    }

    pub(crate) fn new_checked(
        interpreter: &'a mut Interpreter,
        span: &impl HasSpanRange,
    ) -> ExecutionResult<Self> {
        // Validate that output is not frozen, then drop the temporary reference
        let _ = interpreter.output(span)?;
        Ok(Self { interpreter })
    }

    /// Creates a new `OutputInterpreter` by reborrowing from an existing one.
    pub(crate) fn reborrow(&mut self) -> OutputInterpreter<'_> {
        OutputInterpreter {
            interpreter: self.interpreter,
        }
    }

    /// Executes `f` with mutable access to the underlying `Interpreter`.
    ///
    /// The executed code must not change the height of the output stack or
    /// leave the output in a frozen state — i.e. it should evaluate expressions
    /// or resolve variables, not manipulate output buffers directly.
    pub(crate) fn with_interpreter<R, E>(
        &mut self,
        f: impl FnOnce(&mut Interpreter) -> Result<R, E>,
    ) -> Result<R, E> {
        let height_before = self.interpreter.output_stack_height();
        let result = f(self.interpreter);
        assert_eq!(
            self.interpreter.output_stack_height(),
            height_before,
            "OutputInterpreter::with_interpreter: closure must not change the output stack height"
        );
        result
    }

    pub(crate) fn capture_output<E>(
        &mut self,
        f: impl FnOnce(&mut OutputInterpreter) -> Result<(), E>,
    ) -> Result<OutputStream, E> {
        self.interpreter.capture_output(f)
    }

    pub(crate) fn in_output_group<E>(
        &mut self,
        delimiter: Delimiter,
        span: Span,
        f: impl FnOnce(&mut OutputInterpreter) -> Result<(), E>,
    ) -> Result<(), E> {
        self.interpreter.in_output_group(delimiter, span, f)
    }
}

impl Deref for OutputInterpreter<'_> {
    type Target = OutputStream;

    fn deref(&self) -> &Self::Target {
        self.interpreter.current_output_unchecked()
    }
}

impl DerefMut for OutputInterpreter<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.interpreter.current_output_mut_unchecked()
    }
}
