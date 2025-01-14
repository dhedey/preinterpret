use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum InterpretationItem {
    Command(Command),
    Variable(Variable),
    Group(InterpretationGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl InterpretationItem {
    pub(super) fn parse(parse_stream: &mut InterpreterParseStream) -> Result<Option<Self>> {
        let next = match parse_stream.next_token_tree_or_end() {
            Some(next) => next,
            None => return Ok(None),
        };
        Ok(Some(match next {
            TokenTree::Group(group) => {
                if let Some(command) = Command::attempt_parse_from_group(&group)? {
                    InterpretationItem::Command(command)
                } else {
                    InterpretationItem::Group(InterpretationGroup::parse(group)?)
                }
            }
            TokenTree::Punct(punct) => {
                if let Some(variable) =
                    Variable::parse_consuming_only_if_match(&punct, parse_stream)
                {
                    InterpretationItem::Variable(variable)
                } else {
                    InterpretationItem::Punct(punct)
                }
            }
            TokenTree::Ident(ident) => InterpretationItem::Ident(ident),
            TokenTree::Literal(literal) => InterpretationItem::Literal(literal),
        }))
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
            InterpretationItem::Variable(variable) => {
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

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        match self {
            InterpretationItem::Command(command_invocation) => {
                command_invocation.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            InterpretationItem::Variable(variable) => {
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
            InterpretationItem::Variable(variable_substitution) => {
                variable_substitution.span_range()
            }
            InterpretationItem::Group(group) => group.span_range(),
            InterpretationItem::Punct(punct) => punct.span_range(),
            InterpretationItem::Ident(ident) => ident.span_range(),
            InterpretationItem::Literal(literal) => literal.span_range(),
        }
    }
}
