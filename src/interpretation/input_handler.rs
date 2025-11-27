use super::*;

pub(crate) struct InputHandler {
    input_stack: Vec<ParseStack<'static, Output>>,
}

impl InputHandler {
    pub(crate) fn new() -> Self {
        Self {
            input_stack: vec![],
        }
    }

    /// SAFETY:
    /// * Must be paired with a later `finish_parse` call, even in the face of control flow interrupts.
    /// * `finish_parse` must be called whilst the input is still alive.
    ///
    /// TODO: Replace this with returning a ParseGuard which captures the lifetime and handles calling
    /// `finish_parse` automatically when dropped, to avoid misuse.
    pub(super) unsafe fn start_parse(&mut self, input: ParseStream<Output>) {
        let parse_stack = ParseStack::new(input);
        self.input_stack.push(std::mem::transmute::<
            ParseStack<'_, Output>,
            ParseStack<'static, Output>,
        >(parse_stack));
    }

    /// SAFETY: Must be called after a prior `start_parse` call, and while the input is still alive.
    pub(super) unsafe fn finish_parse(&mut self) {
        self.input_stack.pop();
    }

    pub(super) fn current_stack(
        &mut self,
        span_source: &impl HasSpanRange,
    ) -> ExecutionResult<&mut ParseStack<'static, Output>> {
        match self.input_stack.last_mut() {
            Some(parse_stack) => Ok(parse_stack),
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
