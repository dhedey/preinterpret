use crate::internal_prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[allow(unused)]
pub(crate) struct FieldsParseDefinition<T> {
    new_builder: T,
    field_definitions: FieldDefinitions<T>,
}

#[allow(unused)]
impl<T: 'static> FieldsParseDefinition<T> {
    pub(crate) fn new(new_builder: T) -> Self {
        Self {
            new_builder,
            field_definitions: FieldDefinitions(BTreeMap::new()),
        }
    }

    pub(crate) fn add_required_field<F: ParseFromSource + 'static>(
        self,
        field_name: &str,
        example: &str,
        explanation: Option<&str>,
        set: impl Fn(&mut T, F) + 'static,
    ) -> Self {
        self.add_field(
            field_name,
            example,
            explanation,
            true,
            F::parse_from_source,
            set,
        )
    }

    pub(crate) fn add_optional_field<F: ParseFromSource + 'static>(
        self,
        field_name: &str,
        example: &str,
        explanation: Option<&str>,
        set: impl Fn(&mut T, F) + 'static,
    ) -> Self {
        self.add_field(
            field_name,
            example,
            explanation,
            false,
            F::parse_from_source,
            set,
        )
    }

    pub(crate) fn add_field<F>(
        mut self,
        field_name: &str,
        example: &str,
        explanation: Option<&str>,
        is_required: bool,
        parse: impl Fn(SourceParseStream) -> ParseResult<F> + 'static,
        set: impl Fn(&mut T, F) + 'static,
    ) -> Self {
        if self
            .field_definitions
            .0
            .insert(
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
            )
            .is_some()
        {
            panic!("Duplicate field name: {field_name:?}");
        }
        self
    }

    pub(crate) fn create_syn_parser(
        self,
        error_span_range: SpanRange,
    ) -> impl FnOnce(SynParseStream) -> ParseResult<T> {
        fn inner<T>(
            input: SourceParseStream,
            new_builder: T,
            field_definitions: &FieldDefinitions<T>,
            error_span_range: SpanRange,
        ) -> ParseResult<T> {
            let mut builder = new_builder;
            let (_, content) = input.parse_specific_group(Delimiter::Brace)?;

            let mut required_field_names: BTreeSet<_> = field_definitions
                .0
                .iter()
                .filter_map(
                    |(field_name, field_definition)| match field_definition.is_required {
                        true => Some(field_name.clone()),
                        false => None,
                    },
                )
                .collect();
            let mut seen_field_names = HashSet::new();

            while !content.is_empty() {
                let field_name = content.parse::<Ident>()?;
                let field_name_value = field_name.to_string();
                if !seen_field_names.insert(field_name_value.clone()) {
                    return field_name.parse_err("Duplicate field name");
                }
                required_field_names.remove(field_name_value.as_str());
                let _ = content.parse::<Token![:]>()?;
                let field_definition = field_definitions
                    .0
                    .get(field_name_value.as_str())
                    .ok_or_else(|| field_name.error("Unsupported field name".to_string()))?;
                (field_definition.parse_and_set)(&mut builder, &content)?;
                if !content.is_empty() {
                    content.parse::<Token![,]>()?;
                }
            }

            if !required_field_names.is_empty() {
                return error_span_range.parse_err(format!(
                    "Missing required fields: {missing_fields:?}",
                    missing_fields = required_field_names,
                ));
            }

            Ok(builder)
        }
        move |input: SynParseStream| {
            inner(
                input.into(),
                self.new_builder,
                &self.field_definitions,
                error_span_range,
            )
            .add_context_if_error_and_no_context(|| self.field_definitions.error_message())
        }
    }
}

struct FieldDefinitions<T>(BTreeMap<String, FieldParseDefinition<T>>);

impl<T> FieldDefinitions<T> {
    fn error_message(&self) -> String {
        use std::fmt::Write;
        let mut message = "Expected: {\n".to_string();
        for (field_name, field_definition) in &self.0 {
            write!(
                &mut message,
                "    {}{}: {}",
                field_name,
                if field_definition.is_required {
                    ""
                } else {
                    "?"
                },
                field_definition.example,
            )
            .unwrap();
            match field_definition.explanation {
                Some(ref explanation) => writeln!(
                    &mut message,
                    ", // {explanation}",
                    explanation = explanation,
                )
                .unwrap(),
                None => writeln!(&mut message, ",").unwrap(),
            }
        }
        message.push('}');
        message
    }
}

#[allow(unused)]
struct FieldParseDefinition<T> {
    is_required: bool,
    example: String,
    explanation: Option<String>,
    #[allow(clippy::type_complexity)]
    parse_and_set: Box<dyn Fn(&mut T, SourceParseStream) -> ParseResult<()>>,
}
