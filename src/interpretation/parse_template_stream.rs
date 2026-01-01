use crate::internal_prelude::*;

/// A stream inside @parser[...] literals/patterns, which reads from the current parser
pub(crate) struct ParseTemplateStream {
    items: Vec<ParseTemplateItem>,
    span: Span,
}

impl ParseTemplateStream {
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

impl ParseTemplateStream {
    pub(crate) fn consume(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        for item in self.items.iter() {
            item.consume(interpreter)?;
        }
        Ok(())
    }
}

impl HasSpan for ParseTemplateStream {
    fn span(&self) -> Span {
        self.span
    }
}

pub(crate) enum ParseTemplateItem {
    EmbeddedStatements(EmbeddedStatements),
    Group(ParseTemplateGroup),
    Punct(char),
    Ident(String),
    Literal(String),
}

impl ParseSource for ParseTemplateItem {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Group(_) => ParseTemplateItem::Group(input.parse()?),
            SourcePeekMatch::EmbeddedVariable => return input.parse_err("Variables cannot be embedded into a parse template stream. If you intend to parse the content of the variable, use { parser.read(%[#variable]); }. If you intend to parse a #, use { parser.read(%[#]); }"),
            SourcePeekMatch::EmbeddedExpression => return input.parse_err("Expressions cannot be embedded into a parse template stream. If you intend to parse the content of a value, use { parser.read(%[#variable]); }. If you intend to parse a #, use { parser.read(%[#]); }"),
            SourcePeekMatch::EmbeddedStatements => {
                ParseTemplateItem::EmbeddedStatements(input.parse()?)
            }
            SourcePeekMatch::Punct(_) => {
                let punct = input.parse_any_punct()?;
                ParseTemplateItem::Punct(punct.as_char())
            }
            SourcePeekMatch::Literal(_) => {
                let literal: Literal = input.parse()?;
                ParseTemplateItem::Literal(literal.to_string())
            }
            SourcePeekMatch::Ident(_) => {
                let ident = input.parse_any_ident()?;
                ParseTemplateItem::Ident(ident.to_string())
            }
            SourcePeekMatch::StreamLiteral => return input.parse_err("Stream literals cannot be embedded into a parse template stream. Use { parser.read(%[...]); } to parse the content of a stream."),
            SourcePeekMatch::ObjectLiteral => return input.parse_err("Object literals cannot be embedded into a parse template stream."),
            SourcePeekMatch::End => return input.parse_err("Expected some item."),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            ParseTemplateItem::EmbeddedStatements(block) => block.control_flow_pass(context),
            ParseTemplateItem::Group(group) => group.control_flow_pass(context),
            ParseTemplateItem::Punct { .. } => Ok(()),
            ParseTemplateItem::Ident { .. } => Ok(()),
            ParseTemplateItem::Literal { .. } => Ok(()),
        }
    }
}

impl ParseTemplateItem {
    fn consume(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        match self {
            ParseTemplateItem::EmbeddedStatements(block) => {
                block.consume(interpreter)?;
            }
            ParseTemplateItem::Group(group) => {
                group.consume(interpreter)?;
            }
            ParseTemplateItem::Punct(char) => {
                let _ = interpreter.input().parse_punct_matching(*char)?;
            }
            ParseTemplateItem::Ident(ident) => {
                let _ = interpreter.input().parse_ident_matching(ident)?;
            }
            ParseTemplateItem::Literal(literal) => {
                let _ = interpreter.input().parse_literal_matching(literal)?;
            }
        }
        Ok(())
    }
}

pub(crate) struct ParseTemplateGroup {
    source_delimiter: Delimiter,
    content: ParseTemplateStream,
}

impl ParseSource for ParseTemplateGroup {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_group()?;
        let content = ParseTemplateStream::parse_with_span(&content, delim_span.join())?;
        Ok(Self {
            source_delimiter: delimiter,
            content,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl ParseTemplateGroup {
    fn consume(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        interpreter.parse_group(Some(self.source_delimiter), |interpreter, _, _| {
            self.content.consume(interpreter)
        })
    }
}
