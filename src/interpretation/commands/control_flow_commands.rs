use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IfCommand {
    condition: SourceExpression,
    true_code: SourceCodeBlock,
    else_ifs: Vec<(SourceExpression, SourceCodeBlock)>,
    else_code: Option<SourceCodeBlock>,
}

impl CommandType for IfCommand {
    type OutputKind = OutputKindControlFlow;
}

impl ControlFlowCommandDefinition for IfCommand {
    const COMMAND_NAME: &'static str = "if";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                let condition = input.parse()?;
                let true_code = input.parse()?;
                let mut else_ifs = Vec::new();
                let mut else_code = None;
                while !input.is_empty() {
                    input.parse::<Token![!]>()?;
                    if input.peek_ident_matching("elif") {
                        input.parse_ident_matching("elif")?;
                        input.parse::<Token![!]>()?;
                        else_ifs.push((input.parse()?, input.parse()?));
                    } else {
                        input.parse_ident_matching("else")?;
                        input.parse::<Token![!]>()?;
                        else_code = Some(input.parse()?);
                        break;
                    }
                }
                Ok(Self {
                    condition,
                    true_code,
                    else_ifs,
                    else_code,
                })
            },
            "Expected [!if! ... { ... } !else if! ... { ... } !else! ... { ... }]",
        )
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let evaluated_condition = self
            .condition
            .evaluate(interpreter)?
            .expect_bool("An if condition must evaluate to a boolean")?;

        if evaluated_condition {
            return self.true_code.interpret_into(interpreter, output);
        }

        for (condition, code) in self.else_ifs {
            let evaluated_condition = condition
                .evaluate(interpreter)?
                .expect_bool("An else if condition must evaluate to a boolean")?;

            if evaluated_condition {
                return code.interpret_into(interpreter, output);
            }
        }

        if let Some(false_code) = self.else_code {
            return false_code.interpret_into(interpreter, output);
        }

        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct WhileCommand {
    condition: SourceExpression,
    loop_code: SourceCodeBlock,
}

impl CommandType for WhileCommand {
    type OutputKind = OutputKindControlFlow;
}

impl ControlFlowCommandDefinition for WhileCommand {
    const COMMAND_NAME: &'static str = "while";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    condition: input.parse()?,
                    loop_code: input.parse()?,
                })
            },
            "Expected [!while! (condition) { code }]",
        )
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let mut iteration_counter = interpreter.start_iteration_counter(&self.condition);
        loop {
            iteration_counter.increment_and_check()?;

            let evaluated_condition = self
                .condition
                .clone()
                .evaluate(interpreter)?
                .expect_bool("An if condition must evaluate to a boolean")?;

            if !evaluated_condition {
                break;
            }

            match self
                .loop_code
                .clone()
                .interpret_loop_content_into(interpreter, output)?
            {
                None => {}
                Some(ControlFlowInterrupt::Continue) => continue,
                Some(ControlFlowInterrupt::Break) => break,
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct LoopCommand {
    loop_code: SourceCodeBlock,
}

impl CommandType for LoopCommand {
    type OutputKind = OutputKindControlFlow;
}

impl ControlFlowCommandDefinition for LoopCommand {
    const COMMAND_NAME: &'static str = "loop";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    loop_code: input.parse()?,
                })
            },
            "Expected [!loop! { ... }]",
        )
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let mut iteration_counter = interpreter.start_iteration_counter(&self.loop_code);

        loop {
            iteration_counter.increment_and_check()?;
            match self
                .loop_code
                .clone()
                .interpret_loop_content_into(interpreter, output)?
            {
                None => {}
                Some(ControlFlowInterrupt::Continue) => continue,
                Some(ControlFlowInterrupt::Break) => break,
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ForCommand {
    parse_place: DestructureUntil<Token![in]>,
    #[allow(unused)]
    in_token: Token![in],
    input: CommandStreamInput,
    loop_code: SourceCodeBlock,
}

impl CommandType for ForCommand {
    type OutputKind = OutputKindControlFlow;
}

impl ControlFlowCommandDefinition for ForCommand {
    const COMMAND_NAME: &'static str = "for";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    parse_place: input.parse()?,
                    in_token: input.parse()?,
                    input: input.parse()?,
                    loop_code: input.parse()?,
                })
            },
            "Expected [!for! #x in [ ... ] { code }]",
        )
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let stream = self.input.interpret_to_new_stream(interpreter)?;

        let mut iteration_counter = interpreter.start_iteration_counter(&self.in_token);

        for token in stream {
            iteration_counter.increment_and_check()?;
            self.parse_place
                .handle_destructure_from_stream(token.into(), interpreter)?;
            match self
                .loop_code
                .clone()
                .interpret_loop_content_into(interpreter, output)?
            {
                None => {}
                Some(ControlFlowInterrupt::Continue) => continue,
                Some(ControlFlowInterrupt::Break) => break,
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ContinueCommand {
    span: Span,
}

impl CommandType for ContinueCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for ContinueCommand {
    const COMMAND_NAME: &'static str = "continue";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.assert_empty("The !continue! command takes no arguments")?;
        Ok(Self {
            span: arguments.command_span(),
        })
    }

    fn execute(self: Box<Self>, _: &mut Interpreter) -> ExecutionResult<()> {
        ExecutionResult::Err(ExecutionInterrupt::ControlFlow(
            ControlFlowInterrupt::Continue,
            self.span,
        ))
    }
}

#[derive(Clone)]
pub(crate) struct BreakCommand {
    span: Span,
}

impl CommandType for BreakCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for BreakCommand {
    const COMMAND_NAME: &'static str = "break";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.assert_empty("The !break! command takes no arguments")?;
        Ok(Self {
            span: arguments.command_span(),
        })
    }

    fn execute(self: Box<Self>, _: &mut Interpreter) -> ExecutionResult<()> {
        ExecutionResult::Err(ExecutionInterrupt::ControlFlow(
            ControlFlowInterrupt::Break,
            self.span,
        ))
    }
}
