use crate::internal_prelude::*;

/// Parses a group { .. } for interpretation
#[derive(Clone)]
pub(crate) struct CommandCodeInput {
    delim_span: DelimSpan,
    inner: InterpretationStream,
}

impl Parse for CommandCodeInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let bracket = syn::braced!(content in input);
        let inner = content.parse_with(bracket.span.span_range())?;
        Ok(Self {
            delim_span: bracket.span,
            inner,
        })
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
    ) -> Result<LoopCondition> {
        match self.inner.interpret_into(interpreter, output) {
            Ok(()) => Ok(LoopCondition::None),
            Err(err) => match interpreter.outstanding_loop_condition() {
                LoopCondition::None => Err(err),
                LoopCondition::Break => Ok(LoopCondition::Break),
                LoopCondition::Continue => Ok(LoopCondition::Continue),
            },
        }
    }
}

impl Interpret for CommandCodeInput {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.inner.interpret_into(interpreter, output)
    }
}
