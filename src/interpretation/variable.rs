use crate::internal_prelude::*;

pub(crate) trait IsVariable: HasSpanRange {
    fn get_name(&self) -> String;

    fn set_stream(
        &self,
        interpreter: &mut Interpreter,
        stream: OutputStream,
    ) -> ExecutionResult<()> {
        interpreter.set_variable(self, stream.to_value(self.span_range()))
    }

    fn set_coerced_stream(
        &self,
        interpreter: &mut Interpreter,
        stream: OutputStream,
    ) -> ExecutionResult<()> {
        interpreter.set_variable(self, stream.coerce_into_value(self.span_range()))
    }

    fn set_value(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        interpreter.set_variable(self, value)
    }

    fn get_existing_for_mutation(
        &self,
        interpreter: &Interpreter,
    ) -> ExecutionResult<VariableData> {
        Ok(self.read_existing(interpreter)?.cheap_clone())
    }

    fn get_value(&self, interpreter: &Interpreter) -> ExecutionResult<ExpressionValue> {
        let value = self
            .read_existing(interpreter)?
            .get(self)?
            .clone()
            .with_span_range(self.span_range());
        Ok(value)
    }

    fn substitute_into(
        &self,
        interpreter: &mut Interpreter,
        grouping: Grouping,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.read_existing(interpreter)?.get(self)?.output_to(
            grouping,
            output,
            StreamOutputBehaviour::Standard,
        )
    }

    fn read_existing<'i>(&self, interpreter: &'i Interpreter) -> ExecutionResult<&'i VariableData> {
        interpreter.get_existing_variable_data(self, || {
            self.error("The variable does not already exist in the current scope")
        })
    }
}

#[derive(Clone)]
pub(crate) enum MarkedVariable {
    Grouped(GroupedVariable),
    Flattened(FlattenedVariable),
}

impl Parse<Source> for MarkedVariable {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        if input.peek2(Token![..]) {
            Ok(MarkedVariable::Flattened(input.parse()?))
        } else {
            Ok(MarkedVariable::Grouped(input.parse()?))
        }
    }
}

impl HasSpanRange for MarkedVariable {
    fn span_range(&self) -> SpanRange {
        match self {
            MarkedVariable::Grouped(variable) => variable.span_range(),
            MarkedVariable::Flattened(variable) => variable.span_range(),
        }
    }
}

impl HasSpanRange for &MarkedVariable {
    fn span_range(&self) -> SpanRange {
        <MarkedVariable as HasSpanRange>::span_range(self)
    }
}

impl IsVariable for MarkedVariable {
    fn get_name(&self) -> String {
        match self {
            MarkedVariable::Grouped(variable) => variable.get_name(),
            MarkedVariable::Flattened(variable) => variable.get_name(),
        }
    }
}

impl Interpret for &MarkedVariable {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            MarkedVariable::Grouped(variable) => variable.interpret_into(interpreter, output),
            MarkedVariable::Flattened(variable) => variable.interpret_into(interpreter, output),
        }
    }
}

#[derive(Clone)]
pub(crate) struct GroupedVariable {
    marker: Token![#],
    variable_name: Ident,
}

impl Parse<Source> for GroupedVariable {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        input.try_parse_or_error(
            |input| {
                Ok(Self {
                    marker: input.parse()?,
                    variable_name: input.parse_any_ident()?,
                })
            },
            "Expected #variable",
        )
    }
}

impl IsVariable for GroupedVariable {
    fn get_name(&self) -> String {
        self.variable_name.to_string()
    }
}

impl Interpret for &GroupedVariable {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.substitute_into(interpreter, Grouping::Grouped, output)
    }
}

impl InterpretToValue for &GroupedVariable {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        self.get_value(interpreter)
    }
}

impl HasSpanRange for GroupedVariable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.variable_name.span())
    }
}

impl HasSpanRange for &GroupedVariable {
    fn span_range(&self) -> SpanRange {
        <GroupedVariable as HasSpanRange>::span_range(self)
    }
}

impl core::fmt::Display for GroupedVariable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.variable_name)
    }
}

#[derive(Clone)]
pub(crate) struct FlattenedVariable {
    marker: Token![#],
    #[allow(unused)]
    flatten: Token![..],
    variable_name: Ident,
}

impl Parse<Source> for FlattenedVariable {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        input.try_parse_or_error(
            |input| {
                Ok(Self {
                    marker: input.parse()?,
                    flatten: input.parse()?,
                    variable_name: input.parse_any_ident()?,
                })
            },
            "Expected #..variable",
        )
    }
}

impl IsVariable for FlattenedVariable {
    fn get_name(&self) -> String {
        self.variable_name.to_string()
    }
}

impl Interpret for &FlattenedVariable {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.substitute_into(interpreter, Grouping::Flattened, output)
    }
}

impl HasSpanRange for FlattenedVariable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.variable_name.span())
    }
}

impl HasSpanRange for &FlattenedVariable {
    fn span_range(&self) -> SpanRange {
        <FlattenedVariable as HasSpanRange>::span_range(self)
    }
}

impl core::fmt::Display for FlattenedVariable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#..{}", self.variable_name)
    }
}

// An identifier for a variable path in an expression
#[derive(Clone)]
pub(crate) struct VariablePath {
    root: Ident,
    fields: Vec<(Token![.], Ident)>,
}

impl Parse<Source> for VariablePath {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            root: input.parse()?,
            fields: {
                let mut fields = vec![];
                while input.peek(Token![.]) {
                    fields.push((input.parse()?, input.parse()?));
                }
                fields
            },
        })
    }
}

impl IsVariable for VariablePath {
    fn get_name(&self) -> String {
        self.root.to_string()
    }
}

impl HasSpanRange for VariablePath {
    fn span_range(&self) -> SpanRange {
        match self.fields.last() {
            Some((_, ident)) => SpanRange::new_between(self.root.span(), ident.span()),
            None => self.root.span_range(),
        }
    }
}

impl InterpretToValue for &VariablePath {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        self.get_value(interpreter)
    }
}

#[derive(Clone)]
pub(crate) struct VariableDestructuring {
    name: Ident,
}

impl Parse<Source> for VariableDestructuring {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            name: input.parse()?,
        })
    }
}

impl IsVariable for VariableDestructuring {
    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

impl HasSpan for VariableDestructuring {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl HandleDestructure for VariableDestructuring {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        self.set_value(interpreter, value)
    }
}
