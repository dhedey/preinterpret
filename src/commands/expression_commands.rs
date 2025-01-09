use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EvaluateCommand {
    expression: InterpretationStream,
}

impl CommandDefinition for EvaluateCommand {
    const COMMAND_NAME: &'static str = "evaluate";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::SingleToken;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        Ok(Self {
            expression: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for EvaluateCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let expression = self.expression.interpret_as_expression(interpreter)?;
        Ok(expression.evaluate()?.into_interpreted_stream())
    }
}

#[derive(Clone)]
pub(crate) struct AssignCommand {
    variable: Variable,
    operator: Punct,
    #[allow(unused)]
    equals: Punct,
    expression: InterpretationStream,
}

impl CommandDefinition for AssignCommand {
    const COMMAND_NAME: &'static str = "assign";

    const OUTPUT_BEHAVIOUR: CommandOutputBehaviour = CommandOutputBehaviour::EmptyStream;

    fn parse(mut arguments: InterpreterParseStream) -> Result<Self> {
        static ERROR: &str = "Expected [!assign! #variable += ...] for + or some other operator supported in an expression";
        Ok(Self {
            variable: arguments.next_as_variable(ERROR)?,
            operator: {
                let operator = arguments.next_as_punct(ERROR)?;
                match operator.as_char() {
                    '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
                    _ => return operator.err("Expected one of + - * / % & | or ^"),
                }
                operator
            },
            equals: arguments.next_as_punct_matching('=', ERROR)?,
            expression: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for AssignCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<InterpretedStream> {
        let Self {
            variable,
            operator,
            equals: _,
            expression,
        } = *self;

        let mut expression_stream = ExpressionStream::new();
        variable.interpret_as_expression_into(interpreter, &mut expression_stream)?;
        expression_stream.push_punct(operator);
        expression.interpret_as_expression_into(interpreter, &mut expression_stream)?;

        let output = expression_stream.evaluate()?.into_interpreted_stream();
        variable.set(interpreter, output);

        Ok(InterpretedStream::new())
    }
}
