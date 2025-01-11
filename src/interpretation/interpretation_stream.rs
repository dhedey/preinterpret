use crate::internal_prelude::*;

/// A parsed stream ready for interpretation
#[derive(Clone)]
pub(crate) struct InterpretationStream {
    items: Vec<NextItem>,
    span_range: SpanRange,
}

impl InterpretationStream {
    pub(crate) fn parse_from_token_stream(
        token_stream: TokenStream,
        span_range: SpanRange,
    ) -> Result<Self> {
        InterpreterParseStream::new(token_stream, span_range).parse_all_for_interpretation()
    }

    pub(crate) fn parse(
        parse_stream: &mut InterpreterParseStream,
        span_range: SpanRange,
    ) -> Result<Self> {
        let mut items = Vec::new();
        while let Some(next_item) = NextItem::parse(parse_stream)? {
            items.push(next_item);
        }
        Ok(Self { items, span_range })
    }
}

impl Interpret for InterpretationStream {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        for item in self.items {
            item.interpret_as_tokens_into(interpreter, output)?;
        }
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        let mut inner_expression_stream = ExpressionStream::new(self.span_range);
        for item in self.items {
            item.interpret_as_expression_into(interpreter, &mut inner_expression_stream)?;
        }
        expression_stream.push_expression_group(
            inner_expression_stream,
            Delimiter::None,
            self.span_range,
        );
        Ok(())
    }
}

impl HasSpanRange for InterpretationStream {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

/// A parsed group ready for interpretation
#[derive(Clone)]
pub(crate) struct InterpretationGroup {
    source_group: Group,
    interpretation_stream: InterpretationStream,
}

impl InterpretationGroup {
    pub(super) fn parse(source_group: Group) -> Result<Self> {
        let interpretation_stream =
            InterpreterParseStream::new(source_group.stream(), source_group.span_range())
                .parse_all_for_interpretation()?;
        Ok(Self {
            source_group,
            interpretation_stream,
        })
    }

    pub(crate) fn delimiter(&self) -> Delimiter {
        self.source_group.delimiter()
    }

    pub(crate) fn into_inner_stream(self) -> InterpretationStream {
        self.interpretation_stream
    }
}

impl Interpret for InterpretationGroup {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.push_new_group(
            self.interpretation_stream
                .interpret_as_tokens(interpreter)?,
            self.source_group.delimiter(),
            self.source_group.span_range(),
        );
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        expression_stream.push_expression_group(
            self.interpretation_stream
                .interpret_as_expression(interpreter)?,
            self.source_group.delimiter(),
            self.source_group.span_range(),
        );
        Ok(())
    }
}

impl HasSpanRange for InterpretationGroup {
    fn span_range(&self) -> SpanRange {
        self.source_group.span_range()
    }
}
