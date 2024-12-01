use crate::internal_prelude::*;

pub(crate) enum NextItem {
    CommandInvocation(CommandInvocation),
    VariableSubstitution(VariableSubstitution),
    Group(Group),
    Leaf(TokenTree),
    EndOfStream,
}

pub(crate) fn parse_next_item(
    tokens: &mut Tokens,
) -> Result<NextItem> {
    Ok(match tokens.next() {
        Some(TokenTree::Group(group)) => {
            if let Some(command_invocation) = parse_command_invocation(&group)? {
                NextItem::CommandInvocation(command_invocation)
            } else {
                NextItem::Group(group)
            }
        }
        Some(TokenTree::Punct(punct)) => {
            if let Some(variable_substitution) = parse_only_if_variable_substitution(&punct, tokens) {
                NextItem::VariableSubstitution(variable_substitution)
            } else {
                NextItem::Leaf(TokenTree::Punct(punct))
            }
        }
        Some(leaf) => NextItem::Leaf(leaf),
        None => NextItem::EndOfStream,
    })
}

pub(crate) fn parse_variable_set(tokens: &mut Tokens) -> Option<Ident> {
    let variable_name = parse_variable(tokens)?;
    tokens.next_as_punct_matching('=')?;
    Some(variable_name)
}

pub(crate) fn parse_variable(tokens: &mut Tokens) -> Option<Ident> {
    tokens.next_as_punct_matching('#')?;
    tokens.next_as_ident()
}

fn parse_command_invocation(group: &Group) -> Result<Option<CommandInvocation>> {
    fn consume_command_start(group: &Group) -> Option<(Ident, Tokens)> {
        if group.delimiter() != Delimiter::Bracket {
            return None;
        }
        let mut tokens = Tokens::new(group.stream());
        tokens.next_as_punct_matching('!')?;
        let ident = tokens.next_as_ident()?;
        Some((ident, tokens))
    }

    fn consume_command_end(command_ident: &Ident, tokens: &mut Tokens) -> Option<CommandKind> {
        let command_kind = CommandKind::attempt_parse(command_ident)?;
        tokens.next_as_punct_matching('!')?;
        Some(command_kind)
    }

    // Attempt to match `[!ident`, if that doesn't match, we assume it's not a command invocation,
    // so return `Ok(None)`
    let Some((command_ident, mut remaining_tokens)) = consume_command_start(group) else {
        return Ok(None);
    };

    // We have now checked enough that we're confident the user is pretty intentionally using
    // the call convention. Any issues we hit from this point will be a helpful compiler error.
    match consume_command_end(&command_ident, &mut remaining_tokens) {
        Some(command_kind) => Ok(Some(CommandInvocation::new(command_kind, group, remaining_tokens))),
        None => Err(Error::new(
            group.span(),
            format!("Expected `[!<command>! ..]`, for <command> one of: {}.\nIf this wasn't intended to be a preinterpret command, you can work around this with [!raw! [!{command_ident} ... ]]", CommandKind::list_all()),
        )),
    }
}

// We ensure we don't consume any tokens unless we have a variable substitution
fn parse_only_if_variable_substitution(punct: &Punct, tokens: &mut Tokens) -> Option<VariableSubstitution> {
    if punct.as_char() != '#' {
        return None;
    }
    let Some(TokenTree::Ident(_)) = tokens.peek() else {
        return None;
    };
    let Some(TokenTree::Ident(variable_name)) = tokens.next() else {
        unreachable!("We just peeked a token of this type");
    };
    Some(VariableSubstitution::new(punct.clone(), variable_name))
}
