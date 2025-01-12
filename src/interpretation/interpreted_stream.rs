use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) struct InterpretedStream {
    source_span_range: SpanRange,
    token_stream: TokenStream,
}

impl InterpretedStream {
    pub(crate) fn new(source_span_range: SpanRange) -> Self {
        Self {
            source_span_range,
            token_stream: TokenStream::new(),
        }
    }

    pub(crate) fn raw(empty_span_range: SpanRange, token_stream: TokenStream) -> Self {
        Self {
            source_span_range: empty_span_range,
            token_stream,
        }
    }

    pub(crate) fn extend(&mut self, interpreted_stream: InterpretedStream) {
        self.token_stream.extend(interpreted_stream.token_stream);
    }

    pub(crate) fn push_literal(&mut self, literal: Literal) {
        self.push_raw_token_tree(literal.into());
    }

    pub(crate) fn push_ident(&mut self, ident: Ident) {
        self.push_raw_token_tree(ident.into());
    }

    pub(crate) fn push_punct(&mut self, punct: Punct) {
        self.push_raw_token_tree(punct.into());
    }

    pub(crate) fn push_new_group(
        &mut self,
        inner_tokens: InterpretedStream,
        delimiter: Delimiter,
        span_range: SpanRange,
    ) {
        self.push_raw_token_tree(TokenTree::group(
            inner_tokens.token_stream,
            delimiter,
            span_range.span(),
        ));
    }

    fn push_raw_token_tree(&mut self, token_tree: TokenTree) {
        self.token_stream.extend(iter::once(token_tree));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.token_stream.is_empty()
    }

    pub(crate) fn parse_into_fields<T: 'static>(self, parser: FieldsParseDefinition<T>) -> Result<T> {
        let source_span_range = self.source_span_range;
        self.syn_parse(parser.create_syn_parser(source_span_range))
    }

    /// For a type `T` which implements `syn::Parse`, you can call this as `syn_parse(self, T::parse)`.
    /// For more complicated parsers, just pass the parsing function to this function.
    pub(crate) fn syn_parse<P: syn::parse::Parser>(self, parser: P) -> Result<P::Output> {
        parser.parse2(self.token_stream)
    }

    #[allow(unused)]
    pub(crate) fn into_singleton(self, error_message: &str) -> Result<TokenTree> {
        if self.is_empty() {
            return self.source_span_range.err(error_message);
        }
        let mut iter = self.token_stream.into_iter();
        let first = iter.next().unwrap(); // No panic because we're not empty
        match iter.next() {
            Some(_) => self.source_span_range.err(error_message),
            None => Ok(first),
        }
    }

    pub(crate) fn into_token_stream(self) -> TokenStream {
        self.token_stream
    }
}

impl From<TokenTree> for InterpretedStream {
    fn from(value: TokenTree) -> Self {
        InterpretedStream {
            source_span_range: value.span_range(),
            token_stream: value.into(),
        }
    }
}

impl HasSpanRange for InterpretedStream {
    fn span_range(&self) -> SpanRange {
        if self.token_stream.is_empty() {
            self.source_span_range
        } else {
            self.token_stream.span_range()
        }
    }
}

/// Parses a [..] block.
pub(crate) struct BracketedTokenStream {
    #[allow(unused)]
    pub(crate) brackets: syn::token::Bracket,
    pub(crate) token_stream: TokenStream,
}

impl syn::parse::Parse for BracketedTokenStream {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            brackets: syn::bracketed!(content in input),
            token_stream: content.parse()?,
        })
    }
}


pub(crate) struct FieldsParseDefinition<T> {
    new_builder: T,
    field_definitions: FieldDefinitions<T>,
}

