use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct InterpretationBracketedGroup {
    #[allow(unused)]
    pub(crate) brackets: token::Bracket,
    pub(crate) interpretation_stream: InterpretationStream,
}

impl HasSpanRange for InterpretationBracketedGroup {
    fn span_range(&self) -> SpanRange {
        self.brackets.span.span_range()
    }
}

impl Parse for InterpretationBracketedGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brackets = syn::bracketed!(content in input);
        let interpretation_stream = content.parse_with(brackets.span.span_range())?;
        Ok(Self {
            brackets,
            interpretation_stream,
        })
    }
}

impl InterpretValue for InterpretationBracketedGroup {
    type InterpretedValue = InterpretedBracketedGroup;

    fn interpret(self, interpreter: &mut Interpreter) -> Result<Self::InterpretedValue> {
        Ok(InterpretedBracketedGroup {
            brackets: self.brackets,
            interpreted_stream: self
                .interpretation_stream
                .interpret_as_tokens(interpreter)?,
        })
    }
}

/// Parses a [..] block.
pub(crate) struct InterpretedBracketedGroup {
    #[allow(unused)]
    pub(crate) brackets: token::Bracket,
    pub(crate) interpreted_stream: InterpretedStream,
}

impl syn::parse::Parse for InterpretedBracketedGroup {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        let brackets = syn::bracketed!(content in input);
        let interpreted_stream =
            InterpretedStream::raw(brackets.span.span_range(), content.parse()?);
        Ok(Self {
            brackets,
            interpreted_stream,
        })
    }
}
