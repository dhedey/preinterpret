use crate::internal_prelude::*;

pub(crate) trait IsVariable: HasSpanRange {
    fn get_name(&self) -> String;

    fn define(&self, interpreter: &mut Interpreter, value_source: impl ToExpressionValue) {
        interpreter.define_variable(self, value_source.to_value(self.span_range()))
    }

    #[allow(unused)]
    fn define_coerced(&self, interpreter: &mut Interpreter, content: OutputStream) {
        interpreter.define_variable(self, content.coerce_into_value(self.span_range()))
    }

    fn get_transparently_cloned_value(
        &self,
        interpreter: &Interpreter,
    ) -> ExecutionResult<ExpressionValue> {
        Ok(self
            .binding(interpreter)?
            .into_transparently_cloned()?
            .into())
    }

    fn substitute_into(
        &self,
        interpreter: &mut Interpreter,
        grouping: Grouping,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.binding(interpreter)?.into_shared()?.output_to(
            grouping,
            output,
            StreamOutputBehaviour::Standard,
        )
    }

    fn binding(&self, interpreter: &Interpreter) -> ExecutionResult<VariableBinding> {
        interpreter.resolve_variable_binding(self, || {
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
        self.get_transparently_cloned_value(interpreter)
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
pub(crate) struct VariableIdentifier {
    pub(crate) ident: Ident,
}

impl Parse<Source> for VariableIdentifier {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            ident: input.parse()?,
        })
    }
}

impl IsVariable for VariableIdentifier {
    fn get_name(&self) -> String {
        self.ident.to_string()
    }
}

impl HasSpan for VariableIdentifier {
    fn span(&self) -> Span {
        self.ident.span()
    }
}

impl InterpretToValue for &VariableIdentifier {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        self.get_transparently_cloned_value(interpreter)
    }
}

#[derive(Clone)]
pub(crate) struct VariablePattern {
    pub(crate) name: Ident,
}

impl Parse<Source> for VariablePattern {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            name: input.parse()?,
        })
    }
}

impl IsVariable for VariablePattern {
    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

impl HasSpan for VariablePattern {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl HandleDestructure for VariablePattern {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        self.define(interpreter, value);
        Ok(())
    }
}
