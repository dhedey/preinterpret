use crate::internal_prelude::*;

/// This can be use to represent a value, or a source of that value at parse time.
///
/// For example, `SourceValue<syn::Lit>` could be used to represent a literal,
/// or a variable/command which could convert to a literal after interpretation.
#[derive(Clone)]
pub(crate) enum SourceValue<T> {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    ExpressionBlock(ExpressionBlock),
    Code(SourceCodeBlock),
    Value(T),
}

impl<T: Parse<Source>> Parse<Source> for SourceValue<T> {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => Self::Command(input.parse()?),
            SourcePeekMatch::Variable(Grouping::Grouped) => Self::GroupedVariable(input.parse()?),
            SourcePeekMatch::Variable(Grouping::Flattened) => {
                Self::FlattenedVariable(input.parse()?)
            }
            SourcePeekMatch::ExpressionBlock(_) => Self::ExpressionBlock(input.parse()?),
            SourcePeekMatch::Group(Delimiter::Brace) => Self::Code(input.parse()?),
            SourcePeekMatch::ExplicitTransformStream
            | SourcePeekMatch::AppendVariableBinding
            | SourcePeekMatch::Transformer(_) => {
                return input
                    .span()
                    .parse_err("Destructurings are not supported here")
            }
            SourcePeekMatch::Group(_)
            | SourcePeekMatch::Punct(_)
            | SourcePeekMatch::Literal(_)
            | SourcePeekMatch::Ident(_)
            | SourcePeekMatch::End => Self::Value(input.parse()?),
        })
    }
}

impl<T: Parse<Output>> Parse<Output> for SourceValue<T> {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        Ok(Self::Value(input.parse()?))
    }
}

impl<T: HasSpanRange> HasSpanRange for SourceValue<T> {
    fn span_range(&self) -> SpanRange {
        match self {
            SourceValue::Command(command) => command.span_range(),
            SourceValue::GroupedVariable(variable) => variable.span_range(),
            SourceValue::FlattenedVariable(variable) => variable.span_range(),
            SourceValue::ExpressionBlock(block) => block.span_range(),
            SourceValue::Code(code) => code.span_range(),
            SourceValue::Value(value) => value.span_range(),
        }
    }
}

impl<T: InterpretToValue<OutputValue = I>, I: Parse<Output>> InterpretToValue for SourceValue<T> {
    type OutputValue = I;

    fn interpret_to_value(self, interpreter: &mut Interpreter) -> ExecutionResult<I> {
        let descriptor = match self {
            SourceValue::Command(_) => "command output",
            SourceValue::GroupedVariable(_) => "grouped variable output",
            SourceValue::FlattenedVariable(_) => "flattened variable output",
            SourceValue::ExpressionBlock(_) => "an #(...) expression block",
            SourceValue::Code(_) => "output from the { ... } block",
            SourceValue::Value(_) => "value",
        };
        let interpreted_stream = match self {
            SourceValue::Command(command) => command.interpret_to_new_stream(interpreter)?,
            SourceValue::GroupedVariable(variable) => {
                variable.interpret_to_new_stream(interpreter)?
            }
            SourceValue::FlattenedVariable(variable) => {
                variable.interpret_to_new_stream(interpreter)?
            }
            SourceValue::ExpressionBlock(block) => block.interpret_to_new_stream(interpreter)?,
            SourceValue::Code(code) => code.interpret_to_new_stream(interpreter)?,
            SourceValue::Value(value) => return value.interpret_to_value(interpreter),
        };
        unsafe {
            // RUST-ANALYZER SAFETY: If I is a very simple parse function, this is safe.
            // Zip uses it with a parse function which does care about none-delimited groups however.
            interpreted_stream
                .parse_with(I::parse)
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

impl<T: Parse<Source>> Parse<Source> for Grouped<T> {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delimiter, delim_span, inner) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            delim_span,
            inner: inner.parse()?,
        })
    }
}

impl<T: Parse<Output>> Parse<Output> for Grouped<T> {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        let (delimiter, delim_span, inner) = input.parse_any_group()?;
        Ok(Self {
            delimiter,
            delim_span,
            inner: inner.parse()?,
        })
    }
}

impl<T, I> InterpretToValue for Grouped<T>
where
    T: InterpretToValue<OutputValue = I>,
{
    type OutputValue = Grouped<I>;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
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

impl<T: Parse<Source>> Parse<Source> for Repeated<T> {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let mut inner = vec![];
        while !input.is_empty() {
            inner.push(input.parse::<T>()?);
        }
        Ok(Self { inner })
    }
}

impl<T: Parse<Output>> Parse<Output> for Repeated<T> {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        let mut inner = vec![];
        while !input.is_empty() {
            inner.push(input.parse::<T>()?);
        }
        Ok(Self { inner })
    }
}

impl<T, I> InterpretToValue for Repeated<T>
where
    T: InterpretToValue<OutputValue = I>,
{
    type OutputValue = Repeated<I>;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let mut interpreted = Vec::with_capacity(self.inner.len());
        for item in self.inner.into_iter() {
            interpreted.push(item.interpret_to_value(interpreter)?);
        }
        Ok(Repeated { inner: interpreted })
    }
}
