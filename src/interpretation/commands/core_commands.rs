use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct SettingsCommand {
    inputs: SourceSettingsInputs,
}

impl CommandType for SettingsCommand {
    type OutputKind = OutputKindNone;
}

define_object_arguments! {
    SourceSettingsInputs => SettingsInputs {
        required: {},
        optional: {
            iteration_limit: DEFAULT_ITERATION_LIMIT_STR ("The new iteration limit"),
        }
    }
}

impl NoOutputCommandDefinition for SettingsCommand {
    const COMMAND_NAME: &'static str = "settings";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inputs: arguments.fully_parse_as()?,
        })
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        let inputs = self.inputs.interpret_to_value(interpreter)?;
        if let Some(limit) = inputs.iteration_limit {
            let limit = limit
                .expect_integer("The iteration limit")?
                .expect_usize()?;
            interpreter.set_iteration_limit(Some(limit));
        }
        Ok(())
    }
}
