use crate::internal_prelude::*;

/// `InputHandler` manages a stack of input parse streams for the interpreter.
///
/// This is the counterpart to `OutputHandler` for input handling. While `OutputHandler`
/// manages a flat stack of `OutputStream`s, `InputHandler` manages a stack of
/// `ParseStreamStack`s, because parsing may need to handle nested groups.
///
/// ## Safety
///
/// The `ParseStreamStack` contains references with lifetime `'a` from the parsing context.
/// To store these in the interpreter (which doesn't have that lifetime parameter), we
/// transmute the lifetime to `'static`. This is safe as long as:
///
/// 1. The `InputHandler` is only used within the scope where the original `ParseStream` is valid
/// 2. Input streams are pushed and popped in a LIFO manner
/// 3. The interpreter does not outlive the parsing context
///
/// The safety is enforced by the API design:
/// - `push_input` and `pop_input` must be called in matching pairs
/// - The interpreter is created and destroyed within a single parsing invocation
pub(super) struct InputHandler {
    /// Stack of input parse stream stacks.
    /// Each entry represents a separate parsing context (e.g., for nested parsing).
    /// The lifetime has been transmuted to 'static - see safety documentation above.
    input_stack: Vec<ParseStreamStackStatic>,
    /// Stack of revertible entries (forks) that are currently active.
    /// These are forks that parsing uses during revertible segments.
    /// On commit, the original is advanced; on revert, the fork is discarded.
    revertible_stack: Vec<RevertibleInputEntry>,
}

/// A `ParseStreamStack` with the lifetime transmuted to `'static`.
///
/// ## Safety
///
/// This is only safe to use within the original parsing context's lifetime.
/// The transmutation is necessary because:
/// 1. `ParseStreamStack` contains `ParseBuffer` which borrows from the token buffer
/// 2. The interpreter needs to store these without a lifetime parameter
/// 3. The interpreter is always used within the parsing context
struct ParseStreamStackStatic {
    /// The inner stack with transmuted lifetime.
    /// SAFETY: Must only be accessed while the original parse context is valid.
    inner: ParseStreamStack<'static, Output>,
}

impl ParseStreamStackStatic {
    /// Creates a new `ParseStreamStackStatic` by transmuting the lifetime.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that:
    /// 1. The resulting value is not used after the original `ParseStream`'s lifetime ends
    /// 2. The value is dropped before the parsing context ends
    unsafe fn new(input: ParseStream<Output>) -> Self {
        Self {
            inner: ParseStreamStack::new(std::mem::transmute::<
                ParseStream<'_, Output>,
                ParseStream<'static, Output>,
            >(input)),
        }
    }

    /// Gets the current parse stream.
    fn current(&self) -> ParseStream<'_, Output> {
        self.inner.current()
    }

    /// Gets the inner stack mutably.
    fn inner_mut(&mut self) -> &mut ParseStreamStack<'static, Output> {
        &mut self.inner
    }

    /// Forks the current position for speculative parsing.
    ///
    /// Returns a forked version that can be independently advanced.
    /// Call `commit_fork` to accept the fork's position, or just drop it to discard.
    fn fork(&self) -> ParseStreamStackFork {
        // We fork by creating a new ParseStreamStack from the current stream's fork
        let forked_stream = self.inner.current().fork();
        ParseStreamStackFork {
            // SAFETY: Same lifetime constraints as the parent
            inner: unsafe {
                ParseStreamStack::new(std::mem::transmute::<
                    SynParseBuffer<'_>,
                    SynParseBuffer<'static>,
                >(forked_stream))
            },
        }
    }

    /// Commits a fork, advancing this stack's position to match the fork.
    fn commit_fork(&mut self, fork: ParseStreamStackFork) {
        // Advance the current stream to the fork's position
        self.inner.current().advance_to(&fork.inner.current());
    }
}

/// Represents a forked input entry that can be committed or reverted.
///
/// When entering revertible mode, we fork the current input and push it onto
/// the stack. Parsing then uses this fork. On commit, we advance the original
/// to match; on revert, we just pop without advancing.
struct RevertibleInputEntry {
    /// The forked parse stream stack
    fork: ParseStreamStack<'static, Output>,
    /// Index in input_stack of the original that was forked.
    /// This is needed to commit the fork back to the original.
    original_index: usize,
}

#[derive(Debug)]
pub(super) enum InputHandlerError {
    NoInputAvailable,
}

impl InputHandler {
    pub(super) fn new() -> Self {
        Self {
            input_stack: vec![],
            revertible_stack: vec![],
        }
    }

    /// Returns true if there is currently an input stream available.
    pub(super) fn has_input(&self) -> bool {
        !self.input_stack.is_empty() || !self.revertible_stack.is_empty()
    }

