use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct EmbeddedVariable {
    marker: Token![#],
    reference: VariableReference,
}

impl ParseSource for EmbeddedVariable {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(Self {
            marker: input.parse()?,
            reference: input.parse()?,
        })
    }
}

impl Interpret for EmbeddedVariable {
    fn interpret_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.reference
            .substitute_into(interpreter, Grouping::Flattened, output)
    }
}

impl Evaluate for EmbeddedVariable {
    type OutputValue = ExpressionValue;

    fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<Self::OutputValue> {
        self.reference.evaluate(interpreter)
    }
}

impl HasSpanRange for EmbeddedVariable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.reference.span())
    }
}

#[derive(Clone)]
pub(crate) struct VariableDefinition {
    pub(crate) ident: Ident,
    #[allow(unused)]
    pub(crate) id: VariableDefinitionId,
}

impl ParseSource for VariableDefinition {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let ident = input.parse()?;
        let id = input.state(|s| s.define_variable(&ident));
        Ok(Self { ident, id })
    }
}

impl VariableDefinition {
    pub(crate) fn get_name(&self) -> String {
        self.ident.to_string()
    }

    pub(crate) fn define(
        &self,
        interpreter: &mut Interpreter,
        value_source: impl ToExpressionValue,
    ) {
        interpreter.define_variable(self, value_source.into_value())
    }
}

impl HasSpan for VariableDefinition {
    fn span(&self) -> Span {
        self.ident.span()
    }
}

#[derive(Clone)]
pub(crate) struct VariableReference {
    pub(crate) ident: Ident,
    #[allow(unused)]
    pub(crate) id: VariableReferenceId,
}

impl ParseSource for VariableReference {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let ident = input.parse()?;
        let reference_id = input.state(|s| s.reference_variable(&ident))?;
        Ok(Self {
            ident,
            id: reference_id,
        })
    }
}

impl VariableReference {
    pub(crate) fn get_name(&self) -> String {
        self.ident.to_string()
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

    pub(crate) fn binding(&self, interpreter: &Interpreter) -> ExecutionResult<VariableBinding> {
        interpreter.resolve_variable_binding(self, || {
            self.error("The variable does not already exist in the current scope")
        })
    }
}

impl HasSpan for VariableReference {
    fn span(&self) -> Span {
        self.ident.span()
    }
}

impl Evaluate for VariableReference {
    type OutputValue = ExpressionValue;

    fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<Self::OutputValue> {
        self.get_transparently_cloned_value(interpreter)
    }
}

#[derive(Clone)]
pub(crate) struct VariablePattern {
    pub(crate) definition: VariableDefinition,
}

impl ParseSource for VariablePattern {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(Self {
            definition: input.parse()?,
        })
    }
}

impl HasSpan for VariablePattern {
    fn span(&self) -> Span {
        self.definition.span()
    }
}

impl HandleDestructure for VariablePattern {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        self.definition.define(interpreter, value);
        Ok(())
    }
}
