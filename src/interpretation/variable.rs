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
        if self.is_flattened {
            output.extend(self.substitute(interpreter)?);
        } else {
            output.push_new_group(self.substitute(interpreter)?, Delimiter::None, self.span());
        }
        Ok(())
    }
}

impl Express for &Variable {
    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        expression_stream.push_grouped_interpreted_stream(
            self.substitute(interpreter)?,
            self.span_range().span(),
        );
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
