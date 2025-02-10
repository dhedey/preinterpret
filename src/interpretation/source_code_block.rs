use crate::internal_prelude::*;

/// A group `{ ... }` representing code which can be interpreted
#[derive(Clone)]
pub(crate) struct SourceCodeBlock {
    delim_span: DelimSpan,
    inner: SourceStream,
}

impl Parse<Source> for SourceCodeBlock {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delim_span, content) = input.parse_specific_group(Delimiter::Brace)?;
        let inner = content.parse_with_context(delim_span.join())?;
        Ok(Self { delim_span, inner })
    }
}

impl HasSpan for SourceCodeBlock {
    fn span(&self) -> Span {
        self.delim_span.join()
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
