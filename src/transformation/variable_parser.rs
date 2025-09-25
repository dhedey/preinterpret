use crate::internal_prelude::*;

/// We have the following write modes:
/// * `#x` - Reads a token tree, writes a stream (opposite of #x)
/// * `#..x` - Reads a stream, writes a stream (opposite of #..x)
#[derive(Clone)]
#[allow(unused)]
pub(crate) enum VariableParserKind {
    /// #x - Reads a token tree, writes a stream (opposite of #x)
    Grouped { marker: Token![#], name: Ident },
    /// #..x - Reads a stream, writes a stream (opposite of #..x)
    Flattened {
        marker: Token![#],
        flatten: Token![..],
        name: Ident,
        until: ParseUntil,
    },
}

impl VariableParserKind {
    #[allow(unused)]
    pub(crate) fn parse_only_unflattened_input(input: ParseStream<Source>) -> ParseResult<Self> {
        let variable: VariableParserKind = Self::parse_until::<UntilEnd>(input)?;
        if variable.is_flattened_input() {
            return variable
                .span_range()
                .parse_err("A flattened input variable is not supported here");
        }
        Ok(variable)
    }

    pub(crate) fn parse_until<C: StopCondition<Source>>(
        input: ParseStream<Source>,
    ) -> ParseResult<Self> {
        let marker = input.parse()?;
        if input.peek(Token![..]) {
            let flatten = input.parse()?;
            let name = input.parse()?;
            let until = ParseUntil::peek_flatten_limit::<C>(input)?;
            return Ok(Self::Flattened {
                marker,
                flatten,
                name,
                until,
            });
        }
        let name = input.parse()?;
        Ok(Self::Grouped { marker, name })
    }
}

impl IsVariable for VariableParserKind {
    fn get_name(&self) -> String {
        let name_ident = match self {
            VariableParserKind::Grouped { name, .. } => name,
            VariableParserKind::Flattened { name, .. } => name,
        };
        name_ident.to_string()
    }
}

impl HasSpanRange for VariableParserKind {
    fn span_range(&self) -> SpanRange {
        let (marker, name) = match self {
            VariableParserKind::Grouped { marker, name, .. } => (marker, name),
            VariableParserKind::Flattened { marker, name, .. } => (marker, name),
        };
        SpanRange::new_between(marker.span, name.span())
    }
}

impl VariableParserKind {
    pub(crate) fn is_flattened_input(&self) -> bool {
        matches!(self, VariableParserKind::Flattened { .. })
    }
}

impl HandleTransformation for VariableParserKind {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        _: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match self {
            VariableParserKind::Grouped { .. } => {
                let content = input.parse::<ParsedTokenTree>()?.into_interpreted();
                self.define_coerced(interpreter, content);
            }
            VariableParserKind::Flattened { until, .. } => {
                let mut content = OutputStream::new();
                until.handle_parse_into(input, &mut content)?;
                self.define_coerced(interpreter, content);
            }
        }
        Ok(())
    }
}

/// Unwraps a single none-group, or parses a single ident, punct or literal -- but not any other kind of group.
pub(super) enum ParsedTokenTree {
    NoneGroup(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl ParsedTokenTree {
    pub(super) fn into_interpreted(self) -> OutputStream {
        match self {
            ParsedTokenTree::NoneGroup(group) => OutputStream::raw(group.stream()),
            ParsedTokenTree::Ident(ident) => OutputStream::raw(ident.to_token_stream()),
            ParsedTokenTree::Punct(punct) => OutputStream::raw(punct.to_token_stream()),
            ParsedTokenTree::Literal(literal) => OutputStream::raw(literal.to_token_stream()),
        }
    }

    #[allow(unused)]
    pub(super) fn push_as_token_tree(self, output: &mut OutputStream) {
        match self {
            ParsedTokenTree::NoneGroup(group) => {
                output.push_raw_token_tree(TokenTree::Group(group))
            }
            ParsedTokenTree::Ident(ident) => output.push_ident(ident),
            ParsedTokenTree::Punct(punct) => output.push_punct(punct),
            ParsedTokenTree::Literal(literal) => output.push_literal(literal),
        }
    }

    #[allow(unused)]
    fn flatten_into(self, output: &mut OutputStream) {
        match self {
            ParsedTokenTree::NoneGroup(group) => output.extend_raw_tokens(group.stream()),
            ParsedTokenTree::Ident(ident) => output.push_ident(ident),
            ParsedTokenTree::Punct(punct) => output.push_punct(punct),
            ParsedTokenTree::Literal(literal) => output.push_literal(literal),
        }
    }
}

impl Parse<Output> for ParsedTokenTree {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        Ok(match input.parse::<TokenTree>()? {
            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                ParsedTokenTree::NoneGroup(group)
            }
            TokenTree::Group(group) => {
                return group
                    .delim_span()
                    .open()
                    .parse_err("Expected a group with transparent delimiters");
            }
            TokenTree::Ident(ident) => ParsedTokenTree::Ident(ident),
            TokenTree::Punct(punct) => ParsedTokenTree::Punct(punct),
            TokenTree::Literal(literal) => ParsedTokenTree::Literal(literal),
        })
    }
}
