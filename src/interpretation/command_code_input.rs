use crate::internal_prelude::*;

/// Parses a group { .. } for interpretation
#[derive(Clone)]
pub(crate) struct CommandCodeInput {
    delim_span: DelimSpan,
    inner: InterpretationStream,
}

impl Parse for CommandCodeInput {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (delim_span, content) = input.parse_group_matching(Delimiter::Brace)?;
        let inner = content.parse_with(delim_span.join().span_range())?;
        Ok(Self { delim_span, inner })
    }
}

impl HasSpanRange for CommandCodeInput {
    fn span_range(&self) -> SpanRange {
        self.delim_span.span_range()
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
