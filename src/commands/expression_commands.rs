use crate::internal_prelude::*;

pub(crate) struct EvaluateCommand;

impl CommandDefinition for EvaluateCommand {
    const COMMAND_NAME: &'static str = "evaluate";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let token_stream =
            command.interpret_remaining_arguments(interpreter, SubstitutionMode::expression())?;
        Ok(evaluate_expression(token_stream, ExpressionParsingMode::Standard)?.into_token_stream())
    }
}

pub(crate) struct AssignCommand;

impl CommandDefinition for AssignCommand {
    const COMMAND_NAME: &'static str = "assign";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let AssignStatementStart {
            variable,
            operator,
        } = AssignStatementStart::parse(command.argument_tokens())
            .ok_or_else(|| command.error("Expected [!assign! #variable += ...] for + or some other operator supported in an expression"))?;

        let mut expression_tokens = TokenStream::new();
        expression_tokens.push_new_group(
            // TODO: Replace with `variable.read_into(tokens, substitution_mode)`
            // TODO: Replace most methods on interpeter with e.g.
            // command.interpret_into, next_item.interpet_into, etc.
            // And also create an Expression struct
            // TODO: Fix Expression to not need different parsing modes,
            // and to be parsed from the full token stream or until braces { .. }
            // or as a single item
            variable.read_required(interpreter)?.clone(),
            Delimiter::None,
            variable.span_range(),
        );
        expression_tokens.push_token_tree(operator.into());
        expression_tokens.push_new_group(
            command.interpret_remaining_arguments(interpreter, SubstitutionMode::expression())?,
            Delimiter::None,
            command.span_range(),
        );

        let output = evaluate_expression(expression_tokens, ExpressionParsingMode::Standard)?.into_token_stream();
        variable.set(interpreter, output);

        Ok(TokenStream::new())
    }
}

struct AssignStatementStart {
    variable: Variable,
    operator: Punct,
}

impl AssignStatementStart {
    fn parse(tokens: &mut Tokens) -> Option<Self> {
        let variable = tokens.next_item_as_variable("").ok()?;
        let operator = tokens.next_as_punct()?;
        match operator.as_char() {
            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' => {}
            _ => return None,
        }
        tokens.next_as_punct_matching('=')?;
        Some(AssignStatementStart {
            variable,
            operator,
        })
    }
}
