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

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.reference.control_flow_pass(context)
    }
}

impl Interpret for EmbeddedVariable {
    fn interpret(&self, interpreter: &mut Interpreter) -> ExecutionResult<()> {
        self.reference
            .substitute_into_output(interpreter, Grouping::Flattened)
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
    pub(crate) id: VariableDefinitionId,
}

impl ParseSource for VariableDefinition {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let ident: Ident = input.parse()?;

        let ident_str = ident.to_string();

        // Ident::parse() already errors on rust identifiers, so we only need
        // to check preinterpret-exclusive keywords here.
        if is_keyword(ident_str.as_str()) {
            return ident.parse_err(format!(
                "Cannot use preinterpret keyword `{}` as a variable name",
                ident_str
            ));
        }

        let id = VariableDefinitionId::new_placeholder();
        Ok(Self { ident, id })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        context.register_variable_definition(&self.ident, &mut self.id);
        context.define_variable(self.id);
        Ok(())
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

#[derive(Clone)]
pub(crate) struct VariableReference {
    pub(crate) ident: Ident,
    #[allow(unused)]
    pub(crate) id: VariableReferenceId,
    #[cfg(feature = "debug")]
    pub(crate) assertion: FinalUseAssertion,
}

impl ParseSource for VariableReference {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let ident = input.parse()?;
        #[cfg(feature = "debug")]
        let assertion = {
            if let Some((_, next)) = input.cursor().punct_matching(':') {
                if let Some(_) = next.ident_matching("FINAL") {
                    let _ = input.parse_punct_matching(':')?;
                    let ident = input.parse_any_ident()?;
                    FinalUseAssertion::IsFinal(ident.span())
                } else if let Some(_) = next.ident_matching("NONFINAL") {
                    let _ = input.parse_punct_matching(':')?;
                    let ident = input.parse_any_ident()?;
                    FinalUseAssertion::IsNotFinal(ident.span())
                } else {
                    FinalUseAssertion::None
                }
            } else {
                FinalUseAssertion::None
            }
        };
        Ok(Self {
            ident,
            id: VariableReferenceId::new_placeholder(),
            #[cfg(feature = "debug")]
            assertion,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        context.register_variable_reference(&self.ident, &mut self.id);
        context.reference_variable(
            self.id,
            #[cfg(feature = "debug")]
            self.assertion,
        )
    }
}

impl VariableReference {
    fn substitute_into_output(
        &self,
        interpreter: &mut Interpreter,
        grouping: Grouping,
    ) -> ExecutionResult<()> {
        let value = self.resolve_shared(interpreter)?;
        value.output_to(
            grouping,
            &mut ToStreamContext::new(interpreter.output(self)?, self.span_range()),
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

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.definition.control_flow_pass(context)
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
