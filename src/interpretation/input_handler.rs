use super::*;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub(crate) struct ParserHandle;
}

pub(crate) struct InputHandler {
    parsers: SlotMap<ParserHandle, ParseStack<'static, Output>>,
    parser_stack: Vec<ParserHandle>,
}

impl InputHandler {
    pub(crate) fn new() -> Self {
        Self {
            parsers: SlotMap::with_key(),
            parser_stack: Vec::new(),
        }
    }

    /// SAFETY:
    /// * Must be paired with a later `finish_parse` call, even in the face of control flow interrupts.
    /// * `finish_parse` must be called whilst the input is still alive.
    ///
    /// TODO: Replace this with returning a ParseGuard which captures the lifetime and handles calling
    /// `finish_parse` automatically when dropped, to avoid misuse.
    /// EDIT: That doesn't work because the ParseGuard would need to borrow InputHandler mutably,
    /// preventing further use of InputHandler while the guard is alive.
    pub(super) unsafe fn start_parse(&mut self, input: ParseStream<Output>) -> ParserHandle {
        let parse_stack = ParseStack::new(input);

        self.parsers.insert(std::mem::transmute::<
            ParseStack<'_, Output>,
            ParseStack<'static, Output>,
        >(parse_stack))
    }

    /// SAFETY:
    /// * Must be called after a prior `start_parse` call, and while the input is still alive.
    pub(super) unsafe fn finish_parse(&mut self, handle: ParserHandle) -> Result<(), ParseError> {
        let stack = self
            .parsers
            .get(handle)
            .expect("finish_parse called with invalid handle");

        // Check for unclosed groups before removing the parser
        if let Some(delimiter) = stack.innermost_active_group_delimiter() {
            return stack.parse_err(format!("expected '{}'", delimiter.description_of_close()));
        }

        self.parsers.remove(handle);
        Ok(())
    }

    /// SAFETY:
    /// * Must be paired with pop_current_handle
    /// * finish_parse of the handle must not be called before pop_current_handle is called.
    pub(super) unsafe fn push_current_handle(&mut self, handle: ParserHandle) {
        self.parser_stack.push(handle);
    }

    /// SAFETY: Must be paired with push_current_handle
    pub(super) unsafe fn pop_current_handle(&mut self, handle: ParserHandle) {
        let popped_handle = self.parser_stack.pop();
        assert_eq!(
            popped_handle,
            Some(handle),
            "Popped handle does not match the provided handle"
        );
    }

    pub(super) fn get(&mut self, handle: ParserHandle) -> Option<&mut ParseStack<'static, Output>> {
        self.parsers.get_mut(handle)
    }

    pub(super) fn current_stack(&mut self) -> &mut ParseStack<'static, Output> {
        match self.parser_stack.last() {
            Some(parser_handle) => self
                .parsers
                .get_mut(*parser_handle)
                .expect("Parser handle in stack must be valid"),
            None => {
                panic!("There is no input stream available to read from. Consuming from an input stream should only be possible when a parser is available.")
            }
        }
    }

    pub(super) fn current_input<'a>(&'a mut self) -> ParseStream<'a, Output> {
        self.current_stack().current()
    }

    /// Start a fork on all active parsers.
    ///
    /// # Safety
    /// Must be paired with either `commit_fork` or `rollback_fork`.
    pub(super) unsafe fn start_fork(&mut self) {
        for (_, parser) in self.parsers.iter_mut() {
            parser.start_fork();
        }
    }

    /// Commit the fork on all active parsers.
    ///
    /// # Safety
    /// Must be called after `start_fork`.
    pub(super) unsafe fn commit_fork(&mut self) {
        for (_, parser) in self.parsers.iter_mut() {
            if parser.is_forked() {
                parser.commit_fork();
            }
        }
    }

    /// Rollback the fork on all active parsers.
    ///
    /// # Safety
    /// Must be called after `start_fork`.
    pub(super) unsafe fn rollback_fork(&mut self) {
        for (_, parser) in self.parsers.iter_mut() {
            if parser.is_forked() {
                parser.rollback_fork();
            }
        }
    }
}