    /// Gets the current input stream for reading/parsing.
    ///
    /// If in revertible mode, returns the fork. Otherwise returns the main input.
    ///
    /// Returns an error if no input is available.
    pub(super) fn current_input(&self) -> Result<ParseStream<'_, Output>, InputHandlerError> {
        // First check revertible stack - if we're in revertible mode, use the fork
        if let Some(revertible) = self.revertible_stack.last() {
            return Ok(revertible.fork.current());
        }

        // Otherwise use the main input stack
        if self.input_stack.is_empty() {
            return Err(InputHandlerError::NoInputAvailable);
        }

        Ok(self.input_stack.last().unwrap().current())
    }

    /// Gets the current input stack mutably for operations like entering/exiting groups.
    ///
    /// If in revertible mode, returns the fork's stack. Otherwise returns the main input.
    ///
    /// Returns an error if no input is available.
    pub(super) fn current_input_stack_mut(
        &mut self,
    ) -> Result<&mut ParseStreamStack<'static, Output>, InputHandlerError> {
        // First check revertible stack - if we're in revertible mode, use the fork
        if let Some(revertible) = self.revertible_stack.last_mut() {
            return Ok(&mut revertible.fork);
        }

        // Otherwise use the main input stack
        if self.input_stack.is_empty() {
            return Err(InputHandlerError::NoInputAvailable);
        }

        Ok(self.input_stack.last_mut().unwrap().inner_mut())
    }

    /// Pushes a new input stream onto the stack.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that `pop_input` is called before the original
    /// `ParseStream`'s lifetime ends, and that the interpreter does not outlive
    /// the parsing context.
    pub(super) unsafe fn push_input(&mut self, input: ParseStream<Output>) {
        self.input_stack.push(ParseStreamStackStatic::new(input));
    }

    /// Pops the current input stream from the stack.
    ///
    /// ## Panics
    ///
    /// Panics if the input stack is empty or if there are uncommitted revertible entries
    /// that reference this input.
    pub(super) fn pop_input(&mut self) {
        let current_index = self.input_stack.len() - 1;

        // Check that no revertible entries reference this input
        assert!(
            !self
                .revertible_stack
                .iter()
                .any(|r| r.original_index == current_index),
            "Cannot pop input while revertible entries reference it"
        );

        self.input_stack
            .pop()
            .expect("Cannot pop from empty input stack");
    }

    /// Enters revertible mode for the current input.
    ///
    /// This forks the current input so that parsing happens on the fork.
    /// Call `commit_revertible` to accept the parsing, or `revert_revertible`
    /// to discard it.
    ///
    /// Does nothing if there is no current input (allows use in contexts where
    /// input may or may not be present).
    ///
    /// ## Safety
    ///
    /// Must be paired with either `commit_revertible` or `revert_revertible`.
    pub(super) unsafe fn enter_revertible(&mut self) {
        if self.input_stack.is_empty() {
            return;
        }

        let original_index = self.input_stack.len() - 1;
        let forked_stream = self.input_stack[original_index].inner.current().fork();

        self.revertible_stack.push(RevertibleInputEntry {
            fork: ParseStreamStack::new(std::mem::transmute::<
                SynParseBuffer<'_>,
                SynParseBuffer<'static>,
            >(forked_stream)),
            original_index,
        });
    }

    /// Commits the current revertible entry, advancing the original input
    /// to match the fork's position.
    ///
    /// ## Safety
    ///
    /// Must be paired with `enter_revertible`.
    ///
    /// ## Panics
    ///
    /// Panics if not in revertible mode, or if the original input is no longer
    /// at the expected position in the stack.
    pub(super) unsafe fn commit_revertible(&mut self) {
        let entry = self
            .revertible_stack
            .pop()
            .expect("commit_revertible called without matching enter_revertible");

        // Verify the original is still where we expect
        assert!(
            entry.original_index < self.input_stack.len(),
            "Original input was removed during revertible segment"
        );

        // Advance the original to match the fork's position
        self.input_stack[entry.original_index]
            .inner
            .current()
            .advance_to(&entry.fork.current());
    }

    /// Reverts the current revertible entry, discarding any parsing done
    /// on the fork. The original input position remains unchanged.
    ///
    /// ## Safety
    ///
    /// Must be paired with `enter_revertible`.
    ///
    /// ## Panics
    ///
    /// Panics if not in revertible mode.
    pub(super) unsafe fn revert_revertible(&mut self) {
        let _entry = self
            .revertible_stack
            .pop()
            .expect("revert_revertible called without matching enter_revertible");

        // Just dropping the entry discards the fork - the original is unchanged
    }

    /// Returns true if currently in revertible mode.
    #[allow(unused)]
    pub(super) fn is_in_revertible_mode(&self) -> bool {
        !self.revertible_stack.is_empty()
    }
}
