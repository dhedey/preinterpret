use crate::internal_prelude::*;

pub(crate) trait TransformerDefinition: Clone {
    const TRANSFORMER_NAME: &'static str;

    fn parse(arguments: TransformerArguments) -> ParseResult<Self>;

    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;
}

#[derive(Clone)]
pub(crate) struct TransformerArguments<'a> {
    parse_stream: ParseStream<'a, Source>,
    transformer_name: Ident,
    full_span: Span,
}

#[allow(unused)]
impl<'a> TransformerArguments<'a> {
    pub(crate) fn new(
        parse_stream: ParseStream<'a, Source>,
        transformer_name: Ident,
        full_span: Span,
    ) -> Self {
        Self {
            parse_stream,
            transformer_name,
            full_span,
        }
    }

    pub(crate) fn full_span(&self) -> Span {
        self.full_span
    }

    /// We use this instead of the "unexpected / drop glue" pattern in order to give a better error message
    pub(crate) fn assert_empty(&self, error_message: impl std::fmt::Display) -> ParseResult<()> {
        if self.parse_stream.is_empty() {
            Ok(())
        } else {
            self.full_span.parse_err(error_message)
        }
    }

    pub(crate) fn fully_parse_no_error_override<T: Parse<Source>>(&self) -> ParseResult<T> {
        self.parse_stream.parse()
    }

    pub(crate) fn fully_parse_as<T: ArgumentsContent>(&self) -> ParseResult<T> {
        self.fully_parse_or_error(T::parse, T::error_message())
    }

    pub(crate) fn fully_parse_or_error<T>(
        &self,
        parse_function: impl FnOnce(ParseStream<Source>) -> ParseResult<T>,
        error_message: impl std::fmt::Display,
    ) -> ParseResult<T> {
        // In future, when the diagnostic API is stable,
        // we can add this context directly onto the transformer ident...
        // Rather than just selectively adding it to the inner-most error.
        //
        // For now though, we can add additional context to the error message.
        // But we can avoid adding this additional context if it's already been added in an
        // inner error, because that's likely the correct local context to show.
        let parsed =
            parse_function(self.parse_stream).add_context_if_error_and_no_context(|| {
                format!(
                    "Occurred whilst parsing @[{} ...] - {}",
                    self.transformer_name, error_message,
                )
            })?;

        self.assert_empty(error_message)?;

        Ok(parsed)
    }
}

#[derive(Clone)]
pub(crate) struct Transformer {
    #[allow(unused)]
    transformer_token: Token![@],
    instance: NamedTransformer,
    #[allow(unused)]
    source_brackets: Option<Brackets>,
}

impl Parse<Source> for Transformer {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let transformer_token = input.parse()?;

        let (name, arguments) = if input.cursor().ident().is_some() {
            let ident = input.parse_any_ident()?;
            (ident, None)
        } else if input.peek_specific_group(Delimiter::Bracket) {
            let (brackets, content) = input.parse_brackets()?;
            let ident = content.parse_any_ident()?;
            (ident, Some((content, brackets)))
        } else {
            return input.parse_err("Expected @TRANSFORMER or @[TRANSFORMER ...arguments...]");
        };

        let transformer_kind = match TransformerKind::for_ident(&name) {
            Some(transformer_kind) => transformer_kind,
            None => name.span().err(
                // TODO: Check the EXACT guidance is still correct
                format!(
                    "Expected `@NAME` or `@[NAME ...arguments...]` for NAME one of: {}.\nIf this wasn't intended to be a named transformer, you can work around this by replacing the @ with @[EXACT(%raw[@])]",
                    TransformerKind::list_all(),
                ),
            )?,
        };

        match arguments {
            Some((buffer, brackets)) => {
                let arguments = TransformerArguments::new(&buffer, name, brackets.join());
                Ok(Self {
                    transformer_token,
                    instance: transformer_kind.parse_instance(arguments)?,
                    source_brackets: Some(brackets),
                })
            }
            None => {
                let span = name.span();
                let (instance, _) = TokenStream::new()
                    .full_source_parse_with(|parse_stream| {
                        let arguments = TransformerArguments::new(parse_stream, name.clone(), span);
                        transformer_kind.parse_instance(arguments)
                    })
                    .map_err(|original_error| {
                        let replacement = span.parse_error(
                            format!("This transformer requires arguments, and should be invoked as @[{} ...]", name),
                        );
                        if let Some(context) = original_error.context() {
                            replacement.add_context_if_none(context)
                        } else {
                            replacement
                        }
                    })?;
                Ok(Self {
                    transformer_token,
                    instance,
                    source_brackets: None,
                })
            }
        }
    }
}

impl HandleTransformation for Transformer {
    fn handle_transform(
        &self,
        input: ParseStream<Output>,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.instance.handle_transform(input, interpreter, output)
    }
}

macro_rules! define_transformers {
    (
        $(
            $transformer:ident,
        )*
    ) => {
        #[allow(clippy::enum_variant_names)]
        #[derive(Clone, Copy)]
        pub(crate) enum TransformerKind {
            $(
                $transformer,
            )*
        }

        impl TransformerKind {
            fn parse_instance(&self, arguments: TransformerArguments) -> ParseResult<NamedTransformer> {
                Ok(match self {
                    $(
                        Self::$transformer => NamedTransformer::$transformer(
                            $transformer::parse(arguments)?
                        ),
                    )*
                })
            }

            pub(crate) fn for_ident(ident: &Ident) -> Option<Self> {
                Some(match ident.to_string().as_ref() {
                    $(
                        $transformer::TRANSFORMER_NAME => Self::$transformer,
                    )*
                    _ => return None,
                })
            }

            const ALL_KIND_NAMES: &'static [&'static str] = &[$($transformer::TRANSFORMER_NAME,)*];

            pub(crate) fn list_all() -> String {
                // TODO: Add "and" at the end
                Self::ALL_KIND_NAMES.join(", ")
            }
        }

        #[derive(Clone)]
        #[allow(clippy::enum_variant_names)]
        pub(crate) enum NamedTransformer {
            $(
                $transformer($transformer),
            )*
        }

        impl NamedTransformer {
            fn handle_transform(&self, input: ParseStream<Output>, interpreter: &mut Interpreter, output: &mut OutputStream) -> ExecutionResult<()> {
                match self {
                    $(
                        Self::$transformer(transformer) => transformer.handle_transform(input, interpreter, output),
                    )*
                }
            }
        }
    };
}

define_transformers! {
    TokenTreeTransformer,
    UntilTransformer,
    RestTransformer,
    IdentTransformer,
    LiteralTransformer,
    PunctTransformer,
    GroupTransformer,
    ExactTransformer,
}
