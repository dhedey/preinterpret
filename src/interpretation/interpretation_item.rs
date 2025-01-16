use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum InterpretationItem {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    Group(InterpretationGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl Parse for InterpretationItem {
    fn parse(input: ParseStream) -> Result<Self> {
        match detect_group(input.cursor()) {
            GroupMatch::Command => return Ok(InterpretationItem::Command(input.parse()?)),
            GroupMatch::OtherGroup => return Ok(InterpretationItem::Group(input.parse()?)),
            GroupMatch::None => {}
        }
        if input.peek(token::Pound) {
            let fork = input.fork();
            if let Ok(variable) = fork.parse() {
                input.advance_to(&fork);
                return Ok(InterpretationItem::GroupedVariable(variable));
            }
            let fork = input.fork();
            if let Ok(variable) = fork.parse() {
                input.advance_to(&fork);
                return Ok(InterpretationItem::FlattenedVariable(variable));
            }
        }
        Ok(match input.parse::<TokenTree>()? {
            TokenTree::Group(_) => {
                unreachable!("Should have been already handled by the first branch above")
            }
            TokenTree::Punct(punct) => InterpretationItem::Punct(punct),
            TokenTree::Ident(ident) => InterpretationItem::Ident(ident),
            TokenTree::Literal(literal) => InterpretationItem::Literal(literal),
        })
    }
}

enum GroupMatch {
    Command,
    OtherGroup,
    None,
}

fn detect_group(cursor: syn::buffer::Cursor) -> GroupMatch {
    let next = match cursor.any_group() {
        Some((next, Delimiter::Bracket, _, _)) => next,
        Some(_) => return GroupMatch::OtherGroup,
        None => return GroupMatch::None,
    };
    let next = match next.punct() {
        Some((punct, next)) if punct.as_char() == '!' => next,
        _ => return GroupMatch::OtherGroup,
    };
    let next = match next.ident() {
        Some((_, next)) => next,
        _ => return GroupMatch::OtherGroup,
    };
    match next.punct() {
        Some((punct, _)) if punct.as_char() == '!' => GroupMatch::Command,
        _ => GroupMatch::OtherGroup,
    }
}

impl Interpret for InterpretationItem {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match self {
            InterpretationItem::Command(command_invocation) => {
                command_invocation.interpret_as_tokens_into(interpreter, output)?;
            }
            InterpretationItem::GroupedVariable(variable) => {
                variable.interpret_as_tokens_into(interpreter, output)?;
            }
            InterpretationItem::FlattenedVariable(variable) => {
                variable.interpret_as_tokens_into(interpreter, output)?;
            }
            InterpretationItem::Group(group) => {
                group.interpret_as_tokens_into(interpreter, output)?;
            }
            InterpretationItem::Punct(punct) => output.push_punct(punct),
            InterpretationItem::Ident(ident) => output.push_ident(ident),
            InterpretationItem::Literal(literal) => output.push_literal(literal),
        }
        Ok(())
    }
}

impl Express for InterpretationItem {
    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        match self {
            InterpretationItem::Command(command_invocation) => {
                command_invocation.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            InterpretationItem::FlattenedVariable(variable) => {
                variable.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            InterpretationItem::GroupedVariable(variable) => {
                variable.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            InterpretationItem::Group(group) => {
                group.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            InterpretationItem::Punct(punct) => expression_stream.push_punct(punct),
            InterpretationItem::Ident(ident) => expression_stream.push_ident(ident),
            InterpretationItem::Literal(literal) => expression_stream.push_literal(literal),
        }
        Ok(())
    }
}

impl HasSpanRange for InterpretationItem {
    fn span_range(&self) -> SpanRange {
        match self {
            InterpretationItem::Command(command_invocation) => command_invocation.span_range(),
            InterpretationItem::FlattenedVariable(variable) => {
                variable.span_range()
            }
            InterpretationItem::GroupedVariable(variable) => {
                variable.span_range()
            }
            InterpretationItem::Group(group) => group.span_range(),
            InterpretationItem::Punct(punct) => punct.span_range(),
            InterpretationItem::Ident(ident) => ident.span_range(),
            InterpretationItem::Literal(literal) => literal.span_range(),
        }
    }
}
