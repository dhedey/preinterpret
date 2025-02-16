use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IdentTransformer;

impl TransformerDefinition for IdentTransformer {
    const TRANSFORMER_NAME: &'static str = "IDENT";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(|_| Ok(Self), "Expected @IDENT or @[IDENT]")
    }

    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        _: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        if input.cursor().ident().is_some() {
            output.push_ident(input.parse_any_ident()?);
            Ok(())
        } else {
            input.parse_err("Expected an ident")?
        }
    }
}

#[derive(Clone)]
pub(crate) struct LiteralTransformer;

impl TransformerDefinition for LiteralTransformer {
    const TRANSFORMER_NAME: &'static str = "LITERAL";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(|_| Ok(Self), "Expected @LITERAL or @[LITERAL]")
    }

    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        _: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        if input.cursor().literal().is_some() {
            output.push_literal(input.parse()?);
            Ok(())
        } else {
            input.parse_err("Expected a literal")?
        }
    }
}

#[derive(Clone)]
pub(crate) struct PunctTransformer;

impl TransformerDefinition for PunctTransformer {
    const TRANSFORMER_NAME: &'static str = "PUNCT";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(|_| Ok(Self), "Expected @PUNCT or @[PUNCT]")
    }

    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        _: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        if input.cursor().any_punct().is_some() {
            output.push_punct(input.parse_any_punct()?);
            Ok(())
        } else {
            input.parse_err("Expected a punct")?
        }
    }
}

#[derive(Clone)]
pub(crate) struct GroupTransformer {
    inner: TransformStream,
}

impl TransformerDefinition for GroupTransformer {
    const TRANSFORMER_NAME: &'static str = "GROUP";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        Ok(Self {
            inner: arguments.fully_parse_no_error_override()?,
        })
    }

    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let (_, inner) = input.parse_transparent_group()?;
        self.inner.handle_transform(&inner, interpreter, output)
    }
}

#[derive(Clone)]
pub(crate) struct ExactTransformer {
    stream: ExactStream,
}

impl TransformerDefinition for ExactTransformer {
    const TRANSFORMER_NAME: &'static str = "EXACT";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    stream: ExactStream::parse(input)?,
                })
            },
            "Expected @[EXACT ... interpretable input to be matched exactly ...]",
        )
    }

    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.stream.handle_transform(input, interpreter, output)
    }
}
