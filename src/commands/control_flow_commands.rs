use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IfCommand {
    condition: ExpressionInput,
    true_code: CommandCodeInput,
    false_code: Option<CommandCodeInput>,
    nothing_span_range: SpanRange,
}

impl CommandDefinition for IfCommand {
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
                    nothing_span_range: arguments.full_span_range(),
                })
            },
            "Expected [!if! (condition) { true_code }] or [!if! (condition) { true_code } !else! { false_code }]",
        )
    }
}

impl CommandInvocation for IfCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let evaluated_condition = self
            .condition
            .evaluate(interpreter)?
            .expect_bool("An if condition must evaluate to a boolean")?
            .value();

        let output = if evaluated_condition {
            self.true_code.interpret_as_tokens(interpreter)?
        } else if let Some(false_code) = self.false_code {
            false_code.interpret_as_tokens(interpreter)?
        } else {
            InterpretedStream::new(self.nothing_span_range)
        };

        Ok(CommandOutput::Stream(output))
    }
}

#[derive(Clone)]
pub(crate) struct WhileCommand {
    condition: ExpressionInput,
    loop_code: CommandCodeInput,
    nothing_span_range: SpanRange,
}

impl CommandDefinition for WhileCommand {
    const COMMAND_NAME: &'static str = "while";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    condition: input.parse()?,
                    loop_code: input.parse()?,
                    nothing_span_range: arguments.full_span_range(),
                })
            },
            "Expected [!while! (condition) { code }]",
        )
    }
}

impl CommandInvocation for WhileCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let mut output = InterpretedStream::new(self.nothing_span_range);
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
                .interpret_as_tokens_into(interpreter, &mut output)?;
        }

        Ok(CommandOutput::Stream(output))
    }
}
