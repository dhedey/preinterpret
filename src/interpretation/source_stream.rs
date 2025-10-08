use crate::internal_prelude::*;

/// A parsed stream ready for interpretation
#[derive(Clone)]
pub(crate) struct SourceStream {
    items: Vec<SourceItem>,
    span: Span,
}

impl SourceStream {
    pub(crate) fn parse_with_span(input: ParseStream<Source>, span: Span) -> ParseResult<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items, span })
    }
}

impl Interpret for SourceStream {
    fn interpret_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        for item in self.items.iter() {
            item.interpret_into(interpreter, output)?;
        }
        Ok(())
    }
}

impl HasSpan for SourceStream {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone)]
pub(crate) enum SourceItem {
    Command(Command),
    Variable(EmbeddedVariable),
    EmbeddedExpression(EmbeddedExpression),
    SourceGroup(SourceGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
    StreamLiteral(StreamLiteral),
}

impl Parse<Source> for SourceItem {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => SourceItem::Command(input.parse()?),
            SourcePeekMatch::Group(_) => SourceItem::SourceGroup(input.parse()?),
            SourcePeekMatch::EmbeddedVariable => SourceItem::Variable(input.parse()?),
            SourcePeekMatch::EmbeddedExpression => {
                SourceItem::EmbeddedExpression(input.parse()?)
            }
            SourcePeekMatch::ExplicitTransformStream | SourcePeekMatch::Transformer(_) => {
                return input.parse_err("Destructurings are not supported here. If this wasn't intended to be a destructuring, replace @ with %raw[@]");
            }
            SourcePeekMatch::Punct(_) => SourceItem::Punct(input.parse_any_punct()?),
            SourcePeekMatch::Ident(_) => SourceItem::Ident(input.parse_any_ident()?),
            SourcePeekMatch::Literal(_) => SourceItem::Literal(input.parse()?),
            SourcePeekMatch::StreamLiteral(_) => SourceItem::StreamLiteral(input.parse()?),
            SourcePeekMatch::ObjectLiteral => return input.parse_err("Object literals are only supported in an expression context, not a stream context."),
            SourcePeekMatch::End => return input.parse_err("Expected some item."),
        })
    }
}

impl Interpret for SourceItem {
    fn interpret_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            SourceItem::Command(command_invocation) => {
                command_invocation.interpret_into(interpreter, output)?;
            }
            SourceItem::Variable(variable) => {
                variable.interpret_into(interpreter, output)?;
            }
            SourceItem::EmbeddedExpression(block) => {
                block.interpret_into(interpreter, output)?;
            }
            SourceItem::SourceGroup(group) => {
                group.interpret_into(interpreter, output)?;
            }
            SourceItem::Punct(punct) => output.push_punct(punct.clone()),
            SourceItem::Ident(ident) => output.push_ident(ident.clone()),
            SourceItem::Literal(literal) => output.push_literal(literal.clone()),
            SourceItem::StreamLiteral(stream_literal) => {
                stream_literal.interpret_into(interpreter, output)?
            }
        }
        Ok(())
    }
}

impl HasSpanRange for SourceItem {
    fn span_range(&self) -> SpanRange {
        match self {
            SourceItem::Command(command_invocation) => command_invocation.span_range(),
            SourceItem::Variable(variable) => variable.span_range(),
            SourceItem::EmbeddedExpression(block) => block.span_range(),
            SourceItem::SourceGroup(group) => group.span_range(),
            SourceItem::Punct(punct) => punct.span_range(),
            SourceItem::Ident(ident) => ident.span_range(),
            SourceItem::Literal(literal) => literal.span_range(),
            SourceItem::StreamLiteral(stream_literal) => stream_literal.span_range(),
        }
    }
}

/// A parsed group ready for interpretation
#[derive(Clone)]
pub(crate) struct SourceGroup {
    source_delimiter: Delimiter,
    source_delim_span: DelimSpan,
    content: SourceStream,
}

impl Parse<Source> for SourceGroup {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_group()?;
        let content = SourceStream::parse_with_span(&content, delim_span.join())?;
        Ok(Self {
            source_delimiter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }
}

impl Interpret for SourceGroup {
    fn interpret_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let inner = self.content.interpret_to_new_stream(interpreter)?;
        output.push_new_group(inner, self.source_delimiter, self.source_delim_span.join());
        Ok(())
    }
}

impl HasSpan for SourceGroup {
    fn span(&self) -> Span {
        self.source_delim_span.join()
    }
}
