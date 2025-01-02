use crate::internal_prelude::*;

pub(crate) struct IfCommand;

impl CommandDefinition for IfCommand {
    const COMMAND_NAME: &'static str = "if";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let parsed = match parse_if_statement(command.argument_tokens()) {
            Some(parsed) => parsed,
            None => {
                return command.err("Expected [!if! (condition) { true_code }] or [!if! (condition) { true_code } !else! { false_code}]");
            }
        };

        let interpreted_condition =
            interpreter.interpret_item(parsed.condition, SubstitutionMode::expression())?;
        let evaluated_condition = evaluate_expression(
            interpreted_condition,
            ExpressionParsingMode::BeforeCurlyBraces,
        )?
        .expect_bool("An if condition must evaluate to a boolean")?
        .value();

        if evaluated_condition {
            interpreter.interpret_token_stream(parsed.true_code, SubstitutionMode::token_stream())
        } else if let Some(false_code) = parsed.false_code {
            interpreter.interpret_token_stream(false_code, SubstitutionMode::token_stream())
        } else {
            Ok(TokenStream::new())
        }
    }
}

struct IfStatement {
    condition: NextItem,
    true_code: TokenStream,
    false_code: Option<TokenStream>,
}

fn parse_if_statement(tokens: &mut Tokens) -> Option<IfStatement> {
    let condition = tokens.next_item().ok()??;
    let true_code = tokens.next_as_kinded_group(Delimiter::Brace)?.stream();
    let false_code = if tokens.peek().is_some() {
        tokens.next_as_punct_matching('!')?;
        let else_word = tokens.next_as_ident()?;
        tokens.next_as_punct_matching('!')?;
        if else_word != "else" {
            return None;
        }
        Some(tokens.next_as_kinded_group(Delimiter::Brace)?.stream())
    } else {
        None
    };
    tokens.check_end()?;
    Some(IfStatement {
        condition,
        true_code,
        false_code,
    })
}

pub(crate) struct WhileCommand;

impl CommandDefinition for WhileCommand {
    const COMMAND_NAME: &'static str = "while";

    fn execute(interpreter: &mut Interpreter, mut command: Command) -> Result<TokenStream> {
        let parsed = match parse_while_statement(command.argument_tokens()) {
            Some(parsed) => parsed,
            None => {
                return command.err("Expected [!while! (condition) { code }]");
            }
        };

        let mut output = TokenStream::new();
        let mut iteration_count = 0;
        loop {
            let interpreted_condition =
                interpreter.interpret_item(parsed.condition.clone(), SubstitutionMode::expression())?;
            let evaluated_condition = evaluate_expression(
                interpreted_condition,
                ExpressionParsingMode::BeforeCurlyBraces,
            )?
            .expect_bool("An if condition must evaluate to a boolean")?
            .value();

            iteration_count += 1;
            if !evaluated_condition {
                break;
            }
            interpreter.config().check_iteration_count(&command, iteration_count)?;
            interpreter.interpret_token_stream_into(
                parsed.code.clone(),
                SubstitutionMode::token_stream(),
                &mut output,
            )?;
        }

        Ok(output)
    }
}

struct WhileStatement {
    condition: NextItem,
    code: TokenStream,
}

fn parse_while_statement(tokens: &mut Tokens) -> Option<WhileStatement> {
    let condition = tokens.next_item().ok()??;
    let code = tokens.next_as_kinded_group(Delimiter::Brace)?.stream();
    tokens.check_end()?;
    Some(WhileStatement {
        condition,
        code,
    })
}