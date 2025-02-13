use crate::internal_prelude::*;

/// A parsed stream ready for interpretation
#[derive(Clone)]
pub(crate) struct SourceStream {
    items: Vec<SourceItem>,
    span: Span,
}

impl ContextualParse<Source> for SourceStream {
    type Context = Span;

    fn parse(input: ParseStream<Source>, span: Self::Context) -> ParseResult<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items, span })
    }
}

impl Interpret for SourceStream {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        for item in self.items {
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
    Variable(MarkedVariable),
    ExpressionBlock(ExpressionBlock),
    SourceGroup(SourceGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl Parse<Source> for SourceItem {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => SourceItem::Command(input.parse()?),
            SourcePeekMatch::Group(_) => SourceItem::SourceGroup(input.parse()?),
            SourcePeekMatch::Variable(_) => SourceItem::Variable(input.parse()?),
            SourcePeekMatch::ExpressionBlock(_) => SourceItem::ExpressionBlock(input.parse()?),
            SourcePeekMatch::ExplicitTransformStream | SourcePeekMatch::Transformer(_) => {
                return input.parse_err("Destructurings are not supported here. If this wasn't intended to be a destructuring, replace @ with [!raw! @]");
            }
            SourcePeekMatch::AppendVariableBinding => {
                return input.parse_err("Destructurings are not supported here.");
            }
            SourcePeekMatch::Punct(_) => SourceItem::Punct(input.parse_any_punct()?),
            SourcePeekMatch::Ident(_) => SourceItem::Ident(input.parse_any_ident()?),
            SourcePeekMatch::Literal(_) => SourceItem::Literal(input.parse()?),
            SourcePeekMatch::End => return input.parse_err("Expected some item."),
        })
    }
}

impl Interpret for SourceItem {
    fn interpret_into(
        self,
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
            SourceItem::ExpressionBlock(block) => {
                block.interpret_into(interpreter, output)?;
            }
            SourceItem::SourceGroup(group) => {
                group.interpret_into(interpreter, output)?;
            }
            SourceItem::Punct(punct) => output.push_punct(punct),
            SourceItem::Ident(ident) => output.push_ident(ident),
            SourceItem::Literal(literal) => output.push_literal(literal),
        }
        Ok(())
    }
}

impl HasSpanRange for SourceItem {
    fn span_range(&self) -> SpanRange {
        match self {
            SourceItem::Command(command_invocation) => command_invocation.span_range(),
            SourceItem::Variable(variable) => variable.span_range(),
            SourceItem::ExpressionBlock(block) => block.span_range(),
            SourceItem::SourceGroup(group) => group.span_range(),
            SourceItem::Punct(punct) => punct.span_range(),
            SourceItem::Ident(ident) => ident.span_range(),
            SourceItem::Literal(literal) => literal.span_range(),
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
        let content = content.parse_with_context(delim_span.join())?;
        Ok(Self {
            source_delimiter: delimiter,
            source_delim_span: delim_span,
            content,
        })
    }
}

impl Interpret for SourceGroup {
    fn interpret_into(
        self,
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
