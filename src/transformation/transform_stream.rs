use crate::internal_prelude::*;

pub(crate) struct TransformStream {
    inner: Vec<TransformItem>,
}

impl ParseSource for TransformStream {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let mut inner = vec![];
        while !input.is_empty() {
            inner.push(input.parse()?);
        }
        Ok(Self { inner })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        for item in self.inner.iter_mut() {
            item.control_flow_pass(context)?;
        }
        Ok(())
    }
}

impl HandleTransformation for TransformStream {
    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        for item in self.inner.iter() {
            item.handle_transform(interpreter)?;
        }
        Ok(())
    }
}

pub(crate) enum TransformItem {
    EmbeddedExpression(EmbeddedExpression),
    EmbeddedStatements(EmbeddedStatements),
    Transformer(Transformer),
    TransformStreamInput(StreamParser),
    ExactPunct(Span, char),
    ExactIdent(Span, String),
    ExactLiteral(Span, String),
    ExactGroup(TransformGroup),
}

impl ParseSource for TransformItem {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::EmbeddedVariable => return input.parse_err("Variable bindings are not supported here. #(x.to_group()) can be inverted with @(#x = @TOKEN_TREE.flatten()). #x can't necessarily be inverted because its contents are flattened, although @(#x = @REST) or @(#x = @[UNTIL ..]) may work in some instances"),
            SourcePeekMatch::EmbeddedExpression => Self::EmbeddedExpression(input.parse()?),
            SourcePeekMatch::EmbeddedStatements => Self::EmbeddedStatements(input.parse()?),
            SourcePeekMatch::Group(_) => Self::ExactGroup(input.parse()?),
            SourcePeekMatch::ExplicitTransformStream => Self::TransformStreamInput(input.parse()?),
            SourcePeekMatch::Transformer(_) => Self::Transformer(input.parse()?),
            SourcePeekMatch::Punct(_) => {
                let punct = input.parse_any_punct()?;
                Self::ExactPunct(punct.span(), punct.as_char())
            }
            SourcePeekMatch::Literal(_) => {
                let literal: Literal = input.parse()?;
                Self::ExactLiteral(literal.span(), literal.to_string())
            }
            SourcePeekMatch::Ident(_) => {
                let ident = input.parse_any_ident()?;
                Self::ExactIdent(ident.span(), ident.to_string())
            }
            SourcePeekMatch::StreamLiteral(_) => return input.parse_err("Stream literals are not supported here. Use an EXACT parser instead."),
            SourcePeekMatch::ObjectLiteral => return input.parse_err("Object literals are not supported here."),
            SourcePeekMatch::End => return input.parse_err("Unexpected end"),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            TransformItem::EmbeddedExpression(block) => block.control_flow_pass(context),
            TransformItem::EmbeddedStatements(statements) => statements.control_flow_pass(context),
            TransformItem::Transformer(transformer) => transformer.control_flow_pass(context),
            TransformItem::TransformStreamInput(stream) => stream.control_flow_pass(context),
            TransformItem::ExactPunct { .. } => Ok(()),
            TransformItem::ExactIdent { .. } => Ok(()),
            TransformItem::ExactLiteral { .. } => Ok(()),
            TransformItem::ExactGroup(group) => group.control_flow_pass(context),
        }
    }
}

impl HandleTransformation for TransformItem {
    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        match self {
            TransformItem::Transformer(transformer) => {
                transformer.handle_transform(interpreter)?;
            }
            TransformItem::TransformStreamInput(stream) => {
                stream.handle_transform(interpreter)?;
            }
            TransformItem::EmbeddedExpression(block) => {
                block.interpret(interpreter)?;
            }
            TransformItem::EmbeddedStatements(statements) => {
                statements.interpret(interpreter)?;
            }
            TransformItem::ExactPunct(span, punct) => {
                interpreter.input(span)?.parse_punct_matching(*punct)?;
            }
            TransformItem::ExactIdent(span, ident) => {
                interpreter.input(span)?.parse_ident_matching(ident)?;
            }
            TransformItem::ExactLiteral(span, literal) => {
                interpreter.input(span)?.parse_literal_matching(literal)?;
            }
            TransformItem::ExactGroup(group) => {
                group.handle_transform(interpreter)?;
            }
        }
        Ok(())
    }
}

pub(crate) struct TransformGroup {
    delimiter: Delimiter,
    delim_span: DelimSpan,
    inner: TransformStream,
}

