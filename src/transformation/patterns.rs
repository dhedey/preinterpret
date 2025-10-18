use crate::internal_prelude::*;

pub(crate) trait HandleDestructure {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()>;
}

pub(crate) enum Pattern {
    Variable(VariablePattern),
    Array(ArrayPattern),
    Object(ObjectPattern),
    Stream(StreamPattern),
    #[allow(unused)]
    Discarded(Token![_]),
}

impl ParseSource for Pattern {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(Pattern::Variable(input.parse()?))
        } else if lookahead.peek(syn::token::Bracket) {
            Ok(Pattern::Array(input.parse()?))
        } else if lookahead.peek(Token![%]) {
            let (_, next) = input.cursor().punct_matching('%').unwrap();
            if next.group_matching(Delimiter::Brace).is_some() {
                Ok(Pattern::Object(input.parse()?))
            } else if next.group_matching(Delimiter::Bracket).is_some() {
                Ok(Pattern::Stream(input.parse()?))
            } else if next.ident_matching("raw").is_some() {
                input.parse_err(
                    "Use `%[@[EXACT(%raw[...])]]` to match `%raw[...]` stream literal content",
                )
            } else if next.ident_matching("group").is_some() {
                // TODO[parsers]: Check this is the correct syntax once implemented!
                input.parse_err(
                    "Use `%[@[GROUP ...]]` to match `%group[...]` stream literal content",
                )
            } else {
                input.parse_err("Expected a pattern, such as an object pattern `%{ ... }` or stream pattern `%[ ... ]`")
            }
        } else if lookahead.peek(Token![_]) {
            Ok(Pattern::Discarded(input.parse()?))
        } else if input.peek(Token![#]) {
            input.parse_err("Use `var` instead of `#var` in a destructuring")
        } else {
            Err(lookahead.error().into())
        }
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            Pattern::Variable(variable) => variable.control_flow_pass(context),
            Pattern::Array(array) => array.control_flow_pass(context),
            Pattern::Object(object) => object.control_flow_pass(context),
            Pattern::Stream(stream) => stream.control_flow_pass(context),
            Pattern::Discarded(discarded) => discarded.control_flow_pass(context),
        }
    }
}

impl HandleDestructure for Pattern {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        match self {
            Pattern::Variable(variable) => variable.handle_destructure(interpreter, value),
            Pattern::Array(array) => array.handle_destructure(interpreter, value),
            Pattern::Object(object) => object.handle_destructure(interpreter, value),
            Pattern::Stream(stream) => stream.handle_destructure(interpreter, value),
            Pattern::Discarded(_) => Ok(()),
        }
    }
}

pub struct ArrayPattern {
    #[allow(unused)]
    brackets: Brackets,
    items: Punctuated<PatternOrDotDot, Token![,]>,
}

impl ParseSource for ArrayPattern {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let (brackets, inner) = input.parse_brackets()?;
        Ok(Self {
            brackets,
            items: inner.parse_terminated()?,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        for item in self.items.iter_mut() {
            item.control_flow_pass(context)?;
        }
        Ok(())
    }
}

impl HandleDestructure for ArrayPattern {
    /// See also `ArrayAssigneeStackFrame` in `evaluation.rs`
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        let array: ExpressionArray = value
            .into_owned(self.brackets.span_range())
            .resolve_as("The value destructured with an array pattern")?;
        let mut has_seen_dot_dot = false;
        let mut prefix_assignees = Vec::new();
        let mut suffix_assignees = Vec::new();
        for pattern in self.items.iter() {
            match pattern {
                PatternOrDotDot::DotDot(dot_dot) => {
                    if has_seen_dot_dot {
                        return dot_dot.execution_err("Only one .. is allowed in an array pattern");
                    }
                    has_seen_dot_dot = true;
                }
                PatternOrDotDot::Pattern(pattern) => {
                    if has_seen_dot_dot {
                        suffix_assignees.push(pattern);
                    } else {
                        prefix_assignees.push(pattern);
                    }
                }
            }
        }
        let array_length = array.items.len();

        let assignee_pairs: Vec<_> = if has_seen_dot_dot {
            let total_assignees = prefix_assignees.len() + suffix_assignees.len();
            if total_assignees > array_length {
                return self.brackets.execution_err(format!(
                    "The array has {} items, but the pattern expected at least {}",
                    array_length, total_assignees,
                ));
            }
            let discarded_count =
                array.items.len() - prefix_assignees.len() - suffix_assignees.len();
            let assignees = prefix_assignees
                .into_iter()
                .map(Some)
                .chain(std::iter::repeat(None).take(discarded_count))
                .chain(suffix_assignees.into_iter().map(Some));

            assignees
                .zip(array.items)
                .filter_map(|(assignee, value)| Some((assignee?, value)))
                .collect()
        } else {
            let total_assignees = prefix_assignees.len();
            if total_assignees != array_length {
                return self.brackets.execution_err(format!(
                    "The array has {} items, but the pattern expected {}",
                    array_length, total_assignees,
                ));
            }
            prefix_assignees.into_iter().zip(array.items).collect()
        };
        for (pattern, value) in assignee_pairs {
            pattern.handle_destructure(interpreter, value)?;
        }
        Ok(())
    }
}

