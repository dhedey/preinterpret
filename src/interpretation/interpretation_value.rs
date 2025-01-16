use crate::internal_prelude::*;

/// This can be use to represent a value, or a source of that value at parse time.
///
/// For example, `InterpretationValue<syn::Lit>` could be used to represent a literal,
/// or a variable/command which could convert to a literal after interpretation.
#[derive(Clone)]
pub(crate) enum InterpretationValue<T> {
    Command(Command),
    Variable(Variable),
    Value(T),
}

impl<T: Parse> Parse for InterpretationValue<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        if let Ok(command) = fork.parse() {
            input.advance_to(&fork);
            return Ok(InterpretationValue::Command(command));
        }
        let fork = input.fork();
        if let Ok(command) = fork.parse() {
            input.advance_to(&fork);
            return Ok(InterpretationValue::Variable(command));
        }
        Ok(InterpretationValue::Value(input.parse()?))
    }
}

impl<T: HasSpanRange> HasSpanRange for InterpretationValue<T> {
    fn span_range(&self) -> SpanRange {
        match self {
            InterpretationValue::Command(command) => command.span_range(),
            InterpretationValue::Variable(variable) => variable.span_range(),
            InterpretationValue::Value(value) => value.span_range(),
        }
    }
}

impl<T: InterpretValue<InterpretedValue = I>, I: Parse> InterpretValue for InterpretationValue<T> {
    type InterpretedValue = I;

    fn interpret(self, interpreter: &mut Interpreter) -> Result<I> {
        match self {
            InterpretationValue::Command(command) => command
                .interpret_as_tokens(interpreter)?
                .syn_parse(I::parse),
            InterpretationValue::Variable(variable) => variable
                .interpret_as_tokens(interpreter)?
                .syn_parse(I::parse),
            InterpretationValue::Value(value) => value.interpret(interpreter),
        }
    }
}
