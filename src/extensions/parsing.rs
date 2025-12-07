use crate::internal_prelude::*;

pub(crate) trait TokenStreamParseExt: Sized {
    fn source_parse_and_analyze<T>(
        self,
        parser: impl FnOnce(SourceParser) -> ParseResult<T>,
        control_flow_analysis: impl FnOnce(&mut T, FlowCapturer) -> ParseResult<()>,
    ) -> ParseResult<(T, ScopeDefinitions)>;

    fn interpreted_parse_with<T, E: From<ParseError>>(
        self,
        parser: impl FnOnce(ParseStream<Output>) -> Result<T, E>,
    ) -> Result<T, E>;
}

impl TokenStreamParseExt for TokenStream {
    fn source_parse_and_analyze<T>(
        self,
        parser: impl FnOnce(SourceParser) -> ParseResult<T>,
        control_flow_analysis: impl FnOnce(&mut T, FlowCapturer) -> ParseResult<()>,
    ) -> ParseResult<(T, ScopeDefinitions)> {
        let mut parsed = parse_with(self, parse_without_analysis(parser))?;
        let definitions = ControlFlowContext::analyze(&mut parsed, control_flow_analysis)?;
        Ok((parsed, definitions))
    }

    fn interpreted_parse_with<T, E: From<ParseError>>(
        self,
        parser: impl FnOnce(ParseStream<Output>) -> Result<T, E>,
    ) -> Result<T, E> {
        parse_with(self, parser)
    }
}

pub(crate) fn parse_with<T, K, E: From<ParseError>>(
    stream: TokenStream,
    parser: impl FnOnce(ParseStream<K>) -> Result<T, E>,
) -> Result<T, E> {
    let mut result = None;
    let parse_result = (|input: SynParseStream| -> SynResult<()> {
        result = Some(parser(input.into()));
        match &result {
            // Some fallback error to ensure that we don't go down the unexpected branch inside parse2
            Some(Err(_)) => Err(SynError::new(Span::call_site(), "")),
            _ => Ok(()),
        }
    })
    .parse2(stream);

    match (result, parse_result) {
        (Some(Ok(value)), Ok(())) => Ok(value),
        (Some(Err(error)), _) => Err(error),
        // If the inner result was Ok, but the parse result was an error, this indicates that the parse2
        // hit the "unexpected" path, indicating that some parse buffer (i.e. group) wasn't fully consumed.
        // So we propagate this error.
        (Some(Ok(_)), Err(error)) => Err(E::from(ParseError::new(error))),
        (None, _) => unreachable!(),
    }
}

pub(crate) trait CursorExt: Sized {
    /// Because syn doesn't parse ' as a punct (not aligned with the TokenTree abstraction)
    fn any_punct(self) -> Option<(Punct, Self)>;
    fn ident_matching(self, content: &str) -> Option<(Ident, Self)>;
    fn punct_matching(self, char: char) -> Option<(Punct, Self)>;
    fn literal_matching(self, content: &str) -> Option<(Literal, Self)>;
    fn group_matching(self, expected_delimiter: Delimiter) -> Option<(DelimSpan, Self, Self)>;
}

impl CursorExt for Cursor<'_> {
    fn any_punct(self) -> Option<(Punct, Self)> {
        match self.token_tree() {
            Some((TokenTree::Punct(punct), next)) => Some((punct, next)),
            _ => None,
        }
    }

    fn ident_matching(self, content: &str) -> Option<(Ident, Self)> {
        match self.ident() {
            Some((ident, next)) if ident == content => Some((ident, next)),
            _ => None,
        }
    }

    fn punct_matching(self, char: char) -> Option<(Punct, Self)> {
        // self.punct() is a little more efficient, but can't match '
        let matcher = if char == '\'' {
            self.any_punct()
        } else {
            self.punct()
        };
        match matcher {
            Some((punct, next)) if punct.as_char() == char => Some((punct, next)),
            _ => None,
        }
    }

    fn literal_matching(self, content: &str) -> Option<(Literal, Self)> {
        match self.literal() {
            Some((literal, next)) if literal.to_string() == content => Some((literal, next)),
            _ => None,
        }
    }

    fn group_matching(self, expected_delimiter: Delimiter) -> Option<(DelimSpan, Self, Self)> {
        match self.any_group() {
            Some((inner_cursor, delimiter, delim_span, next_outer_cursor))
                if delimiter == expected_delimiter =>
            {
                Some((delim_span, inner_cursor, next_outer_cursor))
            }
            _ => None,
        }
    }
}

