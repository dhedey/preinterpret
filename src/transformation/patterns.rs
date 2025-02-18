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
