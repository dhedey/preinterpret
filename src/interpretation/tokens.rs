use crate::internal_prelude::*;

/// An analogue to [`syn::parse::ParseStream`].
/// 
/// In future, perhaps we should use it.
#[derive(Clone)]
pub(crate) struct Tokens(iter::Peekable<<TokenStream as IntoIterator>::IntoIter>, SpanRange);

impl Tokens {
    pub(crate) fn new(tokens: TokenStream, span_range: SpanRange) -> Self {
        Self(tokens.into_iter().peekable(), span_range)
    }

    pub(crate) fn peek(&mut self) -> Option<&TokenTree> {
        self.0.peek()
    }

    pub(crate) fn next(&mut self) -> Option<TokenTree> {
        self.0.next()
    }

    pub(crate) fn next_as_ident(&mut self) -> Option<Ident> {
        match self.next() {
            Some(TokenTree::Ident(ident)) => Some(ident),
            _ => None,
        }
    }

    pub(crate) fn next_as_punct(&mut self) -> Option<Punct> {
        match self.next() {
            Some(TokenTree::Punct(punct)) => Some(punct),
            _ => None,
        }
    }

    pub(crate) fn next_as_punct_matching(&mut self, char: char) -> Option<Punct> {
        match self.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == char => Some(punct),
            _ => None,
        }
    }

    pub(crate) fn next_as_kinded_group(&mut self, delimiter: Delimiter) -> Option<Group> {
        match self.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == delimiter => Some(group),
            _ => None,
        }
    }

    pub(crate) fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }

    pub(crate) fn check_end(&mut self) -> Option<()> {
        if self.is_empty() {
            Some(())
        } else {
            None
        }
    }

    pub(crate) fn assert_end(&mut self, error_message: &'static str) -> Result<()> {
        match self.next() {
            Some(token) => token.span_range().err(error_message),
            None => Ok(()),
        }
    }

    pub(crate) fn next_item(&mut self) -> Result<Option<NextItem>> {
        let next = match self.next() {
            Some(next) => next,
            None => return Ok(None),
        };
        Ok(Some(match next {
            TokenTree::Group(group) => {
                if let Some(command_invocation) = parse_command_invocation(&group)? {
                    NextItem::CommandInvocation(command_invocation)
                } else {
                    NextItem::Group(group)
                }
            }
            TokenTree::Punct(punct) => {
                if let Some(variable_substitution) =
                    parse_only_if_variable_substitution(&punct, self)
                {
                    NextItem::Variable(variable_substitution)
                } else {
                    NextItem::Leaf(TokenTree::Punct(punct))
                }
            }
            leaf => NextItem::Leaf(leaf),
        }))
    }

    pub(crate) fn next_item_as_variable(
        &mut self,
        error_message: &'static str,
    ) -> Result<Variable> {
        match self.next_item()? {
            Some(NextItem::Variable(variable_substitution)) => Ok(variable_substitution),
            Some(item) => item.span_range().err(error_message),
            None => Span::call_site().span_range().err(error_message),
        }
    }

    pub(crate) fn into_token_stream(&mut self) -> TokenStream {
        core::mem::replace(&mut self.0, TokenStream::new().into_iter().peekable()).collect()
    }
}

impl<'a> Interpret for &'a mut Tokens {
    fn interpret_as_tokens_into(self, interpreter: &mut Interpreter, output: &mut InterpretedStream) -> Result<()> {
        loop {
            match self.next_item()? {
                Some(next_item) => {
                    next_item.interpret_as_tokens_into(interpreter, output)?
                }
                None => return Ok(()),
            }
        }
    }

    fn interpret_as_expression_into(self, interpreter: &mut Interpreter, expression_stream: &mut ExpressionStream) -> Result<()> {
        let mut inner_expression_stream = ExpressionStream::new();
        loop {
            match self.next_item()? {
                Some(next_item) => {
                    next_item.interpret_as_expression_into(interpreter, &mut inner_expression_stream)?
                }
                None => break,
            }
        }
        expression_stream.push_expression_group(
            inner_expression_stream,
            Delimiter::None,
            self.1,
        );
        Ok(())
    }
}

fn parse_command_invocation(group: &Group) -> Result<Option<CommandInvocation>> {
    fn consume_command_start(group: &Group) -> Option<(Ident, Tokens)> {
        if group.delimiter() != Delimiter::Bracket {
            return None;
        }
        let mut tokens = Tokens::new(group.stream(), group.span_range());
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
    let (command_ident, mut remaining_tokens) = match consume_command_start(group) {
        Some(command_start) => command_start,
        None => return Ok(None),
    };

    // We have now checked enough that we're confident the user is pretty intentionally using
    // the call convention. Any issues we hit from this point will be a helpful compiler error.
    match consume_command_end(&command_ident, &mut remaining_tokens) {
        Some(command_kind) => Ok(Some(CommandInvocation::new(command_ident.clone(), command_kind, group, remaining_tokens))),
        None => Err(command_ident.span().error(
            format!(
                "Expected `[!<command>! ..]`, for <command> one of: {}.\nIf this wasn't intended to be a preinterpret command, you can work around this with [!raw! [!{} ... ]]",
                CommandKind::list_all(),
                command_ident,
            ),
        )),
    }
}

// We ensure we don't consume any tokens unless we have a variable substitution
fn parse_only_if_variable_substitution(punct: &Punct, tokens: &mut Tokens) -> Option<Variable> {
    if punct.as_char() != '#' {
        return None;
    }
    match tokens.peek() {
        Some(TokenTree::Ident(_)) => {}
        _ => return None,
    }
    match tokens.next() {
        Some(TokenTree::Ident(variable_name)) => Some(Variable::new(punct.clone(), variable_name)),
        _ => unreachable!("We just peeked a token of this type"),
    }
}
