use crate::internal_prelude::*;

/// This is temporary until we have a proper implementation of #(...)
#[derive(Clone)]
pub(crate) struct ReinterpretCommand {
    content: SourceStream,
}

impl CommandType for ReinterpretCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for ReinterpretCommand {
    const COMMAND_NAME: &'static str = "reinterpret";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            content: arguments.parse_all_as_source()?,
        })
    }

    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let command_span = self.content.span();
        let interpreted = self.content.interpret_to_new_stream(interpreter)?;
        let source = unsafe {
            // RUST-ANALYZER-SAFETY - Can't do much better than this
            interpreted.into_token_stream()
        };
        let reparsed_source_stream =
            source.source_parse_with(|input| SourceStream::parse(input, command_span))?;
        reparsed_source_stream.interpret_into(interpreter, output)
    }
}

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