pub(crate) trait DelimiterExt {
    fn description_of_open(&self) -> &'static str;
    fn description_of_close(&self) -> &'static str;
    #[allow(unused)]
    fn description_of_group(&self) -> &'static str;
}

impl DelimiterExt for Delimiter {
    fn description_of_open(&self) -> &'static str {
        match self {
            Delimiter::Parenthesis => "(",
            Delimiter::Brace => "{",
            Delimiter::Bracket => "[",
            Delimiter::None => "start of transparent group, from a grouped macro $variable substitution or preinterpret %group[...] literal",
        }
    }

    fn description_of_close(&self) -> &'static str {
        match self {
            Delimiter::Parenthesis => ")",
            Delimiter::Brace => "}",
            Delimiter::Bracket => "]",
            Delimiter::None => "end of transparent group, from a grouped macro $variable substitution or preinterpret %group[...] literal",
        }
    }

    fn description_of_group(&self) -> &'static str {
        match self {
            Delimiter::Parenthesis => "(...)",
            Delimiter::Brace => "{ ... }",
            Delimiter::Bracket => "[...]",
            Delimiter::None => "transparent group, from a grouped macro $variable substitution or preinterpret %group[...] literal",
        }
    }
}

/// Allows storing a stack of parse buffers for certain parse strategies which require
/// handling multiple groups in parallel.
///
/// Supports nested forking for attempt blocks: each fork level adds a new buffer to each
/// active level's stack. On commit, buffers are advanced; on rollback, fork buffers are discarded.
pub(crate) struct ParseStack<'a, K> {
    /// The base level (root stream + its forks)
    base: BaseLevel<'a, K>,
    /// Group levels, each with their own fork stacks
    groups: Vec<GroupLevel<'a, K>>,
}

/// The base (root) level of the parse stack.
struct BaseLevel<'a, K> {
    /// The original parse stream (a reference)
    root: ParseStream<'a, K>,
    /// Forks at each depth: forks[i] is the fork at depth i+1
    forks: Vec<ParseBuffer<'a, K>>,
}

/// A group level in the parse stack.
struct GroupLevel<'a, K> {
    /// The delimiter used to enter this group.
    delimiter: Delimiter,
    /// Stack of buffers for this group.
    /// - buffers[0] is at depth `fork_depth - buffers.len() + 1`
    /// - buffers.last() is at the current fork_depth
    ///
    /// When a level's buffers becomes empty after commit/rollback, the level is removed.
    buffers: Vec<GroupBuffer<'a, K>>,
}

/// State of a group buffer at a particular fork depth.
enum GroupBuffer<'a, K> {
    /// Group is active at this depth
    Active(ParseBuffer<'a, K>),
    /// Group was exited at this depth
    Ended,
}

impl<'a, K> ParseStack<'a, K> {
    pub(crate) fn new(base: ParseStream<'a, K>) -> Self {
        Self {
            base: BaseLevel {
                root: base,
                forks: Vec::new(),
            },
            groups: Vec::new(),
        }
    }

    /// Returns the current fork depth (0 = not forked).
    pub(crate) fn fork_depth(&self) -> usize {
        self.base.forks.len()
    }

