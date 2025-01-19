use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct GroupedVariable {
    marker: Token![#],
    variable_name: Ident,
}

impl Parse for GroupedVariable {
    fn parse(input: ParseStream) -> Result<Self> {
        input.try_parse_or_message(
            |input| {
                Ok(Self {
                    marker: input.parse()?,
                    variable_name: input.call(Ident::parse_any)?,
                })
            },
            "Expected #variable",
        )
    }
}

impl GroupedVariable {
    pub(crate) fn variable_name(&self) -> String {
        self.variable_name.to_string()
    }

    pub(crate) fn set(
        &self,
        interpreter: &mut Interpreter,
        value: InterpretedStream,
    ) -> Result<()> {
        interpreter.set_variable(self.variable_name(), value);
        Ok(())
    }

    pub(crate) fn get_mut<'i>(
        &self,
        interpreter: &'i mut Interpreter,
    ) -> Result<&'i mut InterpretedStream> {
        interpreter
            .get_variable_mut(&self.variable_name())
            .ok_or_else(|| self.error(format!("The variable {} wasn't already set", self)))
    }

    pub(super) fn interpret_ungrouped_contents(
        &self,
        interpreter: &Interpreter,
    ) -> Result<InterpretedStream> {
        let mut cloned = self.read_existing(interpreter)?.clone();
        cloned.set_span_range(self.span_range());
        Ok(cloned)
    }

    pub(crate) fn substitute_contents_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.read_existing(interpreter)?.append_cloned_into(output);
        Ok(())
    }

    pub(crate) fn substitute_grouped_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.push_new_group(
            self.interpret_ungrouped_contents(interpreter)?,
            Delimiter::None,
            self.span(),
        );
        Ok(())
    }

    fn read_existing<'i>(&self, interpreter: &'i Interpreter) -> Result<&'i InterpretedStream> {
        match self.read_option(interpreter) {
            Some(token_stream) => Ok(token_stream),
            None => self.span_range().err(format!(
                "The variable {} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {}]",
                self,
                self,
            )),
        }
    }

    fn read_option<'i>(&self, interpreter: &'i Interpreter) -> Option<&'i InterpretedStream> {
        interpreter.get_variable(&self.variable_name.to_string())
    }
}

impl Interpret for &GroupedVariable {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.substitute_grouped_into(interpreter, output)
    }
}

impl Express for &GroupedVariable {
    fn add_to_expression(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionBuilder,
    ) -> Result<()> {
        expression_stream.push_grouped(
            |inner| self.substitute_contents_into(interpreter, inner),
            self.span(),
        )
    }
}

impl HasSpanRange for GroupedVariable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.variable_name.span())
    }
}

impl HasSpanRange for &GroupedVariable {
    fn span_range(&self) -> SpanRange {
        GroupedVariable::span_range(self)
    }
}

impl core::fmt::Display for GroupedVariable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#..{}", self.variable_name)
    }
}

#[derive(Clone)]
pub(crate) struct FlattenedVariable {
    marker: Token![#],
    #[allow(unused)]
    flatten: Token![..],
    variable_name: Ident,
}

impl Parse for FlattenedVariable {
    fn parse(input: ParseStream) -> Result<Self> {
        input.try_parse_or_message(
            |input| {
                Ok(Self {
                    marker: input.parse()?,
                    flatten: input.parse()?,
                    variable_name: input.call(Ident::parse_any)?,
                })
            },
            "Expected #..variable",
        )
    }
}

impl FlattenedVariable {
    pub(crate) fn substitute_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.read_existing(interpreter)?.append_cloned_into(output);
        Ok(())
    }

    fn read_existing<'i>(&self, interpreter: &'i Interpreter) -> Result<&'i InterpretedStream> {
        match self.read_option(interpreter) {
            Some(token_stream) => Ok(token_stream),
            None => self.span_range().err(format!(
                "The variable {} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {}]",
                self,
                self,
            )),
        }
    }

    fn read_option<'i>(&self, interpreter: &'i Interpreter) -> Option<&'i InterpretedStream> {
        let FlattenedVariable { variable_name, .. } = self;
        interpreter.get_variable(&variable_name.to_string())
    }

    pub(crate) fn display_grouped_variable_token(&self) -> String {
        format!("#{}", self.variable_name)
    }
}

impl Interpret for &FlattenedVariable {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.substitute_into(interpreter, output)
    }
}

impl Express for &FlattenedVariable {
    fn add_to_expression(self, _: &mut Interpreter, _: &mut ExpressionBuilder) -> Result<()> {
        // Just like with commands, we throw an error in the flattened case so
        // that we can determine in future the exact structure of the expression
        // at parse time.
        self.flatten
            .err("Flattened variables cannot be used directly in expressions.\nConsider removing the .. or wrapping it inside a command such as [!group! ..] which returns an expression")
    }
}

impl HasSpanRange for FlattenedVariable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.variable_name.span())
    }
}

impl HasSpanRange for &FlattenedVariable {
    fn span_range(&self) -> SpanRange {
        FlattenedVariable::span_range(self)
    }
}

impl core::fmt::Display for FlattenedVariable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#..{}", self.variable_name)
    }
}
