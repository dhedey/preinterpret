use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IfCommand {
    condition: ExpressionInput,
    true_code: CommandCodeInput,
    else_ifs: Vec<(ExpressionInput, CommandCodeInput)>,
    else_code: Option<CommandCodeInput>,
}

impl CommandType for IfCommand {
    type OutputKind = OutputKindControlFlow;
}

impl ControlFlowCommandDefinition for IfCommand {
    const COMMAND_NAME: &'static str = "if";

    fn parse(arguments: CommandArguments) -> Result<Self> {
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
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let evaluated_condition = self
            .condition
            .evaluate(interpreter)?
            .expect_bool("An if condition must evaluate to a boolean")?
            .value();

        if evaluated_condition {
            return self.true_code.interpret_into(interpreter, output);
        }

        for (condition, code) in self.else_ifs {
            let evaluated_condition = condition
                .evaluate(interpreter)?
                .expect_bool("An else if condition must evaluate to a boolean")?
                .value();

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
    condition: ExpressionInput,
    loop_code: CommandCodeInput,
}

impl CommandType for WhileCommand {
    type OutputKind = OutputKindControlFlow;
}

impl ControlFlowCommandDefinition for WhileCommand {
    const COMMAND_NAME: &'static str = "while";

    fn parse(arguments: CommandArguments) -> Result<Self> {
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
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let mut iteration_count = 0;
        loop {
            let evaluated_condition = self
                .condition
                .clone()
                .evaluate(interpreter)?
                .expect_bool("An if condition must evaluate to a boolean")?
                .value();

            if !evaluated_condition {
                break;
            }

            iteration_count += 1;
            interpreter
                .config()
                .check_iteration_count(&self.condition, iteration_count)?;
            self.loop_code.clone().interpret_into(interpreter, output)?;
        }

        Ok(())
    }
}
