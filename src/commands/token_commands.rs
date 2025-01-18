use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EmptyCommand;

impl CommandDefinition for EmptyCommand {
    const COMMAND_NAME: &'static str = "empty";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.assert_empty(
            "The !empty! command does not take any arguments. Perhaps you want !is_empty! instead?",
        )?;
        Ok(Self)
    }
}

impl CommandInvocation for EmptyCommand {
    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<CommandOutput> {
        Ok(CommandOutput::Empty)
    }
}

#[derive(Clone)]
pub(crate) struct IsEmptyCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for IsEmptyCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
        Ok(CommandOutput::Ident(Ident::new_bool(
            interpreted.is_empty(),
            output_span,
        )))
    }
}

#[derive(Clone)]
pub(crate) struct LengthCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for LengthCommand {
    const COMMAND_NAME: &'static str = "length";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for LengthCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
        let stream_length = interpreted.into_token_stream().into_iter().count();
        let length_literal = Literal::usize_unsuffixed(stream_length).with_span(output_span);
        Ok(CommandOutput::Literal(length_literal))
    }
}

#[derive(Clone)]
pub(crate) struct GroupCommand {
    arguments: InterpretationStream,
}

impl CommandDefinition for GroupCommand {
    const COMMAND_NAME: &'static str = "group";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }
}

impl CommandInvocation for GroupCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let mut output = InterpretedStream::new(self.arguments.span_range());
        let group_span = self.arguments.span();
        let inner = self.arguments.interpret_as_tokens(interpreter)?;
        output.push_new_group(inner, Delimiter::None, group_span);
        Ok(CommandOutput::Stream(output))
    }
}

#[derive(Clone)]
pub(crate) struct IntersperseCommand {
    span_range: SpanRange,
    inputs: IntersperseInputs,
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

impl CommandDefinition for IntersperseCommand {
    const COMMAND_NAME: &'static str = "intersperse";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            span_range: arguments.full_span_range(),
            inputs: arguments.fully_parse_as()?,
        })
    }
}

impl CommandInvocation for IntersperseCommand {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput> {
        let items = self
            .inputs
            .items
            .interpret_as_tokens(interpreter)?
            .into_token_stream();
        let add_trailing = match self.inputs.add_trailing {
            Some(add_trailing) => add_trailing.interpret(interpreter)?.value(),
            None => false,
        };

        let mut output = InterpretedStream::new(self.span_range);

        if items.is_empty() {
            return Ok(CommandOutput::Stream(output));
        }

        let mut appender = SeparatorAppender {
            separator: self.inputs.separator,
            final_separator: self.inputs.final_separator,
            add_trailing,
        };

        let mut items = items.into_iter().peekable();
        let mut this_item = items.next().unwrap(); // Safe to unwrap as non-empty
        loop {
            output.push_raw_token_tree(this_item);
            let next_item = items.next();
            match next_item {
                Some(next_item) => {
                    let remaining = if items.peek().is_some() {
                        RemainingItemCount::MoreThanOne
                    } else {
                        RemainingItemCount::ExactlyOne
                    };
                    appender.add_separator(interpreter, remaining, &mut output)?;
                    this_item = next_item;
                }
                None => {
                    appender.add_separator(interpreter, RemainingItemCount::None, &mut output)?;
                    break;
                }
            }
        }

        Ok(CommandOutput::Stream(output))
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
    ) -> Result<()> {
        let extender = match self.separator(remaining) {
            TrailingSeparator::Normal => self.separator.clone().interpret_as_tokens(interpreter)?,
            TrailingSeparator::Final => match self.final_separator.take() {
                Some(final_separator) => final_separator.interpret_as_tokens(interpreter)?,
                None => self.separator.clone().interpret_as_tokens(interpreter)?,
            },
            TrailingSeparator::None => InterpretedStream::new(SpanRange::ignored()),
        };
        output.extend(extender);
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
