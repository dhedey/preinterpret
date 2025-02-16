use crate::internal_prelude::*;

/// A group `{ ... }` representing code which can be interpreted
#[derive(Clone)]
pub(crate) struct SourceCodeBlock {
    braces: Braces,
    inner: SourceStream,
}

impl Parse<Source> for SourceCodeBlock {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (braces, content) = input.parse_braces()?;
        let inner = content.parse_with_context(braces.join())?;
        Ok(Self { braces, inner })
    }
}

impl HasSpan for SourceCodeBlock {
    fn span(&self) -> Span {
        self.braces.span()
    }
}

impl SourceCodeBlock {
    pub(crate) fn interpret_loop_content_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<Option<ControlFlowInterrupt>> {
        match self.inner.interpret_into(interpreter, output) {
            Ok(()) => Ok(None),
            Err(ExecutionInterrupt::ControlFlow(control_flow_interrupt, _)) => {
                Ok(Some(control_flow_interrupt))
            }
            Err(error) => Err(error),
        }
    }
}

impl Interpret for SourceCodeBlock {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.inner.interpret_into(interpreter, output)
    }
}
