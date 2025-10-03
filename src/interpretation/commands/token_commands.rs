use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IntersperseCommand {
    span: Span,
    inputs: SourceIntersperseInputs,
}

impl CommandType for IntersperseCommand {
    type OutputKind = OutputKindValue;
}

define_object_arguments! {
    SourceIntersperseInputs => IntersperseInputs {
        required: {
            items: r#"["Hello", "World"]"# ("An array or stream (by coerced token-tree) to intersperse"),
            separator: "%[,]" ("The value to add between each item"),
        },
        optional: {
            add_trailing: "false" ("Whether to add the separator after the last item (default: false)"),
            final_separator: "%[or]" ("Define a different final separator (default: same as normal separator)"),
        }
    }
}

impl ValueCommandDefinition for IntersperseCommand {
    const COMMAND_NAME: &'static str = "intersperse";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            span: arguments.command_span(),
            inputs: arguments.fully_parse_as()?,
        })
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let inputs = self.inputs.interpret_to_value(interpreter)?;
        let items = inputs.items.expect_any_iterator()?;
        let output_span_range = self.span.span_range();
        let add_trailing = match inputs.add_trailing {
            Some(add_trailing) => add_trailing.expect_bool("This parameter")?.value,
            None => false,
        };

        let mut output = Vec::new();

        let mut items = items.into_iter().peekable();

        let mut this_item = match items.next() {
            Some(next) => next,
            None => return Ok(output.to_value(output_span_range)),
        };

        let mut appender = SeparatorAppender {
            separator: inputs.separator,
            final_separator: inputs.final_separator,
            add_trailing,
        };

        loop {
            output.push(this_item);
            let next_item = items.next();
            match next_item {
                Some(next_item) => {
                    let remaining = if items.peek().is_some() {
                        RemainingItemCount::MoreThanOne
                    } else {
                        RemainingItemCount::ExactlyOne
                    };
                    appender.add_separator(remaining, &mut output)?;
                    this_item = next_item;
                }
                None => {
                    appender.add_separator(RemainingItemCount::None, &mut output)?;
                    break;
                }
            }
        }

        Ok(output.to_value(output_span_range))
    }
}

struct SeparatorAppender {
    separator: ExpressionValue,
    final_separator: Option<ExpressionValue>,
    add_trailing: bool,
}

impl SeparatorAppender {
    fn add_separator(
        &mut self,
        remaining: RemainingItemCount,
        output: &mut Vec<ExpressionValue>,
    ) -> ExecutionResult<()> {
        match self.separator(remaining) {
            TrailingSeparator::Normal => output.push(self.separator.clone()),
            TrailingSeparator::Final => match self.final_separator.take() {
                Some(final_separator) => output.push(final_separator),
                None => output.push(self.separator.clone()),
            },
            TrailingSeparator::None => {}
        }
        Ok(())
    }

    fn separator(&self, remaining_item_count: RemainingItemCount) -> TrailingSeparator {
        match remaining_item_count {
            RemainingItemCount::None => {
                if self.add_trailing {
                    TrailingSeparator::Final
                } else {
                    TrailingSeparator::None
                }
            }
            RemainingItemCount::ExactlyOne => {
                if self.add_trailing {
                    TrailingSeparator::Normal
                } else {
                    TrailingSeparator::Final
                }
            }
            RemainingItemCount::MoreThanOne => TrailingSeparator::Normal,
        }
    }
}

enum RemainingItemCount {
    None,
    ExactlyOne,
    MoreThanOne,
}

enum TrailingSeparator {
    Normal,
    Final,
    None,
}

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
