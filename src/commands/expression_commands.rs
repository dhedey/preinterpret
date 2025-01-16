use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EvaluateCommand {
    expression: InterpretationStream,
}

impl CommandDefinition for EvaluateCommand {
    const COMMAND_NAME: &'static str = "evaluate";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            expression: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for EvaluateCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let expression = self.expression.interpret_as_expression(interpreter)?;
        Ok(CommandOutput::GroupedStream(
            expression.evaluate()?.into_interpreted_stream(),
        ))
    }
}

#[derive(Clone)]
pub(crate) struct AssignCommand {
    variable: GroupedVariable,
    operator: Punct,
    #[allow(unused)]
    equals: Token![=],
    expression: InterpretationStream,
}

impl CommandDefinition for AssignCommand {
    const COMMAND_NAME: &'static str = "assign";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    variable: input.parse()?,
                    operator: {
                        let operator: Punct = input.parse()?;
                        match operator.as_char() {
                            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
                            _ => return operator.err("Expected one of + - * / % & | or ^"),
                        }
                        operator
                    },
                    equals: input.parse()?,
                    expression: input.parse_with(arguments.full_span_range())?,
                })
            },
            "Expected [!assign! #variable += ...] for + or some other operator supported in an expression",
        )
    }
}

impl CommandInvocation for AssignCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let Self {
            variable,
            operator,
            equals: _,
            expression,
        } = *self;

        let mut expression_stream = ExpressionStream::new(expression.span_range());
        variable.interpret_as_expression_into(interpreter, &mut expression_stream)?;
        expression_stream.push_punct(operator);
        expression.interpret_as_expression_into(interpreter, &mut expression_stream)?;

        let output = expression_stream.evaluate()?.into_interpreted_stream();
        variable.set(interpreter, output)?;

        Ok(CommandOutput::Empty)
    }
}
