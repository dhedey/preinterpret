use super::*;

#[derive(Clone)]
pub(crate) struct TokenTreeTransformer {
    span: Span,
}

impl TransformerDefinition for TokenTreeTransformer {
    const TRANSFORMER_NAME: &'static str = "TOKEN_TREE";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |_| {
                Ok(Self {
                    span: arguments.full_span(),
                })
            },
            "Expected @TOKEN_TREE or @[TOKEN_TREE]",
        )
    }

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let input = interpreter.input(&self.span)?;
        interpreter
            .output(&self.span)?
            .push_raw_token_tree(input.parse::<TokenTree>()?);
        Ok(())
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct RestTransformer {
    span: Span,
}

impl TransformerDefinition for RestTransformer {
    const TRANSFORMER_NAME: &'static str = "REST";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |_| {
                Ok(Self {
                    span: arguments.full_span(),
                })
            },
            "Expected @REST or @[REST]",
        )
    }

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        ParseUntil::End.handle_parse_into(interpreter, &self.span.span_range())
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct UntilTransformer {
    span: Span,
    until: ParseUntil,
}

impl TransformerDefinition for UntilTransformer {
    const TRANSFORMER_NAME: &'static str = "UNTIL";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(|input| {
            let next: TokenTree = input.parse()?;
            let until = match next {
                TokenTree::Group(group) => {
                    if !group.stream().is_empty() {
                        return group
                            .span()
                            .parse_err("UNTIL only matches until the open of the group. So the group must be empty to indicate this.");
                    }
                    ParseUntil::Group(group.delimiter())
                }
                TokenTree::Ident(ident) => ParseUntil::Ident(ident),
                TokenTree::Punct(punct) => ParseUntil::Punct(punct),
                TokenTree::Literal(literal) => ParseUntil::Literal(literal),
            };
            Ok(Self {
                span: arguments.full_span(),
                until,
            })
        }, "Expected @[UNTIL x] where x is an ident, punct, literal or empty group such as ()")
    }

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        self.until
            .handle_parse_into(interpreter, &self.span.span_range())
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct IdentTransformer {
    span: Span,
}

impl TransformerDefinition for IdentTransformer {
    const TRANSFORMER_NAME: &'static str = "IDENT";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |_| {
                Ok(Self {
                    span: arguments.full_span(),
                })
            },
            "Expected @IDENT or @[IDENT]",
        )
    }

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let input = interpreter.input(&self.span)?;
        if input.cursor().ident().is_some() {
            let ident = input.parse_any_ident()?;
            interpreter
                .output(&self.span.span_range())?
                .push_ident(ident);
            Ok(())
        } else {
            input.parse_err("Expected an ident")?
        }
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct LiteralTransformer {
    span: Span,
}

impl TransformerDefinition for LiteralTransformer {
    const TRANSFORMER_NAME: &'static str = "LITERAL";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |_| {
                Ok(Self {
                    span: arguments.full_span(),
                })
            },
            "Expected @LITERAL or @[LITERAL]",
        )
    }

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let input = interpreter.input(&self.span)?;
        if input.cursor().literal().is_some() {
            let literal = input.parse()?;
            interpreter
                .output(&self.span.span_range())?
                .push_literal(literal);
            Ok(())
        } else {
            input.parse_err("Expected a literal")?
        }
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct PunctTransformer {
    span: Span,
}

impl TransformerDefinition for PunctTransformer {
    const TRANSFORMER_NAME: &'static str = "PUNCT";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |_| {
                Ok(Self {
                    span: arguments.full_span(),
                })
            },
            "Expected @PUNCT or @[PUNCT]",
        )
    }

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let input = interpreter.input(&self.span)?;
        if input.cursor().any_punct().is_some() {
            interpreter
                .output(&self.span.span_range())?
                .push_punct(input.parse_any_punct()?);
            Ok(())
        } else {
            input.parse_err("Expected a punct")?
        }
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

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

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let input = interpreter.input(&Span::call_site())?;
        let (_, inner) = input.parse_transparent_group()?;
        interpreter.with_input(&inner, |interpreter| {
            self.inner.handle_transform(interpreter)
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.inner.control_flow_pass(context)
    }
}

pub(crate) struct ExactTransformer {
    span: Span,
    _parentheses: Parentheses,
    stream: Expression,
}

impl TransformerDefinition for ExactTransformer {
    const TRANSFORMER_NAME: &'static str = "EXACT";

    fn parse(arguments: TransformerArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                let (parentheses, inner) = input.parse_parentheses()?;
                Ok(Self {
                    span: arguments.full_span(),
                    _parentheses: parentheses,
                    stream: inner.parse()?,
                })
            },
            "Expected @[EXACT(%[...stream contents to match exactly...])]",
        )
    }

    fn handle_transform(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        // TODO[parsers]: Ensure that no contextual parser is available when interpreting
        // To save confusion about parse order.
        let stream: ExpressionStream = self
            .stream
            .evaluate_owned(interpreter)?
            .resolve_as("Input to the EXACT parser")?;
        let input = interpreter.input(&self.span)?;
        stream
            .value
            .parse_exact_match(input, interpreter.output(&self.span.span_range())?)
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.stream.control_flow_pass(context)
    }
}
