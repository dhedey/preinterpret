use crate::internal_prelude::*;

/// Parses a group { .. } for interpretation
#[derive(Clone)]
pub(crate) struct CommandCodeInput {
    delim_span: DelimSpan,
    inner: InterpretationStream,
}

impl Parse for CommandCodeInput {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (delim_span, content) = input.parse_specific_group(Delimiter::Brace)?;
        let inner = content.parse_with(delim_span.join())?;
        Ok(Self { delim_span, inner })
    }
}

impl HasSpan for CommandCodeInput {
    fn span(&self) -> Span {
        self.delim_span.join()
    }
}

impl CommandCodeInput {
    pub(crate) fn interpret_loop_content_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
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

impl Interpret for CommandCodeInput {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        self.inner.interpret_into(interpreter, output)
    }
}
