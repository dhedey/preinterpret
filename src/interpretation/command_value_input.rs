use crate::internal_prelude::*;

/// This can be use to represent a value, or a source of that value at parse time.
///
/// For example, `CommandValueInput<syn::Lit>` could be used to represent a literal,
/// or a variable/command which could convert to a literal after interpretation.
#[derive(Clone)]
pub(crate) enum CommandValueInput<T> {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    Code(CommandCodeInput),
    Value(T),
}

impl<T: ParseFromSource> ParseFromSource for CommandValueInput<T> {
    fn parse_from_source(input: SourceParseStream) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            GrammarPeekMatch::Command(_) => Self::Command(input.parse()?),
            GrammarPeekMatch::GroupedVariable => Self::GroupedVariable(input.parse()?),
            GrammarPeekMatch::FlattenedVariable => Self::FlattenedVariable(input.parse()?),
            GrammarPeekMatch::Group(Delimiter::Brace) => Self::Code(input.parse()?),
            GrammarPeekMatch::AppendVariableDestructuring | GrammarPeekMatch::Destructurer(_) => {
                return input
                    .span()
                    .parse_err("Destructurings are not supported here")
            }
            GrammarPeekMatch::Group(_)
            | GrammarPeekMatch::Punct(_)
            | GrammarPeekMatch::Literal(_)
            | GrammarPeekMatch::Ident(_)
            | GrammarPeekMatch::End => Self::Value(input.parse()?),
        })
    }
}

impl<T: ParseFromInterpreted> ParseFromInterpreted for CommandValueInput<T> {
    fn parse_from_interpreted(input: InterpretedParseStream) -> ParseResult<Self> {
        Ok(Self::Value(input.parse()?))
    }
}

impl<T: HasSpanRange> HasSpanRange for CommandValueInput<T> {
    fn span_range(&self) -> SpanRange {
        match self {
            CommandValueInput::Command(command) => command.span_range(),
            CommandValueInput::GroupedVariable(variable) => variable.span_range(),
            CommandValueInput::FlattenedVariable(variable) => variable.span_range(),
            CommandValueInput::Code(code) => code.span_range(),
            CommandValueInput::Value(value) => value.span_range(),
        }
    }
}

impl<T: InterpretValue<InterpretedValue = I>, I: ParseFromInterpreted> InterpretValue
    for CommandValueInput<T>
{
    type InterpretedValue = I;

    fn interpret_to_value(self, interpreter: &mut Interpreter) -> ExecutionResult<I> {
        let descriptor = match self {
            CommandValueInput::Command(_) => "command output",
            CommandValueInput::GroupedVariable(_) => "grouped variable output",
            CommandValueInput::FlattenedVariable(_) => "flattened variable output",
            CommandValueInput::Code(_) => "output from the { ... } block",
            CommandValueInput::Value(_) => "value",
        };
        let interpreted_stream = match self {
            CommandValueInput::Command(command) => command.interpret_to_new_stream(interpreter)?,
            CommandValueInput::GroupedVariable(variable) => {
                variable.interpret_to_new_stream(interpreter)?
            }
            CommandValueInput::FlattenedVariable(variable) => {
                variable.interpret_to_new_stream(interpreter)?
            }
            CommandValueInput::Code(code) => code.interpret_to_new_stream(interpreter)?,
            CommandValueInput::Value(value) => return value.interpret_to_value(interpreter),
        };
        unsafe {
            // RUST-ANALYZER SAFETY: We only use I with simple parse functions so far which don't care about
            // none-delimited groups
            interpreted_stream
                .parse_with(I::parse_from_interpreted)
                .add_context_if_error_and_no_context(|| {
                    format!(
                        "Occurred whilst parsing the {} to a {}.",
                        descriptor,
                        std::any::type_name::<I>()
                    )
                })
                .into_execution_result()
        }
    }
}

#[derive(Clone)]
pub(crate) struct Grouped<T> {
    pub(crate) delimiter: Delimiter,
    pub(crate) delim_span: DelimSpan,
    pub(crate) inner: T,
}

impl<T: ParseFromSource> ParseFromSource for Grouped<T> {
    fn parse_from_source(input: SourceParseStream) -> ParseResult<Self> {
        let (delimiter, delim_span, inner) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            delim_span,
            inner: inner.parse()?,
        })
    }
}

impl<T: ParseFromInterpreted> ParseFromInterpreted for Grouped<T> {
    fn parse_from_interpreted(input: InterpretedParseStream) -> ParseResult<Self> {
        let (delimiter, delim_span, inner) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            delim_span,
            inner: inner.parse()?,
        })
    }
}

impl<T, I> InterpretValue for Grouped<T>
where
    T: InterpretValue<InterpretedValue = I>,
{
    type InterpretedValue = Grouped<I>;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::InterpretedValue> {
        Ok(Grouped {
            delimiter: self.delimiter,
            delim_span: self.delim_span,
            inner: self.inner.interpret_to_value(interpreter)?,
        })
    }
}

#[derive(Clone)]
pub(crate) struct Repeated<T> {
    pub(crate) inner: Vec<T>,
}

impl<T: ParseFromSource> ParseFromSource for Repeated<T> {
    fn parse_from_source(input: SourceParseStream) -> ParseResult<Self> {
        let mut inner = vec![];
        while !input.is_empty() {
            inner.push(input.parse::<T>()?);
        }
        Ok(Self { inner })
    }
}

impl<T: ParseFromInterpreted> ParseFromInterpreted for Repeated<T> {
    fn parse_from_interpreted(input: InterpretedParseStream) -> ParseResult<Self> {
        let mut inner = vec![];
        while !input.is_empty() {
            inner.push(input.parse::<T>()?);
        }
        Ok(Self { inner })
    }
}

impl<T, I> InterpretValue for Repeated<T>
where
    T: InterpretValue<InterpretedValue = I>,
{
    type InterpretedValue = Repeated<I>;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::InterpretedValue> {
        let mut interpreted = Vec::with_capacity(self.inner.len());
        for item in self.inner.into_iter() {
            interpreted.push(item.interpret_to_value(interpreter)?);
        }
        Ok(Repeated { inner: interpreted })
    }
}