    /// Start a fork. From this point, parsing operations work on forked buffers.
    ///
    /// # Safety
    /// Must be paired with either `commit_fork` or `rollback_fork`.
    pub(crate) unsafe fn start_fork(&mut self) {
        // Fork the base level
        let base_current = self
            .base
            .forks
            .last()
            .map(|f| f as &_)
            .unwrap_or(self.base.root);
        self.base.forks.push(base_current.fork());

        // Fork each group level
        for group in &mut self.groups {
            match group.buffers.last() {
                Some(GroupBuffer::Active(buffer)) => {
                    group.buffers.push(GroupBuffer::Active(buffer.fork()));
                }
                Some(GroupBuffer::Ended) => {
                    // Group was ended at previous depth; carry forward the Ended status
                    group.buffers.push(GroupBuffer::Ended);
                }
                None => unreachable!("Group should always have at least one buffer"),
            }
        }
    }

    /// Commit the fork: advance all original buffers to their forked positions.
    ///
    /// # Safety
    /// Must be called after `start_fork`.
    pub(crate) unsafe fn commit_fork(&mut self) {
        assert!(
            self.fork_depth() > 0,
            "commit_fork called without active fork"
        );

        // Commit base level
        let committed_fork = self.base.forks.pop().unwrap();
        if let Some(previous) = self.base.forks.last() {
            previous.advance_to(&committed_fork);
        } else {
            self.base.root.advance_to(&committed_fork);
        }

        // Commit each group level
        for group in &mut self.groups {
            // If buffers.len() == 1, group was created at this depth.
            // On commit, it survives at the new (lower) depth - don't pop.
            if group.buffers.len() == 1 {
                continue;
            }

            // Pop the buffer at the current depth
            let popped = group.buffers.pop().unwrap();

            match popped {
                GroupBuffer::Active(buffer) => {
                    // Advance the previous depth to this position
                    if let Some(GroupBuffer::Active(previous)) = group.buffers.last() {
                        previous.advance_to(&buffer);
                    }
                    // If previous was Ended, nothing to advance
                }
                GroupBuffer::Ended => {
                    // Group was exited at this depth; propagate to previous depth
                    if let Some(previous) = group.buffers.last_mut() {
                        *previous = GroupBuffer::Ended;
                    }
                }
            }
        }

        // Clean up groups that are ended at depth 0
        if self.fork_depth() == 0 {
            self.groups
                .retain(|g| matches!(g.buffers.last(), Some(GroupBuffer::Active(_))));
        }
    }

    /// Rollback the fork: discard fork state, leaving originals unchanged.
    ///
    /// # Safety
    /// Must be called after `start_fork`.
    pub(crate) unsafe fn rollback_fork(&mut self) {
        assert!(
            self.fork_depth() > 0,
            "rollback_fork called without active fork"
        );

        // Rollback base level
        self.base.forks.pop();

        // Rollback each group level
        self.groups.retain_mut(|group| {
            group.buffers.pop();
            // If group has no more buffers, it was entered during this fork; remove it
            !group.buffers.is_empty()
        });
    }

    pub(crate) fn is_forked(&self) -> bool {
        self.fork_depth() > 0
    }

