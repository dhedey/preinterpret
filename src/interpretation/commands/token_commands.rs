use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct SplitCommand {
    inputs: SourceSplitInputs,
}

impl CommandType for SplitCommand {
    type OutputKind = OutputKindValue;
}

define_object_arguments! {
    SourceSplitInputs => SplitInputs {
        required: {
            stream: "%[...] or #var" ("The stream-valued expression to split"),
            separator: "%[::]" ("The token/s to split if they match"),
        },
        optional: {
            drop_empty_start: "false" ("If true, a leading separator does not yield in an empty item at the start (default: false)"),
            drop_empty_middle: "false" ("If true, adjacent separators do not yield an empty item between them (default: false)"),
            drop_empty_end: "true" ("If true, a trailing separator does not yield an empty item at the end (default: true)"),
        }
    }
}

impl ValueCommandDefinition for SplitCommand {
    const COMMAND_NAME: &'static str = "split";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inputs: arguments.fully_parse_as()?,
        })
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let inputs = self.inputs.interpret_to_value(interpreter)?;
        let stream = inputs.stream.expect_stream("The stream input")?;

        let separator = inputs.separator.expect_stream("The separator")?.value;

        let drop_empty_start = match inputs.drop_empty_start {
            Some(value) => value.expect_bool("This parameter")?.value,
            None => false,
        };
        let drop_empty_middle = match inputs.drop_empty_middle {
            Some(value) => value.expect_bool("This parameter")?.value,
            None => false,
        };
        let drop_empty_end = match inputs.drop_empty_end {
            Some(value) => value.expect_bool("This parameter")?.value,
            None => true,
        };

        handle_split(
            interpreter,
            stream,
            separator,
            drop_empty_start,
            drop_empty_middle,
            drop_empty_end,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_split(
    interpreter: &mut Interpreter,
    input: ExpressionStream,
    separator: OutputStream,
    drop_empty_start: bool,
    drop_empty_middle: bool,
    drop_empty_end: bool,
) -> ExecutionResult<ExpressionValue> {
    let output_span_range = input.span_range;
    unsafe {
        // RUST-ANALYZER SAFETY: This is as safe as we can get.
        // Typically the separator won't contain none-delimited groups, so we're OK
        input.value.parse_with(move |input| {
            let mut output = Vec::new();
            let mut current_item = OutputStream::new();

            // Special case separator.len() == 0 to avoid an infinite loop
            if separator.is_empty() {
                while !input.is_empty() {
                    current_item.push_raw_token_tree(input.parse()?);
                    let complete_item = core::mem::replace(&mut current_item, OutputStream::new());
                    output.push(complete_item.to_value(output_span_range));
                }
                return Ok(output.to_value(output_span_range));
            }

            let mut drop_empty_next = drop_empty_start;
            while !input.is_empty() {
                let separator_fork = input.fork();
                let mut ignored_transformer_output = OutputStream::new();
                if separator
                    .parse_exact_match(
                        &separator_fork,
                        interpreter,
                        &mut ignored_transformer_output,
                    )
                    .is_err()
                {
                    current_item.push_raw_token_tree(input.parse()?);
                    continue;
                }
                // This is guaranteed to progress the parser because the separator is non-empty
                input.advance_to(&separator_fork);
                if !current_item.is_empty() || !drop_empty_next {
                    let complete_item = core::mem::replace(&mut current_item, OutputStream::new());
                    output.push(complete_item.to_value(output_span_range));
                }
                drop_empty_next = drop_empty_middle;
            }
            if !current_item.is_empty() || !drop_empty_end {
                output.push(current_item.to_value(output_span_range));
            }
            Ok(output.to_value(output_span_range))
        })
    }
}

#[derive(Clone)]
pub(crate) struct CommaSplitCommand {
    input: SourceStream,
}

impl CommandType for CommaSplitCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for CommaSplitCommand {
    const COMMAND_NAME: &'static str = "comma_split";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            input: arguments.parse_all_as_source()?,
        })
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let comma_span = self.input.span();
        let output_span_range = self.input.span_range();
        let stream = ExpressionStream {
            span_range: output_span_range,
            value: self.input.interpret_to_new_stream(interpreter)?,
        };
        let separator = {
            let mut output = OutputStream::new();
            output.push_punct(Punct::new(',', Spacing::Alone).with_span(comma_span));
            output
        };

        handle_split(interpreter, stream, separator, false, false, true)
    }
}
