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

    pub(crate) fn interpret_tokens(&mut self, mut source_tokens: Tokens) -> Result<TokenStream> {
        let mut expanded = TokenStream::new();
        loop {
            match parse_next_item(&mut source_tokens)? {
                NextItem::CommandInvocation(command_invocation) => {
                    expanded.extend(command_invocation.execute(self)?);
                }
                NextItem::VariableSubstitution(variable_substitution) => {
                    expanded.extend(variable_substitution.execute(self)?);
                }
                NextItem::Group(group) => {
                    expanded.extend(iter::once(TokenTree::Group(Group::new(
                        group.delimiter(),
                        // If it's a group, run interpret on its contents recursively.
                        self.interpret_tokens(Tokens::new(group.stream()))?,
                    ))));
                }
                NextItem::Leaf(token_tree) => {
                    expanded.extend(iter::once(token_tree));
                }
                NextItem::EndOfStream => break,
            }
        }
        return Ok(expanded);    
    }
}
