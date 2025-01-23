use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct StreamDestructurer {
    inner: DestructureRemaining,
}

impl DestructurerDefinition for StreamDestructurer {
    const DESTRUCTURER_NAME: &'static str = "stream";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        Ok(Self {
            inner: arguments.fully_parse_no_error_override()?,
        })
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        self.inner.handle_destructure(input, interpreter)
    }
}

#[derive(Clone)]
pub(crate) struct IdentDestructurer {
    variable: Option<DestructureVariable>,
}

impl DestructurerDefinition for IdentDestructurer {
    const DESTRUCTURER_NAME: &'static str = "ident";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                if input.is_empty() {
                    Ok(Self { variable: None })
                } else {
                    Ok(Self {
                        variable: Some(DestructureVariable::parse_only_unflattened_input(input)?),
                    })
                }
            },
            "Expected (!ident! #x) or (!ident #>>x) or (!ident!)",
        )
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        if input.cursor().ident().is_some() {
            match &self.variable {
                Some(variable) => variable.handle_destructure(input, interpreter),
                None => {
                    let _ = input.parse_any_ident()?;
                    Ok(())
                }
            }
        } else {
            Err(input.error("Expected an ident"))
        }
    }
}

#[derive(Clone)]
pub(crate) struct LiteralDestructurer {
    variable: Option<DestructureVariable>,
}

impl DestructurerDefinition for LiteralDestructurer {
    const DESTRUCTURER_NAME: &'static str = "literal";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                if input.is_empty() {
                    Ok(Self { variable: None })
                } else {
                    Ok(Self {
                        variable: Some(DestructureVariable::parse_only_unflattened_input(input)?),
                    })
                }
            },
            "Expected (!literal! #x) or (!literal! #>>x) or (!literal!)",
        )
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        if input.cursor().literal().is_some() {
            match &self.variable {
                Some(variable) => variable.handle_destructure(input, interpreter),
                None => {
                    let _ = input.parse::<Literal>()?;
                    Ok(())
                }
            }
        } else {
            Err(input.error("Expected a literal"))
        }
    }
}

#[derive(Clone)]
pub(crate) struct PunctDestructurer {
    variable: Option<DestructureVariable>,
}

impl DestructurerDefinition for PunctDestructurer {
    const DESTRUCTURER_NAME: &'static str = "punct";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                if input.is_empty() {
                    Ok(Self { variable: None })
                } else {
                    Ok(Self {
                        variable: Some(DestructureVariable::parse_only_unflattened_input(input)?),
                    })
                }
            },
            "Expected (!punct! #x) or (!punct! #>>x) or (!punct!)",
        )
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        if input.cursor().any_punct().is_some() {
            match &self.variable {
                Some(variable) => variable.handle_destructure(input, interpreter),
                None => {
                    let _ = input.parse_any_punct()?;
                    Ok(())
                }
            }
        } else {
            Err(input.error("Expected a punct"))
        }
    }
}

#[derive(Clone)]
pub(crate) struct GroupDestructurer {
    inner: DestructureRemaining,
}

impl DestructurerDefinition for GroupDestructurer {
    const DESTRUCTURER_NAME: &'static str = "group";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        Ok(Self {
            inner: arguments.fully_parse_no_error_override()?,
        })
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        let (_, inner) = input.parse_group_matching(Delimiter::None)?;
        self.inner.handle_destructure(&inner, interpreter)
    }
}

#[derive(Clone)]
pub(crate) struct RawDestructurer {
    stream: RawDestructureStream,
}

impl DestructurerDefinition for RawDestructurer {
    const DESTRUCTURER_NAME: &'static str = "raw";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        let token_stream: TokenStream = arguments.fully_parse_no_error_override()?;
        Ok(Self {
            stream: RawDestructureStream::new_from_token_stream(token_stream),
        })
    }

    fn handle_destructure(&self, input: ParseStream, _: &mut Interpreter) -> Result<()> {
        self.stream.handle_destructure(input)
    }
}

#[derive(Clone)]
pub(crate) struct ContentDestructurer {
    stream: InterpretationStream,
}

impl DestructurerDefinition for ContentDestructurer {
    const DESTRUCTURER_NAME: &'static str = "content";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    stream: InterpretationStream::parse_with_context(
                        input,
                        arguments.full_span_range(),
                    )?,
                })
            },
            "Expected (!content! ... interpretable input ...)",
        )
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        self.stream
            .clone()
            .interpret_to_new_stream(interpreter)?
            .into_raw_destructure_stream()
            .handle_destructure(input)
    }
}
