use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum SourceItem {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    SourceGroup(SourceGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl Parse<Source> for SourceItem {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(match input.peek_grammar() {
            GrammarPeekMatch::Command(_) => SourceItem::Command(input.parse()?),
            GrammarPeekMatch::Group(_) => SourceItem::SourceGroup(input.parse()?),
            GrammarPeekMatch::GroupedVariable => SourceItem::GroupedVariable(input.parse()?),
            GrammarPeekMatch::FlattenedVariable => SourceItem::FlattenedVariable(input.parse()?),
            GrammarPeekMatch::AppendVariableDestructuring | GrammarPeekMatch::Destructurer(_) => {
                return input.parse_err("Destructurings are not supported here")
            }
            GrammarPeekMatch::Punct(_) => SourceItem::Punct(input.parse_any_punct()?),
            GrammarPeekMatch::Ident(_) => SourceItem::Ident(input.parse_any_ident()?),
            GrammarPeekMatch::Literal(_) => SourceItem::Literal(input.parse()?),
            GrammarPeekMatch::End => return input.parse_err("Expected some item"),
        })
    }
}

impl Interpret for SourceItem {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            SourceItem::Command(command_invocation) => {
                command_invocation.interpret_into(interpreter, output)?;
            }
            SourceItem::GroupedVariable(variable) => {
                variable.interpret_into(interpreter, output)?;
            }
            SourceItem::FlattenedVariable(variable) => {
                variable.interpret_into(interpreter, output)?;
            }
            SourceItem::SourceGroup(group) => {
                group.interpret_into(interpreter, output)?;
            }
            SourceItem::Punct(punct) => output.push_punct(punct),
            SourceItem::Ident(ident) => output.push_ident(ident),
            SourceItem::Literal(literal) => output.push_literal(literal),
        }
        Ok(())
    }
}

impl HasSpanRange for SourceItem {
    fn span_range(&self) -> SpanRange {
        match self {
            SourceItem::Command(command_invocation) => command_invocation.span_range(),
            SourceItem::FlattenedVariable(variable) => variable.span_range(),
            SourceItem::GroupedVariable(variable) => variable.span_range(),
            SourceItem::SourceGroup(group) => group.span_range(),
            SourceItem::Punct(punct) => punct.span_range(),
            SourceItem::Ident(ident) => ident.span_range(),
            SourceItem::Literal(literal) => literal.span_range(),
        }
    }
}