enum PatternOrDotDot {
    Pattern(Pattern),
    DotDot(Token![..]),
}

impl ParseSource for PatternOrDotDot {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        if input.peek(Token![..]) {
            Ok(PatternOrDotDot::DotDot(input.parse()?))
        } else {
            Ok(PatternOrDotDot::Pattern(input.parse()?))
        }
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            PatternOrDotDot::Pattern(pattern) => pattern.control_flow_pass(context),
            PatternOrDotDot::DotDot(_) => Ok(()),
        }
    }
}

pub struct ObjectPattern {
    #[allow(unused)]
    prefix: Token![%],
    #[allow(unused)]
    braces: Braces,
    entries: Punctuated<ObjectEntry, Token![,]>,
}

impl ParseSource for ObjectPattern {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let (braces, inner) = input.parse_braces()?;
        Ok(Self {
            prefix,
            braces,
            entries: inner.parse_terminated()?,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        for entry in self.entries.iter_mut() {
            entry.control_flow_pass(context)?;
        }
        Ok(())
    }
}

impl HandleDestructure for ObjectPattern {
    /// See also `ObjectAssigneeStackFrame` in `evaluation.rs`
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        let object: ExpressionObject = value
            .into_owned(self.braces.span_range())
            .resolve_as("The value destructured with an object pattern")?;
        let mut value_map = object.entries;
        let mut already_used_keys = HashSet::with_capacity(self.entries.len());
        for entry in self.entries.iter() {
            let (key, key_span, pattern) = match entry {
                ObjectEntry::KeyOnly { field, pattern } => {
                    (field.to_string(), field.span(), pattern)
                }
                ObjectEntry::KeyValue { field, pattern, .. } => {
                    (field.to_string(), field.span(), pattern)
                }
                ObjectEntry::IndexValue {
                    access,
                    key,
                    pattern,
                    ..
                } => (key.value(), access.span(), pattern),
            };
            if already_used_keys.contains(&key) {
                return key_span.execution_err(format!("The key `{}` has already used", key));
            }
            let value = value_map
                .remove(&key)
                .map(|entry| entry.value)
                .unwrap_or_else(|| ExpressionValue::None);
            already_used_keys.insert(key);
            pattern.handle_destructure(interpreter, value)?;
        }
        Ok(())
    }
}

enum ObjectEntry {
    KeyOnly {
        field: Ident,
        pattern: Pattern,
    },
    KeyValue {
        field: Ident,
        _colon: Token![:],
        pattern: Pattern,
    },
    IndexValue {
        access: IndexAccess,
        key: syn::LitStr,
        _colon: Token![:],
        pattern: Pattern,
    },
}

impl ParseSource for ObjectEntry {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        if input.peek(syn::Ident) {
            let field = input.parse()?;
            if input.peek(Token![:]) {
                Ok(ObjectEntry::KeyValue {
                    field,
                    _colon: input.parse()?,
                    pattern: input.parse()?,
                })
            } else if input.peek(Token![,]) || input.is_empty() {
                let pattern = Pattern::Variable(VariablePattern {
                    definition: VariableDefinition {
                        ident: field.clone(),
                        id: VariableDefinitionId::new_placeholder(),
                    },
                });
                Ok(ObjectEntry::KeyOnly { field, pattern })
            } else {
                input.parse_err("Expected `:` or `,`")
            }
        } else if input.peek(syn::token::Bracket) {
            let (access, key) = {
                let (brackets, inner) = input.parse_brackets()?;
                (IndexAccess { brackets }, inner.parse()?)
            };
            Ok(ObjectEntry::IndexValue {
                access,
                key,
                _colon: input.parse()?,
                pattern: input.parse()?,
            })
        } else {
            input.parse_err("Expected `property: <pattern>` or `[\"property\"]: <pattern>`")
        }
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            ObjectEntry::KeyOnly { pattern, .. } => pattern.control_flow_pass(context),
            ObjectEntry::KeyValue { pattern, .. } => pattern.control_flow_pass(context),
            ObjectEntry::IndexValue { pattern, .. } => pattern.control_flow_pass(context),
        }
    }
}

pub struct StreamPattern {
    #[allow(unused)]
    prefix: Token![%],
    #[allow(unused)]
    brackets: Brackets,
    content: TransformStream,
}

impl ParseSource for StreamPattern {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let (brackets, inner) = input.parse_brackets()?;
        Ok(Self {
            prefix,
            brackets,
            content: inner.parse()?,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.content.control_flow_pass(context)
    }
}

impl HandleDestructure for StreamPattern {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        let stream: ExpressionStream = value
            .into_owned(self.brackets.span_range())
            .resolve_as("The value destructured with a stream pattern")?;
        let mut discarded = OutputStream::new();
        self.content
            .handle_transform_from_stream(stream.value, interpreter, &mut discarded)?;
        Ok(())
    }
}
