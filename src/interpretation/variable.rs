use crate::internal_prelude::*;

pub(crate) trait IsVariable: HasSpanRange {
    fn get_name(&self) -> String;

    fn define(&self, interpreter: &mut Interpreter, value_source: impl ToExpressionValue) {
        interpreter.define_variable(self, value_source.into_value())
    }

    #[allow(unused)]
    fn define_coerced(&self, interpreter: &mut Interpreter, content: OutputStream) {
        interpreter.define_variable(self, content.coerce_into_value())
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
            &mut ToStreamContext::new(output, self.span_range()),
        )
    }

    fn binding(&self, interpreter: &Interpreter) -> ExecutionResult<VariableBinding> {
        interpreter.resolve_variable_binding(self, || {
            self.error("The variable does not already exist in the current scope")
        })
    }
}

#[derive(Clone)]
pub(crate) struct EmbeddedVariable {
    marker: Token![#],
    variable_name: Ident,
}

impl Parse<Source> for EmbeddedVariable {
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

impl IsVariable for EmbeddedVariable {
    fn get_name(&self) -> String {
        self.variable_name.to_string()
    }
}

impl Interpret for &EmbeddedVariable {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.substitute_into(interpreter, Grouping::Flattened, output)
    }
}

impl InterpretToValue for &EmbeddedVariable {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        self.get_transparently_cloned_value(interpreter)
    }
}

impl HasSpanRange for EmbeddedVariable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.variable_name.span())
    }
}

impl HasSpanRange for &EmbeddedVariable {
    fn span_range(&self) -> SpanRange {
        <EmbeddedVariable as HasSpanRange>::span_range(self)
    }
}

impl core::fmt::Display for EmbeddedVariable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.variable_name)
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
