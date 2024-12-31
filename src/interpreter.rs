use crate::internal_prelude::*;

pub(crate) fn interpret(token_stream: TokenStream) -> Result<TokenStream> {
    Interpreter::new().interpret_tokens(Tokens::new(token_stream))
}

pub(crate) struct Interpreter {
    variables: HashMap<String, TokenStream>,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            variables: Default::default(),
        }
    }

    pub(crate) fn set_variable(&mut self, name: String, tokens: TokenStream) {
        self.variables.insert(name, tokens);
    }

    pub(crate) fn get_variable(&self, name: &str) -> Option<&TokenStream> {
        self.variables.get(name)
    }

    pub(crate) fn interpret_token_stream(
        &mut self,
        token_stream: TokenStream,
    ) -> Result<TokenStream> {
        self.interpret_tokens(Tokens::new(token_stream))
    }

    pub(crate) fn interpret_item(&mut self, item: NextItem) -> Result<TokenStream> {
        let mut expanded = TokenStream::new();
        self.interpret_next_item(item, &mut expanded)?;
        Ok(expanded)
    }

    pub(crate) fn interpret_tokens(&mut self, mut source_tokens: Tokens) -> Result<TokenStream> {
        let mut expanded = TokenStream::new();
        loop {
            match source_tokens.next_item()? {
                Some(next_item) => self.interpret_next_item(next_item, &mut expanded)?,
                None => return Ok(expanded),
            }
        }
    }

    fn interpret_next_item(&mut self, next_item: NextItem, output: &mut TokenStream) -> Result<()> {
        // We wrap command/variable substitutions in a transparent group so that they
        // can be treated as a single item in other commands.
        // e.g. if #x = 1 + 1, then [!math! #x * #x] should be 4.
        // Note that such groups are ignored in the macro output, due to this
        // issue in rustc: https://github.com/rust-lang/rust/issues/67062
        match next_item {
            NextItem::Leaf(token_tree) => {
                output.push_token_tree(token_tree);
            }
            NextItem::Group(group) => {
                // If it's a group, run interpret on its contents recursively.
                output.push_new_group(
                    group.span_range(),
                    group.delimiter(),
                    self.interpret_tokens(Tokens::new(group.stream()))?,
                );
            }
            NextItem::Variable(variable_substitution) => {
                // We wrap substituted variables in a transparent group so that
                // they can be used collectively in future expressions, so that
                // e.g. if #x = 1 + 1 then #x * #x = 4 rather than 3
                output.push_new_group(
                    variable_substitution.span_range(),
                    Delimiter::None,
                    variable_substitution.execute_substitution(self)?,
                );
            }
            NextItem::CommandInvocation(command_invocation) => {
                output.push_new_group(
                    command_invocation.span_range(),
                    Delimiter::None,
                    command_invocation.execute(self)?,
                );
            }
        }
        Ok(())
    }
}

pub(crate) struct Tokens(iter::Peekable<<TokenStream as IntoIterator>::IntoIter>);

impl Tokens {
    pub(crate) fn new(tokens: TokenStream) -> Self {
        Self(tokens.into_iter().peekable())
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

    pub(crate) fn check_end(&mut self) -> Option<()> {
        if self.peek().is_none() {
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

    pub(crate) fn into_token_stream(self) -> TokenStream {
        self.0.collect()
    }
}

pub(crate) enum NextItem {
    CommandInvocation(CommandInvocation),
    Variable(Variable),
    Group(Group),
    Leaf(TokenTree),
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
    let (command_ident, mut remaining_tokens) = match consume_command_start(group) {
        Some(command_start) => command_start,
        None => return Ok(None),
    };

    // We have now checked enough that we're confident the user is pretty intentionally using
    // the call convention. Any issues we hit from this point will be a helpful compiler error.
    match consume_command_end(&command_ident, &mut remaining_tokens) {
        Some(command_kind) => Ok(Some(CommandInvocation::new(command_kind, group, remaining_tokens))),
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
