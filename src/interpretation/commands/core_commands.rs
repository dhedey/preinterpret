use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct SettingsCommand {
    pub(crate) settings: Expression,
}

impl CommandType for SettingsCommand {
    type OutputKind = OutputKindNone;
}

define_optional_object! {
    pub(crate) struct SettingsInputs {
        iteration_limit: usize => (DEFAULT_ITERATION_LIMIT_STR, "The new iteration limit"),
    }
}

impl NoOutputCommandDefinition for SettingsCommand {
    const COMMAND_NAME: &'static str = "settings";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    settings: input.parse()?,
                })
            },
            "Expected an expression object literal %{ .. }",
        )
    }

    fn execute(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let inputs = self.settings.evaluate(interpreter)?;
        let inputs: SettingsInputs = inputs.resolve_as("The settings inputs")?;
        if let Some(limit) = inputs.iteration_limit {
            interpreter.set_iteration_limit(Some(limit));
        }
        Ok(())
    }
}
