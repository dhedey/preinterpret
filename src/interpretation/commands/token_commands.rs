use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct IsEmptyCommand {
    arguments: InterpretationStream,
}

impl CommandType for IsEmptyCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<TokenTree> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_to_new_stream(interpreter)?;
        Ok(Ident::new_bool(interpreted.is_empty(), output_span).into())
    }
}

#[derive(Clone)]
pub(crate) struct LengthCommand {
    arguments: InterpretationStream,
}

impl CommandType for LengthCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for LengthCommand {
    const COMMAND_NAME: &'static str = "length";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> ExecutionResult<TokenTree> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_to_new_stream(interpreter)?;
        let length_literal = Literal::usize_unsuffixed(interpreted.len()).with_span(output_span);
        Ok(length_literal.into())
    }
}

#[derive(Clone)]
pub(crate) struct GroupCommand {
    arguments: InterpretationStream,
}

impl CommandType for GroupCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for GroupCommand {
    const COMMAND_NAME: &'static str = "group";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        // The grouping happens automatically because a non-flattened
        // stream command is outputted in a group.
        self.arguments.interpret_into(interpreter, output)
    }
}

#[derive(Clone)]
pub(crate) struct IntersperseCommand {
    inputs: IntersperseInputs,
}

impl CommandType for IntersperseCommand {
    type OutputKind = OutputKindStream;
}

define_field_inputs! {
    IntersperseInputs {
        required: {
            items: CommandStreamInput = "[Hello World] or #var or [!cmd! ...]",
            separator: CommandStreamInput = "[,]" ("The token/s to add between each item"),
        },
        optional: {
            add_trailing: CommandValueInput<LitBool> = "false" ("Whether to add the separator after the last item (default: false)"),
            final_separator: CommandStreamInput = "[or]" ("Define a different final separator (default: same as normal separator)"),
        }
    }
}

impl StreamCommandDefinition for IntersperseCommand {
    const COMMAND_NAME: &'static str = "intersperse";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inputs: arguments.fully_parse_as()?,
        })
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        let items = self.inputs.items.interpret_to_new_stream(interpreter)?;
        let add_trailing = match self.inputs.add_trailing {
            Some(add_trailing) => add_trailing.interpret_to_value(interpreter)?.value(),
            None => false,
        };

        if items.is_empty() {
            return Ok(());
        }

        let mut appender = SeparatorAppender {
            separator: self.inputs.separator,
            final_separator: self.inputs.final_separator,
            add_trailing,
        };

        let mut items = items.into_iter().peekable();
        let mut this_item = items.next().unwrap(); // Safe to unwrap as non-empty
        loop {
            output.push_interpreted_item(this_item);
            let next_item = items.next();
            match next_item {
                Some(next_item) => {
                    let remaining = if items.peek().is_some() {
                        RemainingItemCount::MoreThanOne
                    } else {
                        RemainingItemCount::ExactlyOne
                    };
                    appender.add_separator(interpreter, remaining, output)?;
                    this_item = next_item;
                }
                None => {
                    appender.add_separator(interpreter, RemainingItemCount::None, output)?;
                    break;
                }
            }
        }

        Ok(())
    }
}

struct SeparatorAppender {
    separator: CommandStreamInput,
    final_separator: Option<CommandStreamInput>,
    add_trailing: bool,
}

impl SeparatorAppender {
    fn add_separator(
        &mut self,
        interpreter: &mut Interpreter,
        remaining: RemainingItemCount,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        match self.separator(remaining) {
            TrailingSeparator::Normal => self.separator.clone().interpret_into(interpreter, output),
            TrailingSeparator::Final => match self.final_separator.take() {
                Some(final_separator) => final_separator.interpret_into(interpreter, output),
                None => self.separator.clone().interpret_into(interpreter, output),
            },
            TrailingSeparator::None => Ok(()),
        }
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
    inputs: SplitInputs,
}

impl CommandType for SplitCommand {
    type OutputKind = OutputKindStream;
}

define_field_inputs! {
    SplitInputs {
        required: {
            stream: CommandStreamInput = "[...] or #var or [!cmd! ...]",
            separator: CommandStreamInput = "[::]" ("The token/s to split if they match"),
        },
        optional: {
            drop_empty_start: CommandValueInput<LitBool> = "false" ("If true, a leading separator does not yield in an empty item at the start (default: false)"),
            drop_empty_middle: CommandValueInput<LitBool> = "false" ("If true, adjacent separators do not yield an empty item between them (default: false)"),
            drop_empty_end: CommandValueInput<LitBool> = "true" ("If true, a trailing separator does not yield an empty item at the end (default: true)"),
        }
    }
}

impl StreamCommandDefinition for SplitCommand {
    const COMMAND_NAME: &'static str = "split";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            inputs: arguments.fully_parse_as()?,
        })
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        let output_span = self.inputs.stream.span();
        let stream = self.inputs.stream.interpret_to_new_stream(interpreter)?;
        let separator = self.inputs.separator.interpret_to_new_stream(interpreter)?;

        let drop_empty_start = match self.inputs.drop_empty_start {
            Some(value) => value.interpret_to_value(interpreter)?.value(),
            None => false,
        };
        let drop_empty_middle = match self.inputs.drop_empty_middle {
            Some(value) => value.interpret_to_value(interpreter)?.value(),
            None => false,
        };
        let drop_empty_end = match self.inputs.drop_empty_end {
            Some(value) => value.interpret_to_value(interpreter)?.value(),
            None => true,
        };

        handle_split(
            stream,
            output,
            output_span,
            separator.into_raw_destructure_stream(),
            drop_empty_start,
            drop_empty_middle,
            drop_empty_end,
        )
    }
}

