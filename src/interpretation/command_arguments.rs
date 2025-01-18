use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct CommandArguments<'a> {
    parse_stream: ParseStream<'a>,
    command_name: Ident,
    /// The span range of the original stream, before tokens were consumed
    full_span_range: SpanRange,
    /// The span of the last item consumed (or the full span range if no items have been consumed yet)
    latest_item_span_range: SpanRange,
}

impl<'a> CommandArguments<'a> {
    pub(crate) fn new(
        parse_stream: ParseStream<'a>,
        command_name: Ident,
        span_range: SpanRange,
    ) -> Self {
        Self {
            parse_stream,
            command_name,
            full_span_range: span_range,
            latest_item_span_range: span_range,
        }
    }

    pub(crate) fn full_span_range(&self) -> SpanRange {
        self.full_span_range
    }

    /// We use this instead of the "unexpected / drop glue" pattern in order to give a better error message
    pub(crate) fn assert_empty(&self, error_message: impl std::fmt::Display) -> Result<()> {
        if self.parse_stream.is_empty() {
            Ok(())
        } else {
            self.latest_item_span_range.err(error_message)
        }
    }

    pub(crate) fn fully_parse_as<T: ArgumentsContent>(&self) -> Result<T> {
        self.fully_parse_or_error(T::parse, T::error_message())
    }

    pub(crate) fn fully_parse_or_error<T>(
        &self,
        parse_function: impl FnOnce(ParseStream) -> Result<T>,
        error_message: impl std::fmt::Display,
    ) -> Result<T> {
        let parsed = parse_function(self.parse_stream).or_else(|error| {
            // In future, when the diagnostic API is stable,
            // we can add this context directly onto the command ident...
            // Rather than just selectively adding it to the inner-most error.
            let error_string = error.to_string();

            // We avoid adding this additional context if it's already been added in an
            // inner error, because that's likely the correct error to show.
            if error_string.contains("\nOccurred whilst parsing") {
                return Err(error);
            }
            error.span().err(format!(
                "{}\nOccurred whilst parsing [!{}! ..] - {}",
                error_string, self.command_name, error_message,
            ))
        })?;

        self.assert_empty(error_message)?;

        Ok(parsed)
    }

    pub(crate) fn parse_all_for_interpretation(&self) -> Result<InterpretationStream> {
        self.parse_stream
            .parse_all_for_interpretation(self.full_span_range)
    }

    pub(crate) fn read_all_as_raw_token_stream(&self) -> TokenStream {
        self.parse_stream.parse::<TokenStream>().unwrap()
    }
}

pub(crate) trait ArgumentsContent: Parse {
    fn error_message() -> String;
}
