use crate::internal_prelude::*;

/// A parsed stream ready for interpretation
pub(crate) struct SourceStream {
    items: Vec<SourceItem>,
    span: Span,
}

impl SourceStream {
    pub(crate) fn parse_with_span(input: SourceParser, span: Span) -> ParseResult<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items, span })
    }

    pub(crate) fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        for item in self.items.iter_mut() {
            item.control_flow_pass(context)?;
        }
        Ok(())
    }
}

impl Interpret for SourceStream {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        for item in self.items.iter() {
            item.interpret(interpreter)?;
        }
        Ok(())
    }
}

impl HasSpan for SourceStream {
    fn span(&self) -> Span {
        self.span
    }
}

pub(crate) enum SourceItem {
    Variable(EmbeddedVariable),
    EmbeddedExpression(EmbeddedExpression),
    EmbeddedStatements(EmbeddedStatements),
    SourceGroup(SourceGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
    StreamLiteral(StreamLiteral),
}

impl ParseSource for SourceItem {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Group(_) => SourceItem::SourceGroup(input.parse()?),
            SourcePeekMatch::EmbeddedVariable => SourceItem::Variable(input.parse()?),
            SourcePeekMatch::EmbeddedExpression => {
                SourceItem::EmbeddedExpression(input.parse()?)
            }
            SourcePeekMatch::EmbeddedStatements => {
                SourceItem::EmbeddedStatements(input.parse()?)
            }
            SourcePeekMatch::Punct(_) => SourceItem::Punct(input.parse_any_punct()?),
            SourcePeekMatch::Ident(_) => SourceItem::Ident(input.parse_any_ident()?),
            SourcePeekMatch::Literal(_) => SourceItem::Literal(input.parse()?),
            SourcePeekMatch::StreamLiteral => SourceItem::StreamLiteral(input.parse().map_err(|err| err.add_context_if_none("If this wasn't intended to be a stream-based literal, replace % with %raw[%]."))?),
            SourcePeekMatch::ObjectLiteral => return input.parse_err("Object literals are only supported in an expression context, not a stream context."),
            SourcePeekMatch::End => return input.parse_err("Expected some item."),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            SourceItem::Variable(variable) => variable.control_flow_pass(context),
            SourceItem::EmbeddedExpression(expr) => expr.control_flow_pass(context),
            SourceItem::EmbeddedStatements(block) => block.control_flow_pass(context),
            SourceItem::SourceGroup(group) => group.control_flow_pass(context),
            SourceItem::Punct(punct) => punct.control_flow_pass(context),
            SourceItem::Ident(ident) => ident.control_flow_pass(context),
            SourceItem::Literal(literal) => literal.control_flow_pass(context),
            SourceItem::StreamLiteral(stream_literal) => stream_literal.control_flow_pass(context),
        }
    }
}

impl Interpret for SourceItem {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        match self {
            SourceItem::Variable(variable) => {
                variable.interpret(interpreter)?;
            }
            SourceItem::EmbeddedExpression(block) => {
                block.interpret(interpreter)?;
            }
            SourceItem::EmbeddedStatements(statements) => {
                statements.interpret(interpreter)?;
            }
            SourceItem::SourceGroup(group) => {
                group.interpret(interpreter)?;
            }
            SourceItem::Punct(punct) => interpreter.output(punct)?.push_punct(punct.clone()),
            SourceItem::Ident(ident) => interpreter.output(ident)?.push_ident(ident.clone()),
            SourceItem::Literal(literal) => {
                interpreter.output(literal)?.push_literal(literal.clone())
            }
            SourceItem::StreamLiteral(stream_literal) => stream_literal.interpret(interpreter)?,
        }
        Ok(())
    }
}

impl HasSpanRange for SourceItem {
    fn span_range(&self) -> SpanRange {
        match self {
            SourceItem::Variable(variable) => variable.span_range(),
            SourceItem::EmbeddedExpression(expr) => expr.span_range(),
            SourceItem::EmbeddedStatements(block) => block.span_range(),
            SourceItem::SourceGroup(group) => group.span_range(),
            SourceItem::Punct(punct) => punct.span_range(),
            SourceItem::Ident(ident) => ident.span_range(),
            SourceItem::Literal(literal) => literal.span_range(),
            SourceItem::StreamLiteral(stream_literal) => stream_literal.span_range(),
        }
    }
}

/// A parsed group ready for interpretation
pub(crate) struct SourceGroup {
    source_delimiter: Delimiter,
    source_delim_span: DelimSpan,
    content: SourceStream,
}

impl ParseSource for SourceGroup {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_group()?;
        let content = SourceStream::parse_with_span(&content, delim_span.join())?;
        Ok(Self {
            source_delimiter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl Interpret for SourceGroup {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        interpreter.in_output_group(
            self.source_delimiter,
            self.source_delim_span.join(),
            |interpreter| self.content.interpret(interpreter),
        )
    }
}

impl HasSpan for SourceGroup {
    fn span(&self) -> Span {
        self.source_delim_span.join()
    }
}
