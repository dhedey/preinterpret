use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct CommandArguments<'a> {
    parse_stream: ParseStream<'a, Source>,
    command_name: Ident,
}

impl<'a> CommandArguments<'a> {
    pub(crate) fn new(
        parse_stream: ParseStream<'a, Source>,
        command_name: Ident,
        _command_span: Span,
    ) -> Self {
        Self {
            parse_stream,
            command_name,
        }
    }

    /// We use this instead of the "unexpected / drop glue" pattern in order to give a better error message
    pub(crate) fn assert_empty(&self, error_message: impl std::fmt::Display) -> ParseResult<()> {
        if self.parse_stream.is_empty() {
            Ok(())
        } else {
            self.parse_stream
                .parse_err(format!("Unexpected extra tokens. {}", error_message))
        }
    }

    pub(crate) fn fully_parse_or_error<T>(
        &self,
        parse_function: impl FnOnce(ParseStream<Source>) -> ParseResult<T>,
        error_message: impl std::fmt::Display,
    ) -> ParseResult<T> {
        // In future, when the diagnostic API is stable,
        // we can add this context directly onto the command ident...
        // Rather than just selectively adding it to the inner-most error.
        //
        // For now though, we can add additional context to the error message.
        // But we can avoid adding this additional context if it's already been added in an
        // inner error, because that's likely the correct local context to show.
        let parsed =
            parse_function(self.parse_stream).add_context_if_error_and_no_context(|| {
                format!(
                    "Occurred whilst parsing [!{}! ...] - {}",
                    self.command_name, error_message,
                )
            })?;

        self.assert_empty(error_message)?;

        Ok(parsed)
    }
}

pub(crate) trait ArgumentsContent: Parse<Source> {
    fn error_message() -> String;
}
