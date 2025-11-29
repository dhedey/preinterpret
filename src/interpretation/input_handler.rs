use super::*;
use slotmap::{SlotMap, new_key_type};

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
    pub(super) unsafe fn start_parse(&mut self, input: ParseStream<Output>) -> ParserHandle {
        let parse_stack = ParseStack::new(input);
        let handle = self.parsers.insert(std::mem::transmute::<
            ParseStack<'_, Output>,
            ParseStack<'static, Output>,
        >(parse_stack));
        self.parser_stack.push(handle);
        handle
    }

    /// SAFETY: Must be called after a prior `start_parse` call, and while the input is still alive.
    pub(super) unsafe fn finish_parse(&mut self, handle: ParserHandle) {
        let popped_handle = self.parser_stack.pop();
        assert_eq!(popped_handle, Some(handle), "Popped handle does not match the provided handle");
        self.parsers.remove(handle);
    }

    pub(super) fn get(
        &mut self,
        handle: ParserHandle,
    ) -> Option<&mut ParseStack<'static, Output>> {
        self.parsers.get_mut(handle)
    }

    pub(super) fn current_stack(
        &mut self,
        span_source: &impl HasSpanRange,
    ) -> ExecutionResult<&mut ParseStack<'static, Output>> {
        match self.parser_stack.last() {
            Some(parser_handle) => Ok(self.parsers.get_mut(*parser_handle).unwrap()),
            None => {
                span_source.control_flow_err("There is no input stream available to read from.")
            }
        }
    }

    pub(super) fn current_input<'a>(
        &'a mut self,
        span_source: &impl HasSpanRange,
    ) -> ExecutionResult<ParseStream<'a, Output>> {
        let stack = self.current_stack(span_source)?;
        Ok(stack.current())
    }
}
