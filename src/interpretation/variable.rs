use crate::internal_prelude::*;

pub(crate) trait IsVariable: HasSpanRange {
    fn get_name(&self) -> String;
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

impl GroupedVariable {
    pub(crate) fn set(
        &self,
        interpreter: &mut Interpreter,
        value: OutputStream,
    ) -> ExecutionResult<()> {
        interpreter.set_variable(self, value)
    }

    pub(crate) fn set_value(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        // It will be grouped on the way out; not it.
        interpreter.set_variable(self, value.into_new_output_stream(Grouping::Flattened))
    }

    pub(crate) fn get_existing_for_mutation(
        &self,
        interpreter: &Interpreter,
    ) -> ExecutionResult<VariableData> {
        Ok(interpreter
            .get_existing_variable_data(self, || {
                self.error(format!("The variable {} wasn't already set", self))
            })?
            .cheap_clone())
    }

    pub(crate) fn substitute_ungrouped_contents_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.read_existing(interpreter)?
            .get(self)?
            .append_cloned_into(output);
        Ok(())
    }

    pub(crate) fn substitute_grouped_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        output.push_new_group(
            self.read_existing(interpreter)?.get(self)?.clone(),
            Delimiter::None,
            self.span_range().join_into_span_else_start(),
        );
        Ok(())
    }

    fn read_existing<'i>(&self, interpreter: &'i Interpreter) -> ExecutionResult<&'i VariableData> {
        interpreter.get_existing_variable_data(
            self,
            || self.error(format!(
                "The variable {} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {}]",
                self,
                self,
            )),
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
        self.substitute_grouped_into(interpreter, output)
    }
}

impl InterpretToValue for &GroupedVariable {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let value = self
            .read_existing(interpreter)?
            .get(self)?
            .clone()
            .coerce_into_value(self.span_range());
        Ok(value)
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

impl FlattenedVariable {
    pub(crate) fn substitute_into(
        &self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.read_existing(interpreter)?
            .get(self)?
            .append_cloned_into(output);
        Ok(())
    }

    fn read_existing<'i>(&self, interpreter: &'i Interpreter) -> ExecutionResult<&'i VariableData> {
        interpreter.get_existing_variable_data(
            self,
            || self.error(format!(
                "The variable {} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {}]",
                self.variable_name,
                self,
            )),
        )
    }

    pub(crate) fn display_grouped_variable_token(&self) -> String {
        format!("#{}", self.variable_name)
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
        self.substitute_into(interpreter, output)
    }
}

impl InterpretToValue for &FlattenedVariable {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let mut output_stream = OutputStream::new();
        self.substitute_into(interpreter, &mut output_stream)?;
        Ok(output_stream.to_value(self.span_range()))
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

// An identifier for a variable path in an expression
#[derive(Clone)]
pub(crate) struct VariablePath {
    root: Ident,
    fields: Vec<(Token![.], Ident)>,
}

impl Parse<Source> for VariablePath {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            root: input.parse()?,
            fields: {
                let mut fields = vec![];
                while input.peek(Token![.]) {
                    fields.push((input.parse()?, input.parse()?));
                }
                fields
            },
        })
    }
}

impl VariablePath {
    pub(crate) fn set_value(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        // It will be grouped on the way out, not in
        interpreter.set_variable(self, value.into_new_output_stream(Grouping::Flattened))
    }
}

impl IsVariable for VariablePath {
    fn get_name(&self) -> String {
        self.root.to_string()
    }
}

impl HasSpanRange for VariablePath {
    fn span_range(&self) -> SpanRange {
        match self.fields.last() {
            Some((_, ident)) => SpanRange::new_between(self.root.span(), ident.span()),
            None => self.root.span_range(),
        }
    }
}

impl InterpretToValue for &VariablePath {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let value = interpreter
            .get_existing_variable_data(self, || {
                self.error(format!("The variable {} wasn't set.", &self.root,))
            })?
            .get(self)?
            .clone()
            .coerce_into_value(self.span_range());
        Ok(value)
    }
}
