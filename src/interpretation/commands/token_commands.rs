use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IsEmptyCommand {
    arguments: SourceStream,
}

impl CommandType for IsEmptyCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            arguments: arguments.parse_all_as_source()?,
        })
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let output_span_range = self.arguments.span_range();
        let interpreted = self.arguments.interpret_to_new_stream(interpreter)?;
        Ok(interpreted.is_empty().to_value(output_span_range))
    }
}

#[derive(Clone)]
pub(crate) struct LengthCommand {
    arguments: SourceStream,
}

impl CommandType for LengthCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for LengthCommand {
    const COMMAND_NAME: &'static str = "length";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            arguments: arguments.parse_all_as_source()?,
        })
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let output_span_range = self.arguments.span_range();
        let interpreted = self.arguments.interpret_to_new_stream(interpreter)?;
        Ok(interpreted.len().to_value(output_span_range))
    }
}

#[derive(Clone)]
pub(crate) struct GroupCommand {
    arguments: SourceStream,
}

impl CommandType for GroupCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for GroupCommand {
    const COMMAND_NAME: &'static str = "group";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            arguments: arguments.parse_all_as_source()?,
        })
    }

    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let span = self.arguments.span();
        output.push_grouped(
            |inner| self.arguments.interpret_into(interpreter, inner),
            Delimiter::None,
            span,
        )
    }
}

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
            separator: "[!stream! ,]" ("The value to add between each item"),
        },
        optional: {
            add_trailing: "false" ("Whether to add the separator after the last item (default: false)"),
            final_separator: "[!stream! or]" ("Define a different final separator (default: same as normal separator)"),
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
        let items = inputs.items.expect_any_iterator("The items")?;
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
            stream: "[!stream! ...] or #var" ("The stream-valued expression to split"),
            separator: "[!stream! ::]" ("The token/s to split if they match"),
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
            separator.into_exact_stream()?,
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
    separator: ExactStream,
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
            if separator.len() == 0 {
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
                    .handle_transform(
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
        let output_span_range = self.input.span_range();
        let stream = ExpressionStream {
            span_range: output_span_range,
            value: self.input.interpret_to_new_stream(interpreter)?,
        };
        let separator = Punct::new(',', Spacing::Alone)
            .with_span(output_span_range.join_into_span_else_start())
            .to_token_stream()
            .source_parse_as()?;

        handle_split(interpreter, stream, separator, false, false, true)
    }
}

#[derive(Clone)]
pub(crate) struct ZipCommand {
    inputs: EitherZipInput,
}

#[derive(Clone)]
enum EitherZipInput {
    Fields(SourceZipInputs),
    JustStream(SourceExpression),
}

define_object_arguments! {
    SourceZipInputs => ZipInputs {
        required: {
            streams: r#"[[!stream! Hello Goodbye] ["World", "Friend"]]"# ("An array of arrays/iterators/streams to zip together."),
        },
        optional: {
            error_on_length_mismatch: "true" ("If false, uses shortest stream length, if true, errors on unequal length. Defaults to true."),
        }
    }
}

impl CommandType for ZipCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for ZipCommand {
    const COMMAND_NAME: &'static str = "zip";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                if input.peek(syn::token::Brace) {
                    Ok(Self {
                        inputs: EitherZipInput::Fields(input.parse()?),
                    })
                } else {
                    Ok(Self {
                        inputs: EitherZipInput::JustStream(input.parse()?),
                    })
                }
            },
            format!(
                "Expected [!zip! [... An array of iterables ...]] or [!zip! {}]",
                SourceZipInputs::describe_object()
            ),
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        let (streams, error_on_length_mismatch) = match self.inputs {
            EitherZipInput::Fields(inputs) => {
                let inputs = inputs.interpret_to_value(interpreter)?;
                (inputs.streams, inputs.error_on_length_mismatch)
            }
            EitherZipInput::JustStream(streams) => {
                let streams = streams.interpret_to_value(interpreter)?;
                (streams, None)
            }
        };
        let streams = streams.expect_array("The zip input")?;
        let output_span_range = streams.span_range;
        let mut output = Vec::new();
        let mut iterators = streams
            .items
            .into_iter()
            .map(|x| x.expect_any_iterator("A zip input"))
            .collect::<Result<Vec<_>, _>>()?;

        let error_on_length_mismatch = match error_on_length_mismatch {
            Some(value) => value.expect_bool("This parameter")?.value,
            None => true,
        };

        if iterators.is_empty() {
            return Ok(output.to_value(output_span_range));
        }

        let min_stream_length = iterators.iter().map(|x| x.size_hint().0).min().unwrap();

        if error_on_length_mismatch {
            let max_stream_length = iterators
                .iter()
                .map(|x| x.size_hint().1)
                .max_by(|a, b| match (a, b) {
                    (None, None) => core::cmp::Ordering::Equal,
                    (None, Some(_)) => core::cmp::Ordering::Greater,
                    (Some(_), None) => core::cmp::Ordering::Less,
                    (Some(a), Some(b)) => a.cmp(b),
                })
                .unwrap();
            if Some(min_stream_length) != max_stream_length {
                return output_span_range.execution_err(format!(
                    "Streams have different lengths and zip's error_on_length_mismatch is true. The lengths vary from {} to {}",
                    min_stream_length,
                    match max_stream_length {
                        Some(max_stream_length) => max_stream_length.to_string(),
                        None => "unbounded".to_string(),
                    },
                ));
            }
        }

        let mut counter = interpreter.start_iteration_counter(&output_span_range);

        for _ in 0..min_stream_length {
            counter.increment_and_check()?;
            let mut inner = Vec::with_capacity(iterators.len());
            for iter in iterators.iter_mut() {
                inner.push(iter.next().unwrap());
            }
            output.push(inner.to_value(output_span_range));
        }

        Ok(output.to_value(output_span_range))
    }
}
