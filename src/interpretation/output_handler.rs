use super::*;

#[derive(Debug)]
pub(super) enum OutputHandlerError {
    FrozenOutputModification(MutationBlockReason),
}

pub(super) struct OutputHandler {
    output_stack: Vec<OutputStream>,
    freeze_stack_indices_at_or_below: Vec<(usize, MutationBlockReason)>,
}

impl OutputHandler {
    pub(super) fn new(initial_output: OutputStream) -> Self {
        Self {
            output_stack: vec![initial_output],
            freeze_stack_indices_at_or_below: vec![],
        }
    }

    pub(super) fn complete(self) -> OutputStream {
        let [final_output]: [OutputStream; 1] = self
            .output_stack
            .try_into()
            .map_err(|_| ())
            .expect("Output stack should have height one at completion");
        final_output
    }

    pub(super) fn current_output_mut(
        &mut self,
        span_source: &impl HasSpanRange,
    ) -> ExecutionResult<&mut OutputStream> {
        match self.current_output_mut_inner() {
            Ok(output) => Ok(output),
            Err(OutputHandlerError::FrozenOutputModification(reason)) => {
                span_source.control_flow_err(reason.error_message("emit"))
            }
        }
    }

    pub(super) fn current_output_mut_inner(
        &mut self,
    ) -> Result<&mut OutputStream, OutputHandlerError> {
        let index = self.index_of_last_output();

        self.validate_index(index)?;

        Ok(&mut self.output_stack[index])
    }

    /// SAFETY: Must be paired with a later `finish_inner_buffer_*` call, even in
    /// the face of control flow interrupts.
    pub(super) unsafe fn start_inner_buffer(&mut self) {
        self.output_stack.push(OutputStream::new());
    }

    /// SAFETY: Must be paired with a prior `start_inner_buffer` call.
    pub(super) unsafe fn finish_inner_buffer_as_group(&mut self, delimiter: Delimiter, span: Span) {
        let inner_buffer = self.finish_inner_buffer_as_separate_stream();
        self.current_output_mut_inner()
            .expect("Output stack should not be frozen if SAFETY conditions are met")
            .push_new_group(inner_buffer, delimiter, span);
    }

    /// SAFETY: Must be paired with a prior `start_inner_buffer` call.
    pub(super) unsafe fn finish_inner_buffer_as_separate_stream(&mut self) -> OutputStream {
        if self.output_stack.len() == 1 {
            panic!("Cannot pop the last output stream from the output stack");
        }

        self.output_stack
            .pop()
            .expect("Output stack should never be empty")
    }

    fn index_of_last_output(&self) -> usize {
        // OVERFLOW: Safe as we maintain the invariant that output_stack is never empty
        self.output_stack.len() - 1
    }

    /// SAFETY: Must be paired with unfreeze_existing.
    pub(super) unsafe fn freeze_existing(&mut self, reason: MutationBlockReason) {
        self.freeze_stack_indices_at_or_below
            .push((self.index_of_last_output(), reason));
    }

    /// SAFETY: Must be paired with freeze_existing.
    pub(super) unsafe fn unfreeze_existing(&mut self) {
        let (popped, _reason) = self.freeze_stack_indices_at_or_below.pop().unwrap();
        assert_eq!(
            popped, self.index_of_last_output(),
            "Any additional output streams added during the freeze must be removed before unfreezing"
        );
    }

    pub(super) fn output_stack_height(&self) -> usize {
        self.output_stack.len()
    }

    pub(super) fn current_output_unchecked(&self) -> &OutputStream {
        self.output_stack
            .last()
            .expect("Output stack should never be empty")
    }

    pub(super) fn current_output_mut_unchecked(&mut self) -> &mut OutputStream {
        self.output_stack
            .last_mut()
            .expect("Output stack should never be empty")
    }

    fn validate_index(&self, index: usize) -> Result<(), OutputHandlerError> {
        if let Some(&(freeze_at_or_below_depth, reason)) =
            self.freeze_stack_indices_at_or_below.last()
        {
            if index <= freeze_at_or_below_depth {
                return Err(OutputHandlerError::FrozenOutputModification(reason));
            }
        }
        Ok(())
    }
}
