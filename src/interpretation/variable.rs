use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct Variable {
    marker: Punct, // #
    variable_name: Ident,
}

impl Variable {
    pub(super) fn parse_consuming_only_if_match(
        punct: &Punct,
        parse_stream: &mut InterpreterParseStream,
    ) -> Option<Self> {
        if punct.as_char() != '#' {
            return None;
        }
        match parse_stream.peek_token_tree() {
            Some(TokenTree::Ident(_)) => {}
            _ => return None,
        }
        match parse_stream.next_token_tree_or_end() {
            Some(TokenTree::Ident(variable_name)) => Some(Self {
                marker: punct.clone(),
                variable_name,
            }),
            _ => unreachable!("We just peeked a token of this type"),
        }
    }

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
        expression_stream.push_interpreted_group(self.substitute(interpreter)?, self.span_range());
        Ok(())
    }
}

impl HasSpanRange for Variable {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.marker.span(), self.variable_name.span())
    }
}

impl core::fmt::Display for Variable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.marker.as_char(), self.variable_name)
    }
}
