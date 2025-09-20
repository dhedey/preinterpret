use crate::internal_prelude::*;

// An ExactStream is a transformer created with @[EXACT ...].
// It has subtly different behaviour to a transform stream, but they can be nested inside each other.
//
// It differs in two ways:
// * Commands and Outputs are interpreted, and then matched from the parse stream
// * Each consumed item is output as-is

pub(crate) type ExactStream = ExactSegment<UntilEnd>;

#[derive(Clone)]
pub(crate) struct ExactSegment<C> {
    stop_condition: PhantomData<C>,
    inner: Vec<ExactItem>,
}

impl<C: StopCondition<K>, K> Parse<K> for ExactSegment<C>
where
    ExactItem: Parse<K>,
{
    fn parse(input: ParseStream<K>) -> ParseResult<Self> {
        let mut inner = vec![];
        while !C::should_stop(input) {
            inner.push(ExactItem::parse(input)?);
        }
        Ok(Self {
            stop_condition: PhantomData,
            inner,
        })
    }
}

impl<C> ExactSegment<C> {
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<C> HandleTransformation for ExactSegment<C> {
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

/// This must match item-by-item.
#[derive(Clone)]
pub(crate) enum ExactItem {
    TransformStreamInput(ExplicitTransformStream),
    Transformer(Transformer),
    ExactCommandOutput(Command),
    ExactVariableOutput(MarkedVariable),
    ExactExpressionBlock(ExpressionBlock),
    ExactPunct(Punct),
    ExactIdent(Ident),
    ExactLiteral(Literal),
    ExactGroup(ExactGroup),
}

impl Parse<Source> for ExactItem {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => Self::ExactCommandOutput(input.parse()?),
            SourcePeekMatch::Variable(_) => Self::ExactVariableOutput(input.parse()?),
            SourcePeekMatch::ExpressionBlock(_) => Self::ExactExpressionBlock(input.parse()?),
            SourcePeekMatch::AppendVariableParser => {
                return input
                    .parse_err("Append variable bindings are not supported in an EXACT stream")
            }
            SourcePeekMatch::ExplicitTransformStream => Self::TransformStreamInput(input.parse()?),
            SourcePeekMatch::Transformer(_) => Self::Transformer(input.parse()?),
            SourcePeekMatch::Group(_) => Self::ExactGroup(input.parse()?),
            SourcePeekMatch::Ident(_) => Self::ExactIdent(input.parse_any_ident()?),
            SourcePeekMatch::Punct(_) => Self::ExactPunct(input.parse_any_punct()?),
            SourcePeekMatch::Literal(_) => Self::ExactLiteral(input.parse()?),
            SourcePeekMatch::End => return input.parse_err("Unexpected end"),
        })
    }
}

impl Parse<Output> for ExactItem {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            OutputPeekMatch::Group(_) => Self::ExactGroup(input.parse()?),
            OutputPeekMatch::Ident(_) => Self::ExactIdent(input.parse_any_ident()?),
            OutputPeekMatch::Punct(_) => Self::ExactPunct(input.parse_any_punct()?),
            OutputPeekMatch::Literal(_) => Self::ExactLiteral(input.parse()?),
            OutputPeekMatch::End => return input.parse_err("Unexpected end"),
        })
    }
}

impl HandleTransformation for ExactItem {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            ExactItem::TransformStreamInput(transform_stream_input) => {
                transform_stream_input.handle_transform(input, interpreter, output)?;
            }
            ExactItem::Transformer(transformer) => {
                transformer.handle_transform(input, interpreter, output)?;
            }
            ExactItem::ExactCommandOutput(command) => {
                command
                    .clone()
                    .interpret_to_new_stream(interpreter)?
                    .into_exact_stream()?
                    .handle_transform(input, interpreter, output)?;
            }
            ExactItem::ExactVariableOutput(variable) => {
                variable
                    .interpret_to_new_stream(interpreter)?
                    .into_exact_stream()?
                    .handle_transform(input, interpreter, output)?;
            }
            ExactItem::ExactExpressionBlock(expression_block) => {
                expression_block
                    .interpret_to_new_stream(interpreter)?
                    .into_exact_stream()?
                    .handle_transform(input, interpreter, output)?;
            }
            ExactItem::ExactPunct(punct) => {
                output.push_punct(input.parse_punct_matching(punct.as_char())?);
            }
            ExactItem::ExactIdent(ident) => {
                output.push_ident(input.parse_ident_matching(&ident.to_string())?);
            }
            ExactItem::ExactLiteral(literal) => {
                output.push_literal(input.parse_literal_matching(&literal.to_string())?);
            }
            ExactItem::ExactGroup(exact_group) => {
                exact_group.handle_transform(input, interpreter, output)?
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ExactGroup {
    delimiter: Delimiter,
    inner: ExactStream,
}

impl<K> Parse<K> for ExactGroup
where
    ExactStream: Parse<K>,
{
    fn parse(input: ParseStream<K>) -> ParseResult<Self> {
        let (delimiter, _, content) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            inner: content.parse()?,
        })
    }
}

impl HandleTransformation for ExactGroup {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        // Because `None` is ignored by Syn at parsing time, we can effectively be most permissive by ignoring them.
        // This removes a bit of a footgun for users.
        // If they really want to check for a None group, they can embed `@[GROUP @[EXACT ...]]` transformer.
        if self.delimiter == Delimiter::None {
            self.inner.handle_transform(input, interpreter, output)
        } else {
            let (source_span, inner_source) = input.parse_specific_group(self.delimiter)?;
            output.push_grouped(
                |inner_output| {
                    self.inner
                        .handle_transform(&inner_source, interpreter, inner_output)
                },
                self.delimiter,
                source_span.join(),
            )
        }
    }
}
