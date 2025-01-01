use crate::internal_prelude::*;

pub(crate) struct EvaluateCommand;

impl CommandDefinition for EvaluateCommand {
    const COMMAND_NAME: &'static str = "evaluate";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let token_stream =
            command.interpret_remaining_arguments(interpreter, SubstitutionMode::expression())?;
        Ok(evaluate_expression(token_stream, ExpressionParsingMode::Standard)?.into_token_stream())
    }
}

pub(crate) struct IncrementCommand;

impl CommandDefinition for IncrementCommand {
    const COMMAND_NAME: &'static str = "increment";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let error_message = "Expected [!increment! #variable]";
        let variable = command
            .argument_tokens()
            .next_item_as_variable(error_message)?;
        command.argument_tokens().assert_end(error_message)?;
        let variable_contents = variable.execute_substitution(interpreter)?;
        let evaluated_integer =
            evaluate_expression(variable_contents, ExpressionParsingMode::Standard)?
                .expect_integer(&format!("Expected {variable} to evaluate to an integer"))?;
        interpreter.set_variable(
            variable.variable_name().to_string(),
            evaluated_integer
                .increment(command.span_range())?
                .to_token_stream(),
        );
        Ok(TokenStream::new())
    }
}
