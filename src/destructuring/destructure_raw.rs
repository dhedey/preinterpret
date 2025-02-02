use crate::internal_prelude::*;

/// This is an *exact* destructuring, which must match item-by-item.
#[derive(Clone)]
pub(crate) enum RawDestructureItem {
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
    Group(RawDestructureGroup),
}

impl RawDestructureItem {
    pub(crate) fn handle_destructure(&self, input: InterpretedParseStream) -> ExecutionResult<()> {
        match self {
            RawDestructureItem::Punct(punct) => {
                input.parse_punct_matching(punct.as_char())?;
            }
            RawDestructureItem::Ident(ident) => {
                input.parse_ident_matching(&ident.to_string())?;
            }
            RawDestructureItem::Literal(literal) => {
                input.parse_literal_matching(&literal.to_string())?;
            }
            RawDestructureItem::Group(group) => {
                group.handle_destructure(input)?;
            }
        }
        Ok(())
    }

    pub(crate) fn new_from_token_tree(token_tree: TokenTree) -> Self {
        match token_tree {
            TokenTree::Punct(punct) => Self::Punct(punct),
            TokenTree::Ident(ident) => Self::Ident(ident),
            TokenTree::Literal(literal) => Self::Literal(literal),
            TokenTree::Group(group) => Self::Group(RawDestructureGroup::new_from_group(&group)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RawDestructureStream {
    inner: Vec<RawDestructureItem>,
}

impl RawDestructureStream {
    pub(crate) fn empty() -> Self {
        Self { inner: vec![] }
    }

    pub(crate) fn new_from_token_stream(tokens: impl IntoIterator<Item = TokenTree>) -> Self {
        let mut new = Self::empty();
        new.append_from_token_stream(tokens);
        new
    }

    pub(crate) fn append_from_token_stream(&mut self, tokens: impl IntoIterator<Item = TokenTree>) {
        for token_tree in tokens {
            self.inner
                .push(RawDestructureItem::new_from_token_tree(token_tree));
        }
    }

    pub(crate) fn push_item(&mut self, item: RawDestructureItem) {
        self.inner.push(item);
    }

    pub(crate) fn handle_destructure(&self, input: InterpretedParseStream) -> ExecutionResult<()> {
        for item in self.inner.iter() {
            item.handle_destructure(input)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct RawDestructureGroup {
    delimiter: Delimiter,
    inner: RawDestructureStream,
}

impl RawDestructureGroup {
    pub(crate) fn new(delimiter: Delimiter, inner: RawDestructureStream) -> Self {
        Self { delimiter, inner }
    }

    pub(crate) fn new_from_group(group: &Group) -> Self {
        Self {
            delimiter: group.delimiter(),
            inner: RawDestructureStream::new_from_token_stream(group.stream()),
        }
    }

    pub(crate) fn handle_destructure(&self, input: InterpretedParseStream) -> ExecutionResult<()> {
        let (_, inner) = input.parse_specific_group(self.delimiter)?;
        self.inner.handle_destructure(&inner)
    }
}
