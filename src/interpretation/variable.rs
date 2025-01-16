use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct Variable {
    marker: Token![#],
    is_flattened: bool,
    variable_name: Ident,
}

impl Parse for Variable {
    fn parse(input: ParseStream) -> Result<Self> {
        let marker = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident::peek_any) {
            return Ok(Self {
                marker,
                is_flattened: false,
                variable_name: input.parse()?,
            });
        }
        if lookahead.peek(Token![..]) {
            let _ = input.parse::<Token![..]>();
            return Ok(Self {
                marker,
                is_flattened: true,
                variable_name: input.parse()?,
            });
        }
        Err(lookahead.error())
    }
}

impl Variable {
    pub(crate) fn variable_name(&self) -> String {
        self.variable_name.to_string()
    }

    pub(crate) fn is_flattened(&self) -> bool {
        self.is_flattened
    }

    pub(crate) fn set(&self, interpreter: &mut Interpreter, value: InterpretedStream) {
        interpreter.set_variable(self.variable_name(), value);
    }

    pub(super) fn interpret_into_stream(
        &self,
        interpreter: &Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.read_existing(interpreter)?.append_cloned_into(output);
        Ok(())
    }

    pub(super) fn interpret_as_new_stream(
        &self,
        interpreter: &Interpreter,
    ) -> Result<InterpretedStream> {
        let mut cloned = self.read_existing(interpreter)?.clone();
        cloned.set_span_range(self.span_range());
        Ok(cloned)
    }

    pub(crate) fn substitute_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        if self.is_flattened {
            self.interpret_into_stream(interpreter, output)
        } else {
            output.push_new_group(
                self.interpret_as_new_stream(interpreter)?,
                Delimiter::None,
                self.span(),
            );
            Ok(())
        }
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
        let Variable { variable_name, .. } = self;
        interpreter.get_variable(&variable_name.to_string())
    }

    pub(crate) fn display_unflattened_variable_token(&self) -> String {
        format!("#{}", self.variable_name)
    }
}

impl Interpret for &Variable {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.substitute_into(interpreter, output)
    }
}

impl Express for &Variable {
    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        expression_stream.push_grouped_interpreted_stream(
            self.interpret_as_new_stream(interpreter)?,
            self.span(),
        );
        Ok(())
    }
}

impl HasSpanRange for Variable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span, self.variable_name.span())
    }
}

impl HasSpanRange for &Variable {
    fn span_range(&self) -> SpanRange {
        Variable::span_range(self)
    }
}

impl core::fmt::Display for Variable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.variable_name)
    }
}