impl ParseSource for TransformGroup {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (delimiter, delim_span, content) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            delim_span,
            inner: content.parse()?,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.inner.control_flow_pass(context)
    }
}

impl HandleTransformation for TransformGroup {
    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        // Because `None` is ignored by Syn at parsing time, we can effectively be most permissive by ignoring them.
        // This removes a bit of a footgun for users.
        // If they really want to check for a None group, they can embed `@[GROUP ...]` transformer.
        if self.delimiter == Delimiter::None {
            self.inner.handle_transform(interpreter)
        } else {
            interpreter.parse_group(
                &self.delim_span.open(),
                Some(self.delimiter),
                |interpreter, _, _| self.inner.handle_transform(interpreter),
            )
        }
    }
}

pub(crate) struct StreamParser {
    #[allow(unused)]
    transformer_token: Token![@],
    #[allow(unused)]
    parentheses: Parentheses,
    content: StreamParserContent,
}

impl ParseSource for StreamParser {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let transformer_token = input.parse()?;
        let (parentheses, content) = input.parse_parentheses()?;

        Ok(Self {
            transformer_token,
            parentheses,
            content: content.parse()?,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl HandleTransformation for StreamParser {
    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        self.content.handle_transform(interpreter)
    }
}

pub(crate) enum StreamParserContent {
    Output {
        content: TransformStream,
    },
    StoreToVariable {
        variable: VariableDefinition,
        #[allow(unused)]
        equals: Token![=],
        content: TransformStream,
    },
    ExtendToVariable {
        variable: VariableReference,
        #[allow(unused)]
        plus_equals: Token![+=],
        content: TransformStream,
    },
    Discard {
        #[allow(unused)]
        discard: Token![_],
        #[allow(unused)]
        equals: Token![=],
        content: TransformStream,
    },
}

impl ParseSource for StreamParserContent {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        if input.peek(Token![_]) {
            return Ok(Self::Discard {
                discard: input.parse()?,
                equals: input.parse()?,
                content: input.parse()?,
            });
        }
        if input.peek(Token![#]) {
            let _ = input.parse::<Token![#]>()?;
            if let Some((_, cursor)) = input.cursor().ident() {
                if cursor.punct_matching('=').is_some() {
                    return Ok(Self::StoreToVariable {
                        variable: input.parse()?,
                        equals: input.parse()?,
                        content: input.parse()?,
                    });
                }
                if cursor.punct_matching('+').is_some() {
                    return Ok(Self::ExtendToVariable {
                        variable: input.parse()?,
                        plus_equals: input.parse()?,
                        content: input.parse()?,
                    });
                }
            }
            return input.parse_err("Expected '#var =' or '#var +='")?;
        }
        Ok(Self::Output {
            content: input.parse()?,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            StreamParserContent::Output { content } => content.control_flow_pass(context),
            StreamParserContent::StoreToVariable {
                variable,
                equals: _,
                content,
            } => {
                content.control_flow_pass(context)?;
                variable.control_flow_pass(context)
            }
            StreamParserContent::ExtendToVariable {
                variable,
                plus_equals: _,
                content,
            } => {
                // NB: This is correctly a different order compared to StoreToVariable,
                // as it aligns with the execution flow below
                variable.control_flow_pass(context)?;
                content.control_flow_pass(context)
            }
            StreamParserContent::Discard {
                discard: _,
                equals: _,
                content,
            } => content.control_flow_pass(context),
        }
    }
}

impl HandleTransformation for StreamParserContent {
    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        match self {
            StreamParserContent::Output { content } => {
                content.handle_transform(interpreter)?;
            }
            StreamParserContent::StoreToVariable {
                variable, content, ..
            } => {
                let new_output = interpreter
                    .capture_output(|interpreter| content.handle_transform(interpreter))?;
                variable.define(interpreter, new_output);
            }
            StreamParserContent::ExtendToVariable {
                variable, content, ..
            } => {
                let assignee = variable
                    .resolve_concrete(
                        interpreter,
                        ArgumentOwnership::Assignee { auto_create: false },
                    )?
                    .expect_assignee();
                let new_output = interpreter
                    .capture_output(|interpreter| content.handle_transform(interpreter))?;
                new_output.append_into(assignee.0.into_stream()?.as_mut());
            }
            StreamParserContent::Discard { content, .. } => {
                let _ = interpreter
                    .capture_output(|interpreter| content.handle_transform(interpreter))?;
            }
        }
        Ok(())
    }
}
