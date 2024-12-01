use crate::internal_prelude::*;

pub(crate) fn interpret(token_stream: TokenStream) -> Result<TokenStream> {
    Interpreter::new().interpret_tokens(token_stream.into_iter().peekable())
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

    pub(crate) fn interpret_tokens(&mut self, mut source_tokens: PeekableTokenIter) -> Result<TokenStream> {
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
                    // If it's a group, run interpret on its contents recursively.
                    expanded.extend(iter::once(TokenTree::Group(Group::new(
                        group.delimiter(),
                        self.interpret_tokens(group.stream().into_iter().peekable())?,
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

pub(crate) struct VariableSubstitution {
    marker: Punct, // #
    variable_name: Ident,
}

impl VariableSubstitution {
    pub(crate) fn new(marker: Punct, variable_name: Ident) -> Self {
        Self { marker, variable_name }
    }

    pub(crate) fn execute(self, interpreter: &mut Interpreter) -> Result<TokenStream> {
        let VariableSubstitution { marker, variable_name } = self;
        let Some(substituted) = interpreter.get_variable(&variable_name.to_string()) else {
            let marker = marker.as_char();
            let name_str = variable_name.to_string();
            let name_str = &name_str;
            return Err(Error::new(
                variable_name.span(),
                format!("The variable {marker}{name_str} wasn't set.\nIf this wasn't intended to be a variable, work around this with [!raw! {marker}{name_str}]"),
            ));
        };
        Ok(substituted.clone())
    }
} 
