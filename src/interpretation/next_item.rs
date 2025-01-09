use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum NextItem {
    Command(Command),
    Variable(Variable),
    Group(InterpretationGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl NextItem {
    pub(super) fn parse(parse_stream: &mut InterpreterParseStream) -> Result<Option<Self>> {
        let next = match parse_stream.next_token_tree_or_end() {
            Some(next) => next,
            None => return Ok(None),
        };
        Ok(Some(match next {
            TokenTree::Group(group) => {
                if let Some(command) = Command::attempt_parse_from_group(&group)? {
                    NextItem::Command(command)
                } else {
                    NextItem::Group(InterpretationGroup::parse(group)?)
                }
            }
            TokenTree::Punct(punct) => {
                if let Some(variable) =
                    Variable::parse_consuming_only_if_match(&punct, parse_stream)
                {
                    NextItem::Variable(variable)
                } else {
                    NextItem::Punct(punct)
                }
            }
            TokenTree::Ident(ident) => NextItem::Ident(ident),
            TokenTree::Literal(literal) => NextItem::Literal(literal),
        }))
    }
}

impl Interpret for NextItem {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match self {
            NextItem::Command(command_invocation) => {
                command_invocation.interpret_as_tokens_into(interpreter, output)?;
            }
            NextItem::Variable(variable) => {
                variable.interpret_as_tokens_into(interpreter, output)?;
            }
            NextItem::Group(group) => {
                group.interpret_as_tokens_into(interpreter, output)?;
            }
            NextItem::Punct(punct) => output.push_punct(punct),
            NextItem::Ident(ident) => output.push_ident(ident),
            NextItem::Literal(literal) => output.push_literal(literal),
        }
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        match self {
            NextItem::Command(command_invocation) => {
                command_invocation.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            NextItem::Variable(variable) => {
                variable.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            NextItem::Group(group) => {
                group.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            NextItem::Punct(punct) => expression_stream.push_punct(punct),
            NextItem::Ident(ident) => expression_stream.push_ident(ident),
            NextItem::Literal(literal) => expression_stream.push_literal(literal),
        }
        Ok(())
    }
}

impl HasSpanRange for NextItem {
    fn span_range(&self) -> SpanRange {
        match self {
            NextItem::Command(command_invocation) => command_invocation.span_range(),
            NextItem::Variable(variable_substitution) => variable_substitution.span_range(),
            NextItem::Group(group) => group.span_range(),
            NextItem::Punct(punct) => punct.span_range(),
            NextItem::Ident(ident) => ident.span_range(),
            NextItem::Literal(literal) => literal.span_range(),
        }
    }
}