fn handle_split(
    input: InterpretedStream,
    output: &mut InterpretedStream,
    output_span: Span,
    separator: RawDestructureStream,
    drop_empty_start: bool,
    drop_empty_middle: bool,
    drop_empty_end: bool,
) -> ExecutionResult<()> {
    unsafe {
        // RUST-ANALYZER SAFETY: This is as safe as we can get.
        // Typically the separator won't contain none-delimited groups, so we're OK
        input.syn_parse(move |input| {
            let mut current_item = InterpretedStream::new();
            let mut drop_empty_next = drop_empty_start;
            while !input.is_empty() {
                let separator_fork = input.fork();
                if separator.handle_destructure(&separator_fork).is_err() {
                    current_item.push_raw_token_tree(input.parse()?);
                    continue;
                }
                input.advance_to(&separator_fork);
                if !(current_item.is_empty() && drop_empty_next) {
                    let complete_item =
                        core::mem::replace(&mut current_item, InterpretedStream::new());
                    output.push_new_group(complete_item, Delimiter::None, output_span);
                }
                drop_empty_next = drop_empty_middle;
            }
            if !(current_item.is_empty() && drop_empty_end) {
                output.push_new_group(current_item, Delimiter::None, output_span);
            }
            Ok(())
        })
    }
}

#[derive(Clone)]
pub(crate) struct CommaSplitCommand {
    input: InterpretationStream,
}

impl CommandType for CommaSplitCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for CommaSplitCommand {
    const COMMAND_NAME: &'static str = "comma_split";

    fn parse(arguments: CommandArguments) -> ParseResult<Self> {
        Ok(Self {
            input: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        let output_span = self.input.span();
        let stream = self.input.interpret_to_new_stream(interpreter)?;
        let separator = {
            let mut stream = RawDestructureStream::empty();
            stream.push_item(RawDestructureItem::Punct(
                Punct::new(',', Spacing::Alone).with_span(output_span),
            ));
            stream
        };

        handle_split(stream, output, output_span, separator, false, false, true)
    }
}

#[derive(Clone)]
pub(crate) struct ZipCommand {
    inputs: EitherZipInput,
}

type Streams = CommandValueInput<Grouped<Repeated<CommandStreamInput>>>;

#[derive(Clone)]
enum EitherZipInput {
    Fields(ZipInputs),
    JustStream(Streams),
}

define_field_inputs! {
    ZipInputs {
        required: {
            streams: Streams = r#"([Hello Goodbye] [World Friend])"# ("A group of one or more streams to zip together. The outer brackets are used for the group."),
        },
        optional: {
            error_on_length_mismatch: CommandValueInput<syn::LitBool> = "true" ("If false, uses shortest stream length, if true, errors on unequal length. Defaults to true."),
        }
    }
}

impl CommandType for ZipCommand {
    type OutputKind = OutputKindStream;
}

impl StreamCommandDefinition for ZipCommand {
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
                "Expected [!zip! (#a #b #c)] or [!zip! {}]",
                ZipInputs::fields_description()
            ),
        )
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        let (grouped_streams, error_on_length_mismatch) = match self.inputs {
            EitherZipInput::Fields(inputs) => (inputs.streams, inputs.error_on_length_mismatch),
            EitherZipInput::JustStream(streams) => (streams, None),
        };
        let grouped_streams = grouped_streams.interpret_to_value(interpreter)?;
        let error_on_length_mismatch = match error_on_length_mismatch {
            Some(value) => value.interpret_to_value(interpreter)?.value(),
            None => true,
        };
        let Grouped {
            delimiter,
            delim_span,
            inner: Repeated { inner: streams },
        } = grouped_streams;
        if streams.is_empty() {
            return delim_span
                .join()
                .execution_err("At least one stream is required to zip");
        }
        let stream_lengths = streams
            .iter()
            .map(|stream| stream.stream.len())
            .collect::<Vec<_>>();
        let min_stream_length = *stream_lengths.iter().min().unwrap();
        if error_on_length_mismatch {
            let max_stream_length = *stream_lengths.iter().max().unwrap();
            if min_stream_length != max_stream_length {
                return delim_span.join().execution_err(format!(
                    "Streams have different lengths and zip's error_on_length_mismatch is true. The lengths vary from {} to {}",
                    min_stream_length, max_stream_length
                ));
            }
        }
        let mut iters: Vec<_> = streams
            .into_iter()
            .map(|stream| stream.stream.into_iter())
            .collect();
        for _ in 0..min_stream_length {
            let mut inner = InterpretedStream::new();
            for iter in iters.iter_mut() {
                inner.push_interpreted_item(iter.next().unwrap());
            }
            output.push_new_group(inner, delimiter, delim_span.span());
        }
        Ok(())
    }
}
