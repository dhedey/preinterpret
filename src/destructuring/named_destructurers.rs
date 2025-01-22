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
    variable: DestructureVariable,
}

impl DestructurerDefinition for IdentDestructurer {
    const DESTRUCTURER_NAME: &'static str = "ident";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: DestructureVariable::parse_only_unflattened_input(input)?,
                })
            },
            "Expected (!ident! #x) or (!ident #>>x)",
        )
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        if input.cursor().ident().is_some() {
            self.variable.handle_destructure(input, interpreter)
        } else {
            Err(input.error("Expected an ident"))
        }
    }
}

#[derive(Clone)]
pub(crate) struct LiteralDestructurer {
    variable: DestructureVariable,
}

impl DestructurerDefinition for LiteralDestructurer {
    const DESTRUCTURER_NAME: &'static str = "literal";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: DestructureVariable::parse_only_unflattened_input(input)?,
                })
            },
            "Expected (!literal! #x) or (!literal! #>>x)",
        )
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        if input.cursor().literal().is_some() {
            self.variable.handle_destructure(input, interpreter)
        } else {
            Err(input.error("Expected a literal"))
        }
    }
}

#[derive(Clone)]
pub(crate) struct PunctDestructurer {
    variable: DestructureVariable,
}

impl DestructurerDefinition for PunctDestructurer {
    const DESTRUCTURER_NAME: &'static str = "punct";

    fn parse(arguments: DestructurerArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: DestructureVariable::parse_only_unflattened_input(input)?,
                })
            },
            "Expected (!punct! #x) or (!punct! #>>x)",
        )
    }

    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        if input.cursor().any_punct().is_some() {
            self.variable.handle_destructure(input, interpreter)
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
