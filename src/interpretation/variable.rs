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
    type OutputValue = OwnedValue;

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
    pub(crate) id: VariableDefinitionId,
}

impl ParseSource for VariableDefinition {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let ident = input.parse()?;
        let id = input.define_inactive_variable(&ident);
        Ok(Self { id })
    }
}

impl VariableDefinition {
    pub(crate) fn define(
        &self,
        interpreter: &mut Interpreter,
        value_source: impl ToExpressionValue,
    ) {
        interpreter.define_variable(self.id, value_source.into_value());
    }
}

// impl HasSpan for VariableDefinition {
//     fn span(&self) -> Span {
//         self.ident.span()
//     }
// }

#[derive(Clone)]
pub(crate) struct VariableReference {
    pub(crate) ident: Ident,
    #[allow(unused)]
    pub(crate) id: VariableReferenceId,
}

impl ParseSource for VariableReference {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let ident = input.parse()?;
        let reference_id = input.reference_variable(&ident)?;
        Ok(Self {
            ident,
            id: reference_id,
        })
    }
}

impl VariableReference {
    fn substitute_into(
        &self,
        interpreter: &mut Interpreter,
        grouping: Grouping,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.resolve_shared(interpreter)?.output_to(
            grouping,
            &mut ToStreamContext::new(output, self.span_range()),
        )
    }

    pub(crate) fn resolve_late_bound(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<LateBoundValue> {
        interpreter.resolve(self, RequestedValueOwnership::LateBound)
    }

    pub(crate) fn resolve_resolved(
        &self,
        interpreter: &mut Interpreter,
        ownership: ResolvedValueOwnership,
    ) -> ExecutionResult<ResolvedValue> {
        interpreter
            .resolve(self, RequestedValueOwnership::Concrete(ownership))?
            .resolve(ownership)
    }

    pub(crate) fn resolve_owned(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        Ok(self
            .resolve_resolved(interpreter, ResolvedValueOwnership::Owned)?
            .expect_owned())
    }

    pub(crate) fn resolve_assignee(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<MutableValue> {
        Ok(self
            .resolve_resolved(interpreter, ResolvedValueOwnership::Assignee)?
            .expect_mutable())
    }

    pub(crate) fn resolve_shared(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<SharedValue> {
        Ok(self
            .resolve_resolved(interpreter, ResolvedValueOwnership::Shared)?
            .expect_shared())
    }
}

impl HasSpan for VariableReference {
    fn span(&self) -> Span {
        self.ident.span()
    }
}

impl Evaluate for VariableReference {
    type OutputValue = OwnedValue;

    fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<Self::OutputValue> {
        self.resolve_owned(interpreter)
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

// impl HasSpan for VariablePattern {
//     fn span(&self) -> Span {
//         self.definition.span()
//     }
// }

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
