use crate::internal_prelude::*;

/// This can be use to represent a value, or a source of that value at parse time.
///
/// For example, `CommandValueInput<syn::Lit>` could be used to represent a literal,
/// or a variable/command which could convert to a literal after interpretation.
#[derive(Clone)]
pub(crate) enum CommandValueInput<T> {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    Code(CommandCodeInput),
    Value(T),
}

impl<T: Parse> Parse for CommandValueInput<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(match detect_preinterpret_grammar(input.cursor()) {
            PeekMatch::GroupedCommand => Self::Command(input.parse()?),
            PeekMatch::FlattenedCommand => Self::Command(input.parse()?),
            PeekMatch::GroupedVariable => Self::GroupedVariable(input.parse()?),
            PeekMatch::FlattenedVariable => Self::FlattenedVariable(input.parse()?),
            PeekMatch::InterpretationGroup(Delimiter::Brace) => Self::Code(input.parse()?),
            PeekMatch::InterpretationGroup(_) | PeekMatch::Other => Self::Value(input.parse()?),
        })
    }
}

impl<T: HasSpanRange> HasSpanRange for CommandValueInput<T> {
    fn span_range(&self) -> SpanRange {
        match self {
            CommandValueInput::Command(command) => command.span_range(),
            CommandValueInput::GroupedVariable(variable) => variable.span_range(),
            CommandValueInput::FlattenedVariable(variable) => variable.span_range(),
            CommandValueInput::Code(code) => code.span_range(),
            CommandValueInput::Value(value) => value.span_range(),
        }
    }
}

impl<T: InterpretValue<InterpretedValue = I>, I: Parse> InterpretValue for CommandValueInput<T> {
    type InterpretedValue = I;

    fn interpret(self, interpreter: &mut Interpreter) -> Result<I> {
        let descriptor = match self {
            CommandValueInput::Command(_) => "command output",
            CommandValueInput::GroupedVariable(_) => "grouped variable output",
            CommandValueInput::FlattenedVariable(_) => "flattened variable output",
            CommandValueInput::Code(_) => "output from the { ... } block",
            CommandValueInput::Value(_) => "value",
        };
        let interpreted_stream = match self {
            CommandValueInput::Command(command) => command.interpret_to_new_stream(interpreter)?,
            CommandValueInput::GroupedVariable(variable) => {
                variable.interpret_to_new_stream(interpreter)?
            }
            CommandValueInput::FlattenedVariable(variable) => {
                variable.interpret_to_new_stream(interpreter)?
            }
            CommandValueInput::Code(code) => code.interpret_to_new_stream(interpreter)?,
            CommandValueInput::Value(value) => return value.interpret(interpreter),
        };
        match interpreted_stream.syn_parse(I::parse) {
            Ok(value) => Ok(value),
            Err(err) => Err(err.concat(&format!(
                "\nOccurred whilst parsing the {} to a {}.",
                descriptor,
                std::any::type_name::<I>()
            ))),
        }
    }
}
