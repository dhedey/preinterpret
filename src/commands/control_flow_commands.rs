use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IfCommand {
    condition: NextItem,
    true_code: InterpretationStream,
    false_code: Option<InterpretationStream>,
}

impl CommandDefinition for IfCommand {
    const COMMAND_NAME: &'static str = "if";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::AppendStream;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        static ERROR: &str = "Expected [!if! (condition) { true_code }] or [!if! (condition) { true_code } !else! { false_code}]";

        let condition = arguments.next_item(ERROR)?;
        let true_code = arguments.next_as_kinded_group(Delimiter::Brace, ERROR)?.into_inner_stream();
        let false_code = if !arguments.is_empty() {
            arguments.next_as_punct_matching('!', ERROR)?;
            arguments.next_as_ident_matching("else", ERROR)?;
            arguments.next_as_punct_matching('!', ERROR)?;
            Some(arguments.next_as_kinded_group(Delimiter::Brace, ERROR)?.into_inner_stream())
        } else {
            None
        };
        arguments.assert_end(ERROR)?;

        Ok(Self {
            condition,
            true_code,
            false_code,
        })
    }
}

impl CommandInvocation for IfCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let evaluated_condition = self
            .condition
            .interpret_as_expression(interpreter)?
            .evaluate()?
            .expect_bool("An if condition must evaluate to a boolean")?
            .value();

        if evaluated_condition {
            self.true_code.interpret_as_tokens(interpreter)
        } else if let Some(false_code) = self.false_code {
            false_code.interpret_as_tokens(interpreter)
        } else {
            Ok(InterpretedStream::new())
        }
    }
}

#[derive(Clone)]
pub(crate) struct WhileCommand {
    condition: NextItem,
    loop_code: InterpretationStream,
}

impl CommandDefinition for WhileCommand {
    const COMMAND_NAME: &'static str = "while";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::AppendStream;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        static ERROR: &str = "Expected [!while! (condition) { code }]";

        let condition = arguments.next_item(ERROR)?;
        let loop_code = arguments.next_as_kinded_group(Delimiter::Brace, ERROR)?.into_inner_stream();
        arguments.assert_end(ERROR)?;

        Ok(Self {
            condition,
            loop_code,
        })
    }
}

impl CommandInvocation for WhileCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let mut output = InterpretedStream::new();
        let mut iteration_count = 0;
        loop {
            let evaluated_condition = self.condition
                .clone()
                .interpret_as_expression(interpreter)?
                .evaluate()?
                .expect_bool("An if condition must evaluate to a boolean")?
                .value();

            if !evaluated_condition {
                break;
            }

            iteration_count += 1;
            interpreter
                .config()
                .check_iteration_count(&self.condition, iteration_count)?;
            self.loop_code.clone().interpret_as_tokens_into(interpreter, &mut output)?;
        }

        Ok(output)
    }
}
