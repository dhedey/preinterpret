use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum InterpretationItem {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    InterpretationGroup(InterpretationGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl ParseFromSource for InterpretationItem {
    fn parse_from_source(input: SourceParseStream) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            GrammarPeekMatch::Command(_) => InterpretationItem::Command(input.parse()?),
            GrammarPeekMatch::Group(_) => InterpretationItem::InterpretationGroup(input.parse()?),
            GrammarPeekMatch::GroupedVariable => {
                InterpretationItem::GroupedVariable(input.parse()?)
            }
            GrammarPeekMatch::FlattenedVariable => {
                InterpretationItem::FlattenedVariable(input.parse()?)
            }
            GrammarPeekMatch::AppendVariableDestructuring | GrammarPeekMatch::Destructurer(_) => {
                return input.parse_err("Destructurings are not supported here")
            }
            GrammarPeekMatch::Punct(_) => InterpretationItem::Punct(input.parse_any_punct()?),
            GrammarPeekMatch::Ident(_) => InterpretationItem::Ident(input.parse_any_ident()?),
            GrammarPeekMatch::Literal(_) => InterpretationItem::Literal(input.parse()?),
            GrammarPeekMatch::End => return input.parse_err("Expected some item"),
        })
    }
}

impl Interpret for InterpretationItem {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        match self {
            InterpretationItem::Command(command_invocation) => {
                command_invocation.interpret_into(interpreter, output)?;
            }
            InterpretationItem::GroupedVariable(variable) => {
                variable.interpret_into(interpreter, output)?;
            }
            InterpretationItem::FlattenedVariable(variable) => {
                variable.interpret_into(interpreter, output)?;
            }
            InterpretationItem::InterpretationGroup(group) => {
                group.interpret_into(interpreter, output)?;
            }
            InterpretationItem::Punct(punct) => output.push_punct(punct),
            InterpretationItem::Ident(ident) => output.push_ident(ident),
            InterpretationItem::Literal(literal) => output.push_literal(literal),
        }
        Ok(())
    }
}

impl HasSpanRange for InterpretationItem {
    fn span_range(&self) -> SpanRange {
        match self {
            InterpretationItem::Command(command_invocation) => command_invocation.span_range(),
            InterpretationItem::FlattenedVariable(variable) => variable.span_range(),
            InterpretationItem::GroupedVariable(variable) => variable.span_range(),
            InterpretationItem::InterpretationGroup(group) => group.span_range(),
            InterpretationItem::Punct(punct) => punct.span_range(),
            InterpretationItem::Ident(ident) => ident.span_range(),
            InterpretationItem::Literal(literal) => literal.span_range(),
        }
    }
}