impl<T: 'static> FieldsParseDefinition<T> {
    pub(crate) fn new(new_builder: T) -> Self {
        Self {
            new_builder,
            field_definitions: FieldDefinitions(BTreeMap::new()),
        }
    }

    pub(crate) fn add_required_field<F: syn::parse::Parse + 'static>(
        self,
        field_name: &str,
        example: &str,
        explanation: Option<&str>,
        set: impl Fn(&mut T, F) + 'static,
    ) -> Self {
        self.add_field(field_name, example, explanation, true, F::parse, set)
    }

    pub(crate) fn add_optional_field<F: syn::parse::Parse + 'static>(
        self,
        field_name: &str,
        example: &str,
        explanation: Option<&str>,
        set: impl Fn(&mut T, F) + 'static,
    ) -> Self {
        self.add_field(field_name, example, explanation, false, F::parse, set)
    }

    pub(crate) fn add_field<F>(
        mut self,
        field_name: &str,
        example: &str,
        explanation: Option<&str>,
        is_required: bool,
        parse: impl Fn(syn::parse::ParseStream) -> Result<F> + 'static,
        set: impl Fn(&mut T, F) + 'static,
    ) -> Self {
        if self.field_definitions.0.insert(
            field_name.to_string(),
            FieldParseDefinition {
                is_required,
                example: example.into(),
                explanation: explanation.map(|s| s.to_string()),
                parse_and_set: Box::new(move |builder, content| {
                    let value = parse(content)?;
                    set(builder, value);
                    Ok(())
                }),
            },
        ).is_some() {
            panic!("Duplicate field name: {field_name:?}");
        }
        self
    }

    pub(crate) fn create_syn_parser(self, error_span_range: SpanRange) -> impl FnOnce(syn::parse::ParseStream) -> Result<T> {
        fn inner<T>(
            input: syn::parse::ParseStream,
            new_builder: T,
            field_definitions: &FieldDefinitions<T>,
            error_span_range: SpanRange,
        ) -> Result<T> {
            let mut builder = new_builder;
            let content;
            let _ = syn::braced!(content in input);

            let mut required_field_names: BTreeSet<_> = field_definitions
                .0
                .iter()
                .filter_map(|(field_name, field_definition)| {
                    match field_definition.is_required {
                        true => Some(field_name.clone()),
                        false => None,
                    }
                })
                .collect();
            let mut seen_field_names = HashSet::new();

            while !content.is_empty() {
                let field_name = content.parse::<Ident>()?;
                let field_name_value = field_name.to_string();
                if !seen_field_names.insert(field_name_value.clone()) {
                    return field_name.err("Duplicate field name");
                }
                required_field_names.remove(field_name_value.as_str());
                let _ = content.parse::<syn::Token![:]>()?;
                let field_definition = field_definitions
                    .0
                    .get(field_name_value.as_str())
                    .ok_or_else(|| field_name.error(format!("Unsupported field name")))?;
                (field_definition.parse_and_set)(&mut builder, &content)?;
                if !content.is_empty() {
                    content.parse::<syn::Token![,]>()?;
                }
            }

            if !required_field_names.is_empty() {
                return error_span_range.err(format!(
                    "Missing required fields: {missing_fields:?}",
                    missing_fields = required_field_names,
                ));
            }

            Ok(builder)
        }
        move |input: syn::parse::ParseStream| {
            inner(input, self.new_builder, &self.field_definitions, error_span_range)
                .map_err(|error| {
                    // Sadly error combination is just buggy - the two outputted
                    // compile_error! invocations are back to back which causes a rustc
                    // parse error. Instead, let's do this.
                    error
                        .concat("\n")
                        .concat(&self.field_definitions.error_message())
                })
        }
    }
}


struct FieldDefinitions<T>(BTreeMap<String, FieldParseDefinition<T>>);

impl<T> FieldDefinitions<T> {
    fn error_message(&self) -> String{
        use std::fmt::Write;
        let mut message = "Expected: {\n".to_string();
        for (field_name, field_definition) in &self.0 {
            write!(
                &mut message,
                "    {}{}: {}",
                field_name,
                if field_definition.is_required { "" } else { "?" },
                field_definition.example,
            ).unwrap();
            match field_definition.explanation {
                Some(ref explanation) => write!(
                    &mut message,
                    ", // {explanation}\n",
                    explanation = explanation,
                ).unwrap(),
                None => write!(&mut message, ",\n").unwrap(),
            }
        }
        message.push('}');
        message
    }
}

struct FieldParseDefinition<T> {
    is_required: bool,
    example: String,
    explanation: Option<String>,
    parse_and_set: Box<dyn Fn(&mut T, syn::parse::ParseStream) -> Result<()>>,
}
