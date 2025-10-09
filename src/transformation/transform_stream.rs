use crate::internal_prelude::*;

#[derive(Clone)]
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
}

impl HandleTransformation for TransformStream {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        for item in self.inner.iter() {
            item.handle_transform(input, interpreter, output)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) enum TransformItem {
    Command(Command),
    EmbeddedExpression(EmbeddedExpression),
    Transformer(Transformer),
    TransformStreamInput(StreamParser),
    ExactPunct(Punct),
    ExactIdent(Ident),
    ExactLiteral(Literal),
    ExactGroup(TransformGroup),
}

impl ParseSource for TransformItem {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => Self::Command(input.parse()?),
            SourcePeekMatch::EmbeddedVariable => return input.parse_err("Variable bindings are not supported here. #(x.to_group()) can be inverted with @(#x = @TOKEN_TREE.flatten()). #x can't necessarily be inverted because its contents are flattened, although @(#x = @REST) or @(#x = @[UNTIL ..]) may work in some instances"),
            SourcePeekMatch::EmbeddedExpression => Self::EmbeddedExpression(input.parse()?),
            SourcePeekMatch::Group(_) => Self::ExactGroup(input.parse()?),
            SourcePeekMatch::ExplicitTransformStream => Self::TransformStreamInput(input.parse()?),
            SourcePeekMatch::Transformer(_) => Self::Transformer(input.parse()?),
            SourcePeekMatch::Punct(_) => Self::ExactPunct(input.parse_any_punct()?),
            SourcePeekMatch::Literal(_) => Self::ExactLiteral(input.parse()?),
            SourcePeekMatch::Ident(_) => Self::ExactIdent(input.parse_any_ident()?),
            SourcePeekMatch::StreamLiteral(_) => return input.parse_err("Stream literals are not supported here. Use an EXACT parser instead."),
            SourcePeekMatch::ObjectLiteral => return input.parse_err("Object literals are not supported here."),
            SourcePeekMatch::End => return input.parse_err("Unexpected end"),
        })
    }
}

impl HandleTransformation for TransformItem {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            TransformItem::Command(command) => {
                command.clone().interpret_into(interpreter, output)?;
            }
            TransformItem::Transformer(transformer) => {
                transformer.handle_transform(input, interpreter, output)?;
            }
            TransformItem::TransformStreamInput(stream) => {
                stream.handle_transform(input, interpreter, output)?;
            }
            TransformItem::EmbeddedExpression(block) => {
                block.interpret_into(interpreter, output)?;
            }
            TransformItem::ExactPunct(punct) => {
                input.parse_punct_matching(punct.as_char())?;
            }
            TransformItem::ExactIdent(ident) => {
                input.parse_ident_matching(&ident.to_string())?;
            }
            TransformItem::ExactLiteral(literal) => {
                input.parse_literal_matching(&literal.to_string())?;
            }
            TransformItem::ExactGroup(group) => {
                group.handle_transform(input, interpreter, output)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct TransformGroup {
    delimiter: Delimiter,
    inner: TransformStream,
}

impl ParseSource for TransformGroup {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (delimiter, _, content) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            inner: content.parse()?,
        })
    }
}

impl HandleTransformation for TransformGroup {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        // Because `None` is ignored by Syn at parsing time, we can effectively be most permissive by ignoring them.
        // This removes a bit of a footgun for users.
        // If they really want to check for a None group, they can embed `@[GROUP ...]` transformer.
        if self.delimiter == Delimiter::None {
            self.inner.handle_transform(input, interpreter, output)
        } else {
            let (_, inner) = input.parse_specific_group(self.delimiter)?;
            self.inner.handle_transform(&inner, interpreter, output)
        }
    }
}

#[derive(Clone)]
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
}

impl HandleTransformation for StreamParser {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.content.handle_transform(input, interpreter, output)
    }
}

#[derive(Clone)]
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
}

impl HandleTransformation for StreamParserContent {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            StreamParserContent::Output { content } => {
                content.handle_transform(input, interpreter, output)?;
            }
            StreamParserContent::StoreToVariable {
                variable, content, ..
            } => {
                let mut new_output = OutputStream::new();
                content.handle_transform(input, interpreter, &mut new_output)?;
                variable.define(interpreter, new_output);
            }
            StreamParserContent::ExtendToVariable {
                variable, content, ..
            } => {
                let reference = variable.binding(interpreter)?;
                content.handle_transform(
                    input,
                    interpreter,
                    reference.into_mut()?.into_stream()?.as_mut(),
                )?;
            }
            StreamParserContent::Discard { content, .. } => {
                let mut discarded = OutputStream::new();
                content.handle_transform(input, interpreter, &mut discarded)?;
            }
        }
        Ok(())
    }
}