    pub(crate) fn current(&self) -> ParseStream<'_, K> {
        // Check groups from top to bottom for an active buffer
        for group in self.groups.iter().rev() {
            if let Some(GroupBuffer::Active(buffer)) = group.buffers.last() {
                return buffer;
            }
            // If Ended, continue to next group (or base)
        }
        // Fall back to base level
        self.base
            .forks
            .last()
            .map(|f| f as &_)
            .unwrap_or(self.base.root)
    }

    pub(crate) fn cursor(&self) -> Cursor<'_> {
        self.current().cursor()
    }

    pub(crate) fn parse_err<T>(&self, message: impl std::fmt::Display) -> ParseResult<T> {
        self.current().parse_err(message)
    }

    pub(crate) fn is_current_empty(&self) -> bool {
        self.current().is_empty()
    }

    #[allow(unused)]
    pub(crate) fn peek<T: syn::parse::Peek>(&mut self, token: T) -> bool {
        self.current().peek(token)
    }

    #[allow(unused)]
    pub(crate) fn peek2<T: syn::parse::Peek>(&mut self, token: T) -> bool {
        self.current().peek2(token)
    }

    pub(crate) fn parse_any_ident(&mut self) -> ParseResult<Ident> {
        self.current().parse_any_ident()
    }

    pub(crate) fn parse_and_enter_group(
        &mut self,
        delimiter: Option<Delimiter>,
    ) -> ParseResult<(Delimiter, DelimSpan)> {
        let (delimiter, delim_span, inner) = self.current().parse_group(delimiter)?;
        let inner = unsafe {
            // SAFETY: This is safe because the lifetime is there for two reasons:
            // (A) Prevent mixing up different buffers from e.g. different groups,
            // (B) Ensure the buffers are dropped in the correct order so that the unexpected drop glue triggers
            // in the correct order.
            //
            // This invariant is maintained by this `ParseStack` struct:
            // (A) Is enforced by the fact we're parsing the group from the top parse buffer current().
            // (B) Is enforced by a combination of:
            // ==> exit_group() ensures the parse buffers are dropped in the correct order
            // ==> If a user forgets to do it (or e.g. an error path or panic causes exit_group not to be called)
            //     Then the drop glue ensures the groups are dropped in the correct order.
            std::mem::transmute::<ParseBuffer<'_, K>, ParseBuffer<'a, K>>(inner)
        };

        // Create a new group level with one buffer at the current fork depth
        self.groups.push(GroupLevel {
            delimiter,
            buffers: vec![GroupBuffer::Active(inner)],
        });
        Ok((delimiter, delim_span))
    }

    /// Returns true if there is an active group that can be exited.
    pub(crate) fn has_active_group(&self) -> bool {
        self.groups
            .iter()
            .any(|g| matches!(g.buffers.last(), Some(GroupBuffer::Active(_))))
    }

    /// Should be paired with `parse_and_enter_group`.
    ///
    /// If the group is not finished, the next attempt to read from the parent will trigger an error,
    /// in accordance with the drop glue on `ParseBuffer`.
    ///
    /// If `expected_delimiter` is provided, it will be validated against the actual delimiter
    /// used when entering the group.
    ///
    /// ### Returns
    /// Returns an error if there is no group to exit or if the expected delimiter doesn't match.
    pub(crate) fn exit_group(&mut self, expected_delimiter: Option<Delimiter>) -> ParseResult<()> {
        // Find the innermost active group (what current() would return)
        let group_index = self
            .groups
            .iter()
            .rposition(|g| matches!(g.buffers.last(), Some(GroupBuffer::Active(_))))
            .ok_or_else(|| self.current().parse_error("no group to close"))?;

        // Validate delimiter if expected
        if let Some(expected) = expected_delimiter {
            let actual = self.groups[group_index].delimiter;
            if actual != expected {
                return self.parse_err(format!(
                    "attempting to close '{}' isn't valid, because the currently open group would end with '{}'",
                    expected.description_of_close(),
                    actual.description_of_close()
                ));
            }
        }

        if self.fork_depth() == 0 {
            // Not forked: remove the group entirely
            self.groups.remove(group_index);
        } else {
            // Forked: mark as ended
            let group = &mut self.groups[group_index];
            if let Some(buffer) = group.buffers.last_mut() {
                *buffer = GroupBuffer::Ended;
            }
        }
        Ok(())
    }
}

impl<'a> ParseStack<'a, Source> {
    pub(crate) fn parse<T: ParseSource>(&mut self) -> ParseResult<T> {
        self.current().parse()
    }

    pub(crate) fn peek_grammar(&mut self) -> SourcePeekMatch {
        self.current().peek_grammar()
    }

    pub(crate) fn try_parse_or_revert<T: ParseSource>(&mut self) -> ParseResult<T> {
        let current = self.current();
        let fork = current.fork();
        match fork.parse::<T>() {
            Ok(output) => {
                current.advance_to(&fork);
                Ok(output)
            }
            Err(err) => Err(err),
        }
    }
}

impl<K> Drop for ParseStack<'_, K> {
    fn drop(&mut self) {
        // Drop groups in reverse order (innermost first)
        // Each group's buffers are also dropped in reverse order
        while let Some(mut group) = self.groups.pop() {
            while group.buffers.pop().is_some() {}
        }

        // Drop base forks in reverse order
        while self.base.forks.pop().is_some() {}
    }
}
