use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EmptyCommand;

impl CommandType for EmptyCommand {
    type OutputKind = OutputKindNone;
}

impl NoOutputCommandDefinition for EmptyCommand {
    const COMMAND_NAME: &'static str = "empty";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        arguments.assert_empty(
            "The !empty! command does not take any arguments. Perhaps you want !is_empty! instead?",
        )?;
        Ok(Self)
    }

    fn execute(self: Box<Self>, _interpreter: &mut Interpreter) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct IsEmptyCommand {
    arguments: InterpretationStream,
}

impl CommandType for IsEmptyCommand {
    type OutputKind = OutputKindValue;
}

impl ValueCommandDefinition for IsEmptyCommand {
    const COMMAND_NAME: &'static str = "is_empty";

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<TokenTree> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
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

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<TokenTree> {
        let output_span = self.arguments.span_range().span();
        let interpreted = self.arguments.interpret_as_tokens(interpreter)?;
        let stream_length = interpreted.into_token_stream().into_iter().count();
        let length_literal = Literal::usize_unsuffixed(stream_length).with_span(output_span);
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

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            arguments: arguments.parse_all_for_interpretation()?,
        })
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let group_span = self.arguments.span();
        output.push_grouped(
            |inner| self.arguments.interpret_as_tokens_into(interpreter, inner),
            Delimiter::None,
            group_span,
        )
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

    fn parse(arguments: CommandArguments) -> Result<Self> {
        Ok(Self {
            inputs: arguments.fully_parse_as()?,
        })
    }

    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        let items = self
            .inputs
            .items
            .interpret_as_tokens(interpreter)?
            .into_token_stream();
        let add_trailing = match self.inputs.add_trailing {
            Some(add_trailing) => add_trailing.interpret(interpreter)?.value(),
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
            output.push_raw_token_tree(this_item);
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
