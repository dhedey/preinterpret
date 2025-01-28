use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct CommandArguments<'a> {
    parse_stream: ParseStream<'a>,
    command_name: Ident,
    /// The span of the [ ... ] which contained the command
    command_span: Span,
}

impl<'a> CommandArguments<'a> {
    pub(crate) fn new(
        parse_stream: ParseStream<'a>,
        command_name: Ident,
        command_span: Span,
    ) -> Self {
        Self {
            parse_stream,
            command_name,
            command_span,
        }
    }

    pub(crate) fn command_span(&self) -> Span {
        self.command_span
    }

    pub(crate) fn command_span_range(&self) -> SpanRange {
        self.command_span.span_range()
    }

    /// We use this instead of the "unexpected / drop glue" pattern in order to give a better error message
    pub(crate) fn assert_empty(&self, error_message: impl std::fmt::Display) -> ParseResult<()> {
        if self.parse_stream.is_empty() {
            Ok(())
        } else {
            self.command_span.parse_err(error_message)
        }
    }

    pub(crate) fn fully_parse_as<T: ArgumentsContent>(&self) -> ParseResult<T> {
        self.fully_parse_or_error(T::parse, T::error_message())
    }

    pub(crate) fn fully_parse_or_error<T>(
        &self,
        parse_function: impl FnOnce(ParseStream) -> ParseResult<T>,
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

    pub(crate) fn parse_all_for_interpretation(&self) -> ParseResult<InterpretationStream> {
        self.parse_stream
            .parse_all_for_interpretation(self.command_span.span_range())
    }

    pub(crate) fn read_all_as_raw_token_stream(&self) -> TokenStream {
        self.parse_stream.parse::<TokenStream>().unwrap()
    }
}

pub(crate) trait ArgumentsContent: Parse {
    fn error_message() -> String;
}
