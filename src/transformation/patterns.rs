use crate::internal_prelude::*;

pub(crate) trait HandleDestructure {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()>;
}

#[derive(Clone)]
pub(crate) enum Pattern {
    Variable(VariablePattern),
    Array(ArrayPattern),
    Object(ObjectPattern),
    Stream(ExplicitTransformStream),
    DotDot(Token![..]),
    #[allow(unused)]
    Discarded(Token![_]),
}

impl Parse<Source> for Pattern {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(Pattern::Variable(input.parse()?))
        } else if lookahead.peek(syn::token::Bracket) {
            Ok(Pattern::Array(input.parse()?))
        } else if lookahead.peek(syn::token::Brace) {
            Ok(Pattern::Object(input.parse()?))
        } else if lookahead.peek(Token![@]) {
            Ok(Pattern::Stream(input.parse()?))
        } else if lookahead.peek(Token![_]) {
            Ok(Pattern::Discarded(input.parse()?))
        } else if lookahead.peek(Token![..]) {
            Ok(Pattern::DotDot(input.parse()?))
        } else if input.peek(Token![#]) {
            return input.parse_err("Use `var` instead of `#var` in a destructuring");
        } else {
            Err(lookahead.error().into())
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
            Pattern::DotDot(token) => token.execution_err("This cannot be used here"),
            Pattern::Discarded(_) => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct ArrayPattern {
    #[allow(unused)]
    brackets: Brackets,
    items: Punctuated<Pattern, Token![,]>,
}

impl Parse<Source> for ArrayPattern {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (brackets, inner) = input.parse_brackets()?;
        Ok(Self {
            brackets,
            items: inner.parse_terminated()?,
        })
    }
}

impl HandleDestructure for ArrayPattern {
    /// See also `ArrayAssigneeStackFrame` in `evaluation.rs`
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        let array = value.expect_array("The value destructured with an array pattern")?;
        let mut has_seen_dot_dot = false;
        let mut prefix_assignees = Vec::new();
        let mut suffix_assignees = Vec::new();
        for pattern in self.items.iter() {
            match pattern {
                Pattern::DotDot(dot_dot) => {
                    if has_seen_dot_dot {
                        return dot_dot.execution_err("Only one .. is allowed in an array pattern");
                    }
                    has_seen_dot_dot = true;
                }
                _ => {
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

#[derive(Clone)]
pub struct ObjectPattern {
    #[allow(unused)]
    braces: Braces,
    entries: Punctuated<ObjectEntry, Token![,]>,
}

impl Parse<Source> for ObjectPattern {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (braces, inner) = input.parse_braces()?;
        Ok(Self {
            braces,
            entries: inner.parse_terminated()?,
        })
    }
}

impl HandleDestructure for ObjectPattern {
    /// See also `ObjectAssigneeStackFrame` in `evaluation.rs`
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        let object = value.expect_object("The value destructured with an object pattern")?;
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
                .unwrap_or_else(|| ExpressionValue::None(key_span.span_range()));
            already_used_keys.insert(key);
            pattern.handle_destructure(interpreter, value)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
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

impl Parse<Source> for ObjectEntry {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
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
                    name: field.clone(),
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
}
