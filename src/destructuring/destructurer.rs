use crate::internal_prelude::*;

pub(crate) trait DestructurerDefinition: Clone {
    const DESTRUCTURER_NAME: &'static str;
    fn parse(arguments: DestructurerArguments) -> ParseResult<Self>;
    fn handle_destructure(
        &self,
        input: ParseStream,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()>;
}

#[derive(Clone)]
pub(crate) struct DestructurerArguments<'a> {
    parse_stream: ParseStream<'a>,
    destructurer_name: Ident,
    full_span_range: SpanRange,
}

#[allow(unused)]
impl<'a> DestructurerArguments<'a> {
    pub(crate) fn new(
        parse_stream: ParseStream<'a>,
        destructurer_name: Ident,
        span_range: SpanRange,
    ) -> Self {
        Self {
            parse_stream,
            destructurer_name,
            full_span_range: span_range,
        }
    }

    pub(crate) fn full_span_range(&self) -> SpanRange {
        self.full_span_range
    }

    /// We use this instead of the "unexpected / drop glue" pattern in order to give a better error message
    pub(crate) fn assert_empty(&self, error_message: impl std::fmt::Display) -> ParseResult<()> {
        if self.parse_stream.is_empty() {
            Ok(())
        } else {
            self.full_span_range.parse_err(error_message)
        }
    }

    pub(crate) fn fully_parse_no_error_override<T: Parse>(&self) -> ParseResult<T> {
        self.parse_stream.parse()
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
                    "Occurred whilst parsing (!{}! ...) - {}",
                    self.destructurer_name, error_message,
                )
            })?;

        self.assert_empty(error_message)?;

        Ok(parsed)
    }
}

#[derive(Clone)]
pub(crate) struct Destructurer {
    instance: NamedDestructurer,
    #[allow(unused)]
    source_group_span: DelimSpan,
}

impl Parse for Destructurer {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (delim_span, content) = input.parse_specific_group(Delimiter::Parenthesis)?;
        content.parse::<Token![!]>()?;
        let destructurer_name = content.parse_any_ident()?;
        let destructurer_kind = match DestructurerKind::for_ident(&destructurer_name) {
            Some(destructurer_kind) => destructurer_kind,
            None => destructurer_name.span().err(
                format!(
                    "Expected `(!<name>! ...)`, for <name> one of: {}.\nIf this wasn't intended to be a named destructuring, you can work around this with (!raw! (!{} ... ))",
                    DestructurerKind::list_all(),
                    destructurer_name,
                ),
            )?,
        };
        content.parse::<Token![!]>()?;
        let instance = destructurer_kind.parse_instance(DestructurerArguments::new(
            &content,
            destructurer_name,
            delim_span.join().span_range(),
        ))?;
        Ok(Self {
            instance,
            source_group_span: delim_span,
        })
    }
}

impl HandleDestructure for Destructurer {
    fn handle_destructure(
        &self,
        input: ParseStream,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        self.instance.handle_destructure(input, interpreter)
    }
}

macro_rules! define_destructurers {
    (
        $(
            $destructurer:ident,
        )*
    ) => {
        #[allow(clippy::enum_variant_names)]
        #[derive(Clone, Copy)]
        pub(crate) enum DestructurerKind {
            $(
                $destructurer,
            )*
        }

        impl DestructurerKind {
            fn parse_instance(&self, arguments: DestructurerArguments) -> ParseResult<NamedDestructurer> {
                Ok(match self {
                    $(
                        Self::$destructurer => NamedDestructurer::$destructurer(
                            $destructurer::parse(arguments)?
                        ),
                    )*
                })
            }

            pub(crate) fn for_ident(ident: &Ident) -> Option<Self> {
                Some(match ident.to_string().as_ref() {
                    $(
                        $destructurer::DESTRUCTURER_NAME => Self::$destructurer,
                    )*
                    _ => return None,
                })
            }

            const ALL_KIND_NAMES: &'static [&'static str] = &[$($destructurer::DESTRUCTURER_NAME,)*];

            pub(crate) fn list_all() -> String {
                // TODO improve to add an "and" at the end
                Self::ALL_KIND_NAMES.join(", ")
            }
        }

        #[derive(Clone)]
        #[allow(clippy::enum_variant_names)]
        pub(crate) enum NamedDestructurer {
            $(
                $destructurer($destructurer),
            )*
        }

        impl NamedDestructurer {
            fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> ExecutionResult<()> {
                match self {
                    $(
                        Self::$destructurer(destructurer) => destructurer.handle_destructure(input, interpreter),
                    )*
                }
            }
        }
    };
}

define_destructurers! {
    StreamDestructurer,
    IdentDestructurer,
    LiteralDestructurer,
    PunctDestructurer,
    GroupDestructurer,
    RawDestructurer,
    ContentDestructurer,
}
