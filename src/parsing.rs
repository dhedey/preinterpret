use crate::internal_prelude::*;

pub(crate) enum NextItem {
    CommandInvocation(CommandInvocation),
    VariableSubstitution(VariableSubstitution),
    Group(Group),
    Leaf(TokenTree),
    EndOfStream,
}

pub(crate) fn parse_next_item(
    tokens: &mut PeekableTokenIter,
) -> Result<NextItem> {
    Ok(match tokens.next() {
        None => NextItem::EndOfStream,
        Some(TokenTree::Group(group)) => {
            if let Some(command_invocation) = consume_if_command_invocation(&group)? {
                NextItem::CommandInvocation(command_invocation)
            } else {
                NextItem::Group(group)
            }
        }
        Some(TokenTree::Punct(punct)) => {
            if let Some(variable_substitution) = consume_if_variable_substitution(&punct, tokens) {
                NextItem::VariableSubstitution(variable_substitution)
            } else {
                NextItem::Leaf(TokenTree::Punct(punct))
            }
        }
        Some(leaf) => NextItem::Leaf(leaf),
    })
}

pub(crate) fn consume_set_statement(source_tokens: &mut PeekableTokenIter) -> Option<Ident> {
    consume_if_punct_matching(source_tokens, '#')?;
    let ident = consume_if_ident(source_tokens)?;
    consume_if_punct_matching(source_tokens, '=')?;
    Some(ident)
}

fn consume_if_command_invocation(group: &Group) -> Result<Option<CommandInvocation>> {
    // Attempt to match `[!ident`, if that doesn't match, we assume it's not a command invocation.
    let Some((command_ident, mut remaining_tokens)) = consume_if_command_start(group) else {
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

fn consume_if_command_start(group: &Group) -> Option<(Ident, PeekableTokenIter)> {
    if group.delimiter() != Delimiter::Bracket {
        return None;
    }
    let mut group_tokens = group.stream().into_iter().peekable();
    consume_if_punct_matching(&mut group_tokens, '!')?;
    let ident = consume_if_ident(&mut group_tokens)?;
    Some((ident, group_tokens))
}

fn consume_command_end(command_ident: &Ident, tokens: &mut PeekableTokenIter) -> Option<CommandKind> {
    let command_kind = parse_supported_command_kind(command_ident)?;
    consume_if_punct_matching(tokens, '!')?;
    Some(command_kind)
}

fn consume_if_variable_substitution(punct: &Punct, tokens: &mut PeekableTokenIter) -> Option<VariableSubstitution> {
    if punct.as_char() != '#' {
        return None;
    }
    let variable_name = consume_if_ident(tokens)?;
    Some(VariableSubstitution::new(punct.clone(), variable_name))
}

pub(crate) fn consume_if_punct_matching(tokens: &mut PeekableTokenIter, char: char) -> Option<Punct> {
    let Some(TokenTree::Punct(punct)) = tokens.peek() else {
        return None;
    };
    if punct.as_char() != char {
        return None;
    }
    let Some(TokenTree::Punct(punct)) = tokens.next() else {
        unreachable!("We just peeked a token of this type");
    };
    Some(punct)
}

pub(crate) fn consume_if_ident(tokens: &mut PeekableTokenIter) -> Option<Ident> {
    let Some(TokenTree::Ident(_)) = tokens.peek() else {
        return None;
    };
    let Some(TokenTree::Ident(ident)) = tokens.next() else {
        unreachable!("We just peeked a token of this type");
    };
    Some(ident)
}