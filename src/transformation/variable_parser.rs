use crate::internal_prelude::*;

/// We have the following write modes:
/// * `#x` - Reads a token tree, writes a stream (opposite of #x)
/// * `#..x` - Reads a stream, writes a stream (opposite of #..x)
///
/// And the following append modes:
/// * `#>>x` - Reads a token tree, appends a token tree (opposite of for #y in #x)
/// * `#>>..x` - Reads a token tree, appends a stream (i.e. flatten it if it's a group)
/// * `#..>>x` - Reads a stream, appends a group (opposite of for #y in #x)
/// * `#..>>..x` - Reads a stream, appends a stream
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
    /// #>>x - Reads a token tree, appends the token tree (allows reading with !for! #y in #x)
    GroupedAppendGrouped {
        marker: Token![#],
        append: Token![>>],
        name: Ident,
    },
    /// #>>..x - Reads a token tree, appends it flattened if its a group
    GroupedAppendFlattened {
        marker: Token![#],
        append: Token![>>],
        flattened_write: Token![..],
        name: Ident,
    },
    /// #..>>x - Reads a stream, appends a group (allows reading with !for! #y in #x)
    FlattenedAppendGrouped {
        marker: Token![#],
        flattened_read: Token![..],
        append: Token![>>],
        name: Ident,
        until: ParseUntil,
    },
    /// #..>>..x - Reads a stream, appends a stream
    FlattenedAppendFlattened {
        marker: Token![#],
        flattened_read: Token![..],
        append: Token![>>],
        flattened_write: Token![..],
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
            if input.peek(Token![>>]) {
                let append = input.parse()?;
                if input.peek(Token![..]) {
                    let flattened_write = input.parse()?;
                    let name = input.parse()?;
                    let until = ParseUntil::peek_flatten_limit::<C>(input)?;
                    return Ok(Self::FlattenedAppendFlattened {
                        marker,
                        flattened_read: flatten,
                        append,
                        flattened_write,
                        name,
                        until,
                    });
                }
                let name = input.parse()?;
                let until = ParseUntil::peek_flatten_limit::<C>(input)?;
                return Ok(Self::FlattenedAppendGrouped {
                    marker,
                    flattened_read: flatten,
                    append,
                    name,
                    until,
                });
            }
            let name = input.parse()?;
            let until = ParseUntil::peek_flatten_limit::<C>(input)?;
            return Ok(Self::Flattened {
                marker,
                flatten,
                name,
                until,
            });
        }
        if input.peek(Token![>>]) {
            let append = input.parse()?;
            if input.peek(Token![..]) {
                let flattened_write = input.parse()?;
                let name = input.parse()?;
                return Ok(Self::GroupedAppendFlattened {
                    marker,
                    append,
                    flattened_write,
                    name,
                });
            }
            let name = input.parse()?;
            return Ok(Self::GroupedAppendGrouped {
                marker,
                append,
                name,
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
            VariableParserKind::GroupedAppendGrouped { name, .. } => name,
            VariableParserKind::GroupedAppendFlattened { name, .. } => name,
            VariableParserKind::FlattenedAppendGrouped { name, .. } => name,
            VariableParserKind::FlattenedAppendFlattened { name, .. } => name,
        };
        name_ident.to_string()
    }
}

impl HasSpanRange for VariableParserKind {
    fn span_range(&self) -> SpanRange {
        let (marker, name) = match self {
            VariableParserKind::Grouped { marker, name, .. } => (marker, name),
            VariableParserKind::Flattened { marker, name, .. } => (marker, name),
            VariableParserKind::GroupedAppendGrouped { marker, name, .. } => (marker, name),
            VariableParserKind::GroupedAppendFlattened { marker, name, .. } => (marker, name),
            VariableParserKind::FlattenedAppendGrouped { marker, name, .. } => (marker, name),
            VariableParserKind::FlattenedAppendFlattened { marker, name, .. } => (marker, name),
        };
        SpanRange::new_between(marker.span, name.span())
    }
}

impl VariableParserKind {
    pub(crate) fn is_flattened_input(&self) -> bool {
        matches!(
            self,
            VariableParserKind::Flattened { .. }
                | VariableParserKind::FlattenedAppendGrouped { .. }
                | VariableParserKind::FlattenedAppendFlattened { .. }
        )
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
            VariableParserKind::GroupedAppendGrouped { .. } => {
                let reference = self.binding(interpreter)?;
                input
                    .parse::<ParsedTokenTree>()?
                    .push_as_token_tree(reference.into_mut()?.into_stream()?.as_mut());
            }
            VariableParserKind::GroupedAppendFlattened { .. } => {
                let reference = self.binding(interpreter)?;
                input
                    .parse::<ParsedTokenTree>()?
                    .flatten_into(reference.into_mut()?.into_stream()?.as_mut());
            }
            VariableParserKind::FlattenedAppendGrouped { marker, until, .. } => {
                let reference = self.binding(interpreter)?;
                reference.into_mut()?.into_stream()?.as_mut().push_grouped(
                    |inner| until.handle_parse_into(input, inner),
                    Delimiter::None,
                    marker.span,
                )?;
            }
            VariableParserKind::FlattenedAppendFlattened { until, .. } => {
                let reference = self.binding(interpreter)?;
                until.handle_parse_into(input, reference.into_mut()?.into_stream()?.as_mut())?;
            }
        }
        Ok(())
    }
}

enum ParsedTokenTree {
    NoneGroup(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl ParsedTokenTree {
    fn into_interpreted(self) -> OutputStream {
        match self {
            ParsedTokenTree::NoneGroup(group) => OutputStream::raw(group.stream()),
            ParsedTokenTree::Ident(ident) => OutputStream::raw(ident.to_token_stream()),
            ParsedTokenTree::Punct(punct) => OutputStream::raw(punct.to_token_stream()),
            ParsedTokenTree::Literal(literal) => OutputStream::raw(literal.to_token_stream()),
        }
    }

    fn push_as_token_tree(self, output: &mut OutputStream) {
        match self {
            ParsedTokenTree::NoneGroup(group) => {
                output.push_raw_token_tree(TokenTree::Group(group))
            }
            ParsedTokenTree::Ident(ident) => output.push_ident(ident),
            ParsedTokenTree::Punct(punct) => output.push_punct(punct),
            ParsedTokenTree::Literal(literal) => output.push_literal(literal),
        }
    }

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
