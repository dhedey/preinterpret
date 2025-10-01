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
    inputs: SourceExpression,
}

impl CommandType for ZipCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for ZipCommand {
    const COMMAND_NAME: &'static str = "zip";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    inputs: input.parse()?,
                })
            },
            "Expected [!zip! [<iter1>, <iter2>, ..]] or [!zip! {{ x: <iter1>, y: <iter2>, .. }}] for <iterN> iterable values of the same length. If you instead want to permit different lengths and truncate to the shortest, use `!zip_truncated!` instead.",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        zip_inner(
            self.inputs.interpret_to_value(interpreter)?,
            interpreter,
            true,
        )
    }
}

#[derive(Clone)]
pub(crate) struct ZipTruncatedCommand {
    inputs: SourceExpression,
}

impl CommandType for ZipTruncatedCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for ZipTruncatedCommand {
    const COMMAND_NAME: &'static str = "zip_truncated";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        arguments.fully_parse_or_error(
            |input| {
                Ok(Self {
                    inputs: input.parse()?,
                })
            },
            "Expected [!zip_truncated! [<iter1>, <iter2>, ..]] or [!zip_truncated! {{ x: <iter1>, y: <iter2>, .. }}] for <iterN> iterable values of possible different lengths (the shortest length will be used). If you want to ensure the lengths are equal, use `!zip!` instead",
        )
    }

    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue> {
        zip_inner(
            self.inputs.interpret_to_value(interpreter)?,
            interpreter,
            false,
        )
    }
}

fn zip_inner(
    iterators: ExpressionValue,
    interpreter: &mut Interpreter,
    error_on_length_mismatch: bool,
) -> ExecutionResult<ExpressionValue> {
    let output_span_range = iterators.span_range();
    let mut iterators = ZipIterators::from_value(iterators)?;
    let mut output = Vec::new();

    if iterators.len() == 0 {
        return Ok(output.to_value(output_span_range));
    }

    let (min_iterator_min_length, max_iterator_max_length) = iterators.size_hint_range();

    if error_on_length_mismatch && Some(min_iterator_min_length) != max_iterator_max_length {
        return output_span_range.execution_err(format!(
            "The iterables have different lengths. The lengths vary from {} to {}. To truncate to the shortest, use `!zip_truncated!` instead of `!zip!",
            min_iterator_min_length,
            match max_iterator_max_length {
                Some(max_max) => max_max.to_string(),
                None => "unbounded".to_string(),
            },
        ));
    }

    iterators.zip_into(
        min_iterator_min_length,
        interpreter,
        output_span_range,
        &mut output,
    )?;

    Ok(output.to_value(output_span_range))
}

enum ZipIterators {
    Array(Vec<ExpressionIterator>),
    Object(Vec<(String, Span, ExpressionIterator)>),
}

impl ZipIterators {
    fn from_value(value: ExpressionValue) -> ExecutionResult<Self> {
        Ok(match value {
            ExpressionValue::Object(object) => {
                let span_range = object.span_range;
                let entries = object
                    .entries
                    .into_iter()
                    .take(101)
                    .map(|(k, v)| -> ExecutionResult<_> {
                        Ok((
                            k,
                            v.key_span,
                            v.value.expect_any_iterator("A zip iterator")?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if entries.len() == 101 {
                    return span_range.execution_err("A maximum of 100 iterators are allowed");
                }
                ZipIterators::Object(entries)
            }
            other => {
                let span_range = other.span_range();
                let iterator = other.expect_any_iterator("")
                    .map_err(|_| span_range.execution_error("Expected an object with iterable values, an array of iterables, or some other iterator of iterables."))?;
                let vec = iterator
                    .take(101)
                    .map(|x| x.expect_any_iterator("A zip iterator"))
                    .collect::<Result<Vec<_>, _>>()?;
                if vec.len() == 101 {
                    return span_range.execution_err("A maximum of 100 iterators are allowed");
                }
                ZipIterators::Array(vec)
            }
        })
    }

    /// Panics if called on an empty list of iterators
    fn size_hint_range(&self) -> (usize, Option<usize>) {
        let size_hints: Vec<_> = match self {
            ZipIterators::Array(inner) => inner.iter().map(|x| x.size_hint()).collect(),
            ZipIterators::Object(inner) => inner.iter().map(|x| x.2.size_hint()).collect(),
        };
        let min_min = size_hints.iter().map(|s| s.0).min().unwrap();
        let max_max = size_hints
            .iter()
            .map(|sh| sh.1)
            .max_by(|a, b| match (a, b) {
                (None, None) => core::cmp::Ordering::Equal,
                (None, Some(_)) => core::cmp::Ordering::Greater,
                (Some(_), None) => core::cmp::Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
            .unwrap();
        (min_min, max_max)
    }

    fn len(&self) -> usize {
        match self {
            ZipIterators::Array(inner) => inner.len(),
            ZipIterators::Object(inner) => inner.len(),
        }
    }

    /// Panics if count is larger than the minimum length of any iterator
    fn zip_into(
        &mut self,
        count: usize,
        interpreter: &mut Interpreter,
        output_span_range: SpanRange,
        output: &mut Vec<ExpressionValue>,
    ) -> ExecutionResult<()> {
        let mut counter = interpreter.start_iteration_counter(&output_span_range);

        match self {
            ZipIterators::Array(iterators) => {
                for _ in 0..count {
                    counter.increment_and_check()?;
                    let mut inner = Vec::with_capacity(iterators.len());
                    for iter in iterators.iter_mut() {
                        inner.push(iter.next().unwrap());
                    }
                    output.push(inner.to_value(output_span_range));
                }
            }
            ZipIterators::Object(iterators) => {
                for _ in 0..count {
                    counter.increment_and_check()?;
                    let mut inner = BTreeMap::new();
                    for (key, key_span, iter) in iterators.iter_mut() {
                        inner.insert(
                            key.clone(),
                            ObjectEntry {
                                key_span: *key_span,
                                value: iter.next().unwrap(),
                            },
                        );
                    }
                    output.push(inner.to_value(output_span_range));
                }
            }
        }

        Ok(())
    }
}
