use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IfCommand {
    condition: ExpressionInput,
    true_code: CommandCodeInput,
    false_code: Option<CommandCodeInput>,
}

impl CommandType for IfCommand {
    type OutputKind = OutputKindStreaming;
}

impl StreamingCommandDefinition for IfCommand {
    const COMMAND_NAME: &'static str = "if";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    condition: input.parse()?,
                    true_code: input.parse()?,
                    false_code: {
                        if !input.is_empty() {
                            input.parse::<Token![!]>()?;
                            input.parse::<Token![else]>()?;
                            input.parse::<Token![!]>()?;
                            Some(input.parse()?)
                        } else {
                            None
                        }
                    },
                })
            },
            "Expected [!if! (condition) { true_code }] or [!if! (condition) { true_code } !else! { false_code }]",
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
            self.true_code
                .interpret_as_tokens_into(interpreter, output)?
        } else if let Some(false_code) = self.false_code {
            false_code.interpret_as_tokens_into(interpreter, output)?
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
    type OutputKind = OutputKindStreaming;
}

impl StreamingCommandDefinition for WhileCommand {
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
            self.loop_code
                .clone()
                .interpret_as_tokens_into(interpreter, output)?;
        }

        Ok(())
    }
}
