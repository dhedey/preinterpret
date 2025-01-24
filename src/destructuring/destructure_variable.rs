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
pub(crate) enum DestructureVariable {
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

impl DestructureVariable {
    pub(crate) fn parse_only_unflattened_input(input: ParseStream) -> ParseResult<Self> {
        let variable: DestructureVariable = Self::parse_until::<UntilEnd>(input)?;
        if variable.is_flattened_input() {
            return variable
                .span_range()
                .parse_err("A flattened input variable is not supported here");
        }
        Ok(variable)
    }

    pub(crate) fn parse_until<C: StopCondition>(input: ParseStream) -> ParseResult<Self> {
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

impl IsVariable for DestructureVariable {
    fn get_name(&self) -> String {
        let name_ident = match self {
            DestructureVariable::Grouped { name, .. } => name,
            DestructureVariable::Flattened { name, .. } => name,
            DestructureVariable::GroupedAppendGrouped { name, .. } => name,
            DestructureVariable::GroupedAppendFlattened { name, .. } => name,
            DestructureVariable::FlattenedAppendGrouped { name, .. } => name,
            DestructureVariable::FlattenedAppendFlattened { name, .. } => name,
        };
        name_ident.to_string()
    }
}

impl HasSpanRange for DestructureVariable {
    fn span_range(&self) -> SpanRange {
        let (marker, name) = match self {
            DestructureVariable::Grouped { marker, name, .. } => (marker, name),
            DestructureVariable::Flattened { marker, name, .. } => (marker, name),
            DestructureVariable::GroupedAppendGrouped { marker, name, .. } => (marker, name),
            DestructureVariable::GroupedAppendFlattened { marker, name, .. } => (marker, name),
            DestructureVariable::FlattenedAppendGrouped { marker, name, .. } => (marker, name),
            DestructureVariable::FlattenedAppendFlattened { marker, name, .. } => (marker, name),
        };
        SpanRange::new_between(marker.span, name.span())
    }
}

impl DestructureVariable {
    pub(crate) fn is_flattened_input(&self) -> bool {
        matches!(
            self,
            DestructureVariable::Flattened { .. }
                | DestructureVariable::FlattenedAppendGrouped { .. }
                | DestructureVariable::FlattenedAppendFlattened { .. }
        )
    }

    fn get_variable_data(&self, interpreter: &mut Interpreter) -> ExecutionResult<VariableData> {
        let variable_data = interpreter
            .get_existing_variable_data(self, || {
                self.error(format!(
                    "The variable #{} wasn't already set",
                    self.get_name()
                ))
            })?
            .cheap_clone();
        Ok(variable_data)
    }
}

impl HandleDestructure for DestructureVariable {
    fn handle_destructure(
        &self,
        input: ParseStream,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        match self {
            DestructureVariable::Grouped { .. } => {
                let content = input.parse_v2::<ParsedTokenTree>()?.into_interpreted();
                interpreter.set_variable(self, content)?;
            }
            DestructureVariable::Flattened { until, .. } => {
                let mut content = InterpretedStream::new();
                until.handle_parse_into(input, &mut content)?;
                interpreter.set_variable(self, content)?;
            }
            DestructureVariable::GroupedAppendGrouped { .. } => {
                let variable_data = self.get_variable_data(interpreter)?;
                input
                    .parse_v2::<ParsedTokenTree>()?
                    .push_as_token_tree(variable_data.get_mut(self)?.deref_mut());
            }
            DestructureVariable::GroupedAppendFlattened { .. } => {
                let variable_data = self.get_variable_data(interpreter)?;
                input
                    .parse_v2::<ParsedTokenTree>()?
                    .flatten_into(variable_data.get_mut(self)?.deref_mut());
            }
            DestructureVariable::FlattenedAppendGrouped { marker, until, .. } => {
                let variable_data = self.get_variable_data(interpreter)?;
                variable_data.get_mut(self)?.push_grouped(
                    |inner| until.handle_parse_into(input, inner),
                    Delimiter::None,
                    marker.span,
                )?;
            }
            DestructureVariable::FlattenedAppendFlattened { until, .. } => {
                let variable_data = self.get_variable_data(interpreter)?;
                until.handle_parse_into(input, variable_data.get_mut(self)?.deref_mut())?;
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
    fn into_interpreted(self) -> InterpretedStream {
        match self {
            ParsedTokenTree::NoneGroup(group) => InterpretedStream::raw(group.stream()),
            ParsedTokenTree::Ident(ident) => InterpretedStream::raw(ident.to_token_stream()),
            ParsedTokenTree::Punct(punct) => InterpretedStream::raw(punct.to_token_stream()),
            ParsedTokenTree::Literal(literal) => InterpretedStream::raw(literal.to_token_stream()),
        }
    }

    fn push_as_token_tree(self, output: &mut InterpretedStream) {
        match self {
            ParsedTokenTree::NoneGroup(group) => {
                output.push_raw_token_tree(TokenTree::Group(group))
            }
            ParsedTokenTree::Ident(ident) => output.push_ident(ident),
            ParsedTokenTree::Punct(punct) => output.push_punct(punct),
            ParsedTokenTree::Literal(literal) => output.push_literal(literal),
        }
    }

    fn flatten_into(self, output: &mut InterpretedStream) {
        match self {
            ParsedTokenTree::NoneGroup(group) => output.extend_raw_tokens(group.stream()),
            ParsedTokenTree::Ident(ident) => output.push_ident(ident),
            ParsedTokenTree::Punct(punct) => output.push_punct(punct),
            ParsedTokenTree::Literal(literal) => output.push_literal(literal),
        }
    }
}

impl Parse for ParsedTokenTree {
    fn parse(input: ParseStream) -> ParseResult<Self> {
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

#[derive(Clone)]
pub(crate) enum ParseUntil {
    End,
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl ParseUntil {
    /// Peeks the next token, to discover what we should parse next
    fn peek_flatten_limit<C: StopCondition>(input: ParseStream) -> ParseResult<ParseUntil> {
        if C::should_stop(input) {
            return Ok(ParseUntil::End);
        }
        Ok(match detect_preinterpret_grammar(input.cursor()) {
            PeekMatch::GroupedCommand(_)
            | PeekMatch::FlattenedCommand(_)
            | PeekMatch::GroupedVariable
            | PeekMatch::FlattenedVariable
            | PeekMatch::Destructurer(_)
            | PeekMatch::AppendVariableDestructuring => {
                return input
                    .span()
                    .parse_err("This cannot follow a flattened destructure match");
            }
            PeekMatch::Group(delimiter) => ParseUntil::Group(delimiter),
            PeekMatch::Ident(ident) => ParseUntil::Ident(ident),
            PeekMatch::Literal(literal) => ParseUntil::Literal(literal),
            PeekMatch::Punct(punct) => ParseUntil::Punct(punct),
            PeekMatch::End => ParseUntil::End,
        })
    }

    fn handle_parse_into(
        &self,
        input: ParseStream,
        output: &mut InterpretedStream,
    ) -> ExecutionResult<()> {
        match self {
            ParseUntil::End => output.extend_raw_tokens(input.parse::<TokenStream>()?),
            ParseUntil::Group(delimiter) => {
                while !input.is_empty() {
                    if input.peek_group_matching(*delimiter) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
            ParseUntil::Ident(ident) => {
                let content = ident.to_string();
                while !input.is_empty() {
                    if input.peek_ident_matching(&content) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
            ParseUntil::Punct(punct) => {
                let punct = punct.as_char();
                while !input.is_empty() {
                    if input.peek_punct_matching(punct) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
            ParseUntil::Literal(literal) => {
                let content = literal.to_string();
                while !input.is_empty() {
                    if input.peek_literal_matching(&content) {
                        return Ok(());
                    }
                    output.push_raw_token_tree(input.parse()?);
                }
            }
        }
        Ok(())
    }
}
