use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum NextItem {
    CommandInvocation(CommandInvocation),
    Variable(Variable),
    Group(Group),
    Leaf(TokenTree),
}

impl Interpret for NextItem {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match self {
            NextItem::Leaf(token_tree) => {
                output.push_raw_token_tree(token_tree);
            }
            NextItem::Group(group) => {
                group.interpret_as_tokens_into(interpreter, output)?;
            }
            NextItem::Variable(variable) => {
                variable.interpret_as_tokens_into(interpreter, output)?;
            }
            NextItem::CommandInvocation(command_invocation) => {
                command_invocation.interpret_as_tokens_into(interpreter, output)?;
            }
        }
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        match self {
            NextItem::Leaf(token_tree) => {
                expression_stream.push_raw_token_tree(token_tree);
            }
            NextItem::Group(group) => {
                group.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            NextItem::Variable(variable) => {
                variable.interpret_as_expression_into(interpreter, expression_stream)?;
            }
            NextItem::CommandInvocation(command_invocation) => {
                command_invocation.interpret_as_expression_into(interpreter, expression_stream)?;
            }
        }
        Ok(())
    }
}

impl HasSpanRange for NextItem {
    fn span_range(&self) -> SpanRange {
        match self {
            NextItem::CommandInvocation(command_invocation) => command_invocation.span_range(),
            NextItem::Variable(variable_substitution) => variable_substitution.span_range(),
            NextItem::Group(group) => group.span_range(),
            NextItem::Leaf(token_tree) => token_tree.span_range(),
        }
    }
}
