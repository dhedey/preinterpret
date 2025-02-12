use crate::internal_prelude::*;

pub(crate) type TransformStream = TransformSegment<UntilEnd>;
pub(crate) type TransformStreamUntilToken<T> = TransformSegment<UntilToken<T>>;

#[derive(Clone)]
pub(crate) struct TransformSegment<C> {
    stop_condition: PhantomData<C>,
    inner: Vec<TransformItem>,
}

impl<C: StopCondition<Source>> Parse<Source> for TransformSegment<C> {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let mut inner = vec![];
        while !C::should_stop(input) {
            inner.push(TransformItem::parse_until::<C>(input)?);
        }
        Ok(Self {
            stop_condition: PhantomData,
            inner,
        })
    }
}

impl<C> HandleTransformation for TransformSegment<C> {
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
    Variable(VariableBinding),
    ExpressionBlock(ExpressionBlock),
    Transformer(Transformer),
    TransformStreamInput(ExplicitTransformStream),
    ExactPunct(Punct),
    ExactIdent(Ident),
    ExactLiteral(Literal),
    ExactGroup(TransformGroup),
}

impl TransformItem {
    /// We provide a stop condition so that some of the items can know when to stop consuming greedily -
    /// notably the flattened command. This allows [!let! #..x = Hello => World] to parse as setting
    /// `x` to `Hello => World` rather than having `#..x` peeking to see it is "up to =" and then only
    /// parsing `Hello` into `x`.
    pub(crate) fn parse_until<C: StopCondition<Source>>(
        input: ParseStream<Source>,
    ) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => Self::Command(input.parse()?),
            SourcePeekMatch::Variable(_) | SourcePeekMatch::AppendVariableBinding => {
                Self::Variable(VariableBinding::parse_until::<C>(input)?)
            }
            SourcePeekMatch::ExpressionBlock(_) => Self::ExpressionBlock(input.parse()?),
            SourcePeekMatch::Group(_) => Self::ExactGroup(input.parse()?),
            SourcePeekMatch::ExplicitTransformStream => Self::TransformStreamInput(input.parse()?),
            SourcePeekMatch::Transformer(_) => Self::Transformer(input.parse()?),
            SourcePeekMatch::Punct(_) => Self::ExactPunct(input.parse_any_punct()?),
            SourcePeekMatch::Literal(_) => Self::ExactLiteral(input.parse()?),
            SourcePeekMatch::Ident(_) => Self::ExactIdent(input.parse_any_ident()?),
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
            TransformItem::Variable(variable) => {
                variable.handle_transform(input, interpreter, output)?;
            }
            TransformItem::Command(command) => {
                command.clone().interpret_into(interpreter, output)?;
            }
            TransformItem::Transformer(transformer) => {
                transformer.handle_transform(input, interpreter, output)?;
            }
            TransformItem::TransformStreamInput(stream) => {
                stream.handle_transform(input, interpreter, output)?;
            }
            TransformItem::ExpressionBlock(block) => {
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

impl Parse<Source> for TransformGroup {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
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
        let (_, inner) = input.parse_specific_group(self.delimiter)?;
        self.inner.handle_transform(&inner, interpreter, output)
    }
}

#[derive(Clone)]
pub(crate) struct ExplicitTransformStream {
    #[allow(unused)]
    transformer_token: Token![@],
    #[allow(unused)]
    delim_span: DelimSpan,
    arguments: ExplicitTransformStreamArguments,
}

#[derive(Clone)]
pub(crate) enum ExplicitTransformStreamArguments {
    Output {
        content: TransformStream,
    },
    StoreToVariable {
        variable: GroupedVariable,
        #[allow(unused)]
        equals: Token![=],
        content: TransformStream,
    },
    ExtendToVariable {
        variable: GroupedVariable,
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

impl Parse<Source> for ExplicitTransformStreamArguments {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        if input.peek(Token![_]) {
            return Ok(Self::Discard {
                discard: input.parse()?,
                equals: input.parse()?,
                content: input.parse()?,
            });
        }
        if input.peek(Token![#]) {
            let variable = input.parse()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![=]) {
                return Ok(Self::StoreToVariable {
                    variable,
                    equals: input.parse()?,
                    content: input.parse()?,
                });
            }
            if lookahead.peek(Token![+=]) {
                return Ok(Self::ExtendToVariable {
                    variable,
                    plus_equals: input.parse()?,
                    content: input.parse()?,
                });
            }
            Err(lookahead.error())?;
        }
        Ok(Self::Output {
            content: input.parse()?,
        })
    }
}

impl Parse<Source> for ExplicitTransformStream {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let transformer_token = input.parse()?;
        let (delim_span, content) = input.parse_specific_group(Delimiter::Parenthesis)?;

        Ok(Self {
            transformer_token,
            delim_span,
            arguments: content.parse()?,
        })
    }
}

impl HandleTransformation for ExplicitTransformStream {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match &self.arguments {
            ExplicitTransformStreamArguments::Output { content } => {
                content.handle_transform(input, interpreter, output)?;
            }
            ExplicitTransformStreamArguments::StoreToVariable {
                variable, content, ..
            } => {
                let mut new_output = OutputStream::new();
                content.handle_transform(input, interpreter, &mut new_output)?;
                variable.set_stream(interpreter, new_output)?;
            }
            ExplicitTransformStreamArguments::ExtendToVariable {
                variable, content, ..
            } => {
                let variable_data = variable.get_existing_for_mutation(interpreter)?;
                content.handle_transform(
                    input,
                    interpreter,
                    variable_data.get_mut_stream(variable)?.deref_mut(),
                )?;
            }
            ExplicitTransformStreamArguments::Discard { content, .. } => {
                let mut discarded = OutputStream::new();
                content.handle_transform(input, interpreter, &mut discarded)?;
            }
        }
        Ok(())
    }
}
