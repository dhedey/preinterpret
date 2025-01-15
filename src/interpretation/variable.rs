use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct Variable {
    marker: Token![#],
    variable_name: Ident,
}

impl Parse for Variable {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            marker: input.parse()?,
            variable_name: input.parse()?,
        })
    }
}

impl Variable {
    pub(crate) fn variable_name(&self) -> String {
        self.variable_name.to_string()
    }

    pub(crate) fn set(&self, interpreter: &mut Interpreter, value: InterpretedStream) {
        interpreter.set_variable(self.variable_name(), value);
    }

    fn substitute(&self, interpreter: &Interpreter) -> Result<InterpretedStream> {
        Ok(self.read_or_else(
            interpreter,
            || format!(
                "The variable {} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {}]",
                self,
                self,
            )
        )?.clone())
    }

    fn read_or_else<'i>(
        &self,
        interpreter: &'i Interpreter,
        create_error: impl FnOnce() -> String,
    ) -> Result<&'i InterpretedStream> {
        match self.read_option(interpreter) {
            Some(token_stream) => Ok(token_stream),
            None => self.span_range().err(create_error()),
        }
    }

    fn read_option<'i>(&self, interpreter: &'i Interpreter) -> Option<&'i InterpretedStream> {
        let Variable { variable_name, .. } = self;
        interpreter.get_variable(&variable_name.to_string())
    }
}

impl Interpret for &Variable {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.extend(self.substitute(interpreter)?);
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        expression_stream
            .push_grouped_interpreted_stream(self.substitute(interpreter)?, self.span_range().span());
        Ok(())
    }
}

impl HasSpanRange for &Variable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.variable_name.span())
    }
}

impl core::fmt::Display for Variable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.variable_name)
    }
}
