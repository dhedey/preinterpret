use crate::internal_prelude::*;

pub(crate) struct EvaluateCommand;

impl CommandDefinition for EvaluateCommand {
    const COMMAND_NAME: &'static str = "evaluate";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<InterpretedStream> {
        let expression = command.arguments().interpret_as_expression(interpreter)?;
        Ok(expression.evaluate()?.into_interpreted_stream())
    }
}

pub(crate) struct AssignCommand;

impl CommandDefinition for AssignCommand {
    const COMMAND_NAME: &'static str = "assign";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<InterpretedStream> {
        let AssignStatementStart {
            variable,
            operator,
        } = AssignStatementStart::parse(command.arguments())
            .ok_or_else(|| command.error("Expected [!assign! #variable += ...] for + or some other operator supported in an expression"))?;

        let mut expression_stream = ExpressionStream::new();
        variable.interpret_as_expression_into(interpreter, &mut expression_stream)?;
        operator
            .into_token_stream()
            .interpret_as_expression_into(interpreter, &mut expression_stream)?;
        command
            .arguments()
            .interpret_as_expression_into(interpreter, &mut expression_stream)?;

        let output = expression_stream.evaluate()?.into_interpreted_stream();
        variable.set(interpreter, output);

        Ok(InterpretedStream::new())
    }
}

struct AssignStatementStart {
    variable: Variable,
    operator: Punct,
}

impl AssignStatementStart {
    fn parse(tokens: &mut InterpreterParseStream) -> Option<Self> {
        let variable = tokens.next_item_as_variable("").ok()?;
        let operator = tokens.next_as_punct()?;
        match operator.as_char() {
            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
            _ => return None,
        }
        tokens.next_as_punct_matching('=')?;
        Some(AssignStatementStart { variable, operator })
    }
}
