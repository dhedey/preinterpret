use super::*;

#[derive(Clone)]
pub(crate) struct ObjectExpression {
    pub(crate) entries: BTreeMap<String, ObjectEntry>,
}

impl ToExpressionValue for ObjectExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Object(self)
    }
}

impl_resolvable_argument_for! {
    ObjectTypeData,
    (value, context) -> ObjectExpression {
        match value {
            ExpressionValue::Object(value) => Ok(value),
            _ => context.err("object", value),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ObjectEntry {
    #[allow(unused)]
    pub(crate) key_span: Span,
    pub(crate) value: ExpressionValue,
}

impl ObjectExpression {
    pub(super) fn into_indexed(
        mut self,
        index: Spanned<&ExpressionValue>,
    ) -> ExecutionResult<ExpressionValue> {
        let key = index.resolve_as("An object key")?;
        Ok(self.remove_or_none(key))
    }

    pub(super) fn into_property(
        mut self,
        access: &PropertyAccess,
    ) -> ExecutionResult<ExpressionValue> {
        let key = access.property.to_string();
        Ok(self.remove_or_none(&key))
    }

    pub(crate) fn remove_or_none(&mut self, key: &str) -> ExpressionValue {
        match self.entries.remove(key) {
            Some(entry) => entry.value,
            None => ExpressionValue::None,
        }
    }

    pub(crate) fn remove_no_none(&mut self, key: &str) -> Option<ExpressionValue> {
        match self.entries.remove(key) {
            Some(entry) => {
                if entry.value.is_none() {
                    None
                } else {
                    Some(entry.value)
                }
            }
            None => None,
        }
    }

    pub(super) fn index_mut(
        &mut self,
        index: Spanned<&ExpressionValue>,
        auto_create: bool,
    ) -> ExecutionResult<&mut ExpressionValue> {
        let index: Spanned<&str> = index.resolve_as("An object key")?;
        self.mut_entry(index.map(|s, _| s.to_string()), auto_create)
    }

    pub(super) fn index_ref(
        &self,
        index: Spanned<&ExpressionValue>,
    ) -> ExecutionResult<&ExpressionValue> {
        let key: Spanned<&str> = index.resolve_as("An object key")?;
        let entry = self.entries.get(key.value).ok_or_else(|| {
            key.value_error(format!(
                "The object does not have a field named `{}`",
                key.value
            ))
        })?;
        Ok(&entry.value)
    }

    pub(super) fn property_mut(
        &mut self,
        access: &PropertyAccess,
        auto_create: bool,
    ) -> ExecutionResult<&mut ExpressionValue> {
        self.mut_entry(
            access.property.to_string().spanned(access.property.span()),
            auto_create,
        )
    }

    pub(super) fn property_ref(
        &self,
        access: &PropertyAccess,
    ) -> ExecutionResult<&ExpressionValue> {
        let key = access.property.to_string();
        let entry = self.entries.get(&key).ok_or_else(|| {
            access.value_error(format!("The object does not have a field named `{}`", key))
        })?;
        Ok(&entry.value)
    }

    fn mut_entry(
        &mut self,
        key: Spanned<String>,
        auto_create: bool,
    ) -> ExecutionResult<&mut ExpressionValue> {
        use std::collections::btree_map::*;
        let (key, key_span) = key.deconstruct();
        Ok(match self.entries.entry(key) {
            Entry::Occupied(entry) => &mut entry.into_mut().value,
            Entry::Vacant(entry) => {
                if auto_create {
                    &mut entry
                        .insert(ObjectEntry {
                            key_span: key_span.join_into_span_else_start(),
                            value: ExpressionValue::None,
                        })
                        .value
                } else {
                    return key_span
                        .value_err(format!("No property found for key `{}`", entry.into_key()));
                }
            }
        })
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        if !behaviour.use_debug_literal_syntax {
            return behaviour
                .error_span_range
                .value_err("An object can't be converted to a non-debug string");
        }
        if behaviour.output_literal_structure {
            if self.entries.is_empty() {
                output.push_str("%{}");
                return Ok(());
            }
            output.push_str("%{");
            if behaviour.add_space_between_token_trees {
                output.push(' ');
            }
        }
        let mut is_first = true;
        for (key, entry) in self.entries.iter() {
            if !is_first && behaviour.output_literal_structure {
                output.push(',');
            }
            if !is_first && behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            if syn::parse_str::<Ident>(key).is_ok() {
                output.push_str(key);
            } else {
                output.push('[');
                output.push_str(format!("{:?}", key).as_str());
                output.push(']');
            }
            output.push(':');
            if behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            entry.value.concat_recursive_into(output, behaviour)?;
            is_first = false;
        }
        if behaviour.output_literal_structure {
            if behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            output.push('}');
        }
        Ok(())
    }
}

impl Spanned<&ObjectExpression> {
    pub(crate) fn validate(&self, validation: &impl ObjectValidate) -> ExecutionResult<()> {
        let mut missing_fields = Vec::new();
        for (field_name, _) in validation.required_fields() {
            match self.entries.get(field_name) {
                None
                | Some(ObjectEntry {
                    value: ExpressionValue::None,
                    ..
                }) => {
                    missing_fields.push(field_name);
                }
                _ => {}
            }
        }
        let mut unexpected_fields = Vec::new();
        if !validation.allow_other_fields() {
            let allowed_fields = validation.all_fields_set();
            for field_name in self.entries.keys() {
                if !allowed_fields.contains(field_name) {
                    unexpected_fields.push(field_name.as_str());
                }
            }
        }
        if missing_fields.is_empty() && unexpected_fields.is_empty() {
            return Ok(());
        }
        let mut error_message = String::new();
        error_message.push_str("Expected:\n");
        error_message.push_str(&validation.describe_object());

        if !missing_fields.is_empty() {
            error_message.push('\n');
            error_message.push_str("The following required field/s are missing: ");
            error_message.push_str(&missing_fields.join(", "));
        }

        if !unexpected_fields.is_empty() {
            error_message.push('\n');
            error_message.push_str("The following field/s are unexpected: ");
            error_message.push_str(&unexpected_fields.join(", "));
        }

        self.value_err(error_message)
    }
}

impl HasValueType for ObjectExpression {
    fn value_type(&self) -> &'static str {
        self.entries.value_type()
    }
}

impl HasValueType for BTreeMap<String, ObjectEntry> {
    fn value_type(&self) -> &'static str {
        "object"
    }
}

impl ToExpressionValue for BTreeMap<String, ObjectEntry> {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Object(ObjectExpression { entries: self })
    }
}

define_interface! {
    struct ObjectTypeData,
    parent: IterableTypeData,
    pub(crate) mod object_interface {
        pub(crate) mod methods {
            [context] fn zip(this: ObjectExpression) -> ExecutionResult<ArrayExpression> {
                ZipIterators::new_from_object(this, context.span_range())?.run_zip(context.interpreter, true)
            }

            [context] fn zip_truncated(this: ObjectExpression) -> ExecutionResult<ArrayExpression> {
                ZipIterators::new_from_object(this, context.span_range())?.run_zip(context.interpreter, false)
            }
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {}
        interface_items {
        }
    }
}

#[allow(unused)] // TODO[unused-clearup]
pub(crate) struct ObjectValidation {
    // Should ideally be an indexmap
    fields: Vec<(String, FieldDefinition)>,
    allow_other_fields: bool,
}

pub(crate) trait ObjectValidate {
    fn all_fields(&self) -> Vec<(&str, &FieldDefinition)>;
    fn allow_other_fields(&self) -> bool;

    fn required_fields(&self) -> Vec<(&str, &FieldDefinition)> {
        self.all_fields()
            .into_iter()
            .filter_map(|(key, field_definition)| {
                if field_definition.required {
                    Some((key, field_definition))
                } else {
                    None
                }
            })
            .collect()
    }

    fn all_fields_set(&self) -> HashSet<String> {
        self.all_fields()
            .into_iter()
            .map(|(key, _)| key.to_string())
            .collect()
    }

    fn describe_object(&self) -> String {
        use std::fmt::Write;
        let mut buffer = String::new();
        buffer.write_str("%{\n").unwrap();
        for (key, definition) in self.all_fields() {
            if let Some(description) = &definition.description {
                writeln!(buffer, "    // {}", description).unwrap();
            }
            if definition.required {
                writeln!(buffer, "    {}: {},", key, definition.example).unwrap();
            } else {
                writeln!(buffer, "    {}?: {},", key, definition.example).unwrap();
            }
        }
        buffer.write_str("}").unwrap();
        buffer
    }
}

impl ObjectValidate for ObjectValidation {
    fn all_fields(&self) -> Vec<(&str, &FieldDefinition)> {
        self.fields
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect()
    }

    fn allow_other_fields(&self) -> bool {
        self.allow_other_fields
    }
}

impl ObjectValidate for &[(&'static str, FieldDefinition)] {
    fn all_fields(&self) -> Vec<(&str, &FieldDefinition)> {
        self.iter().map(|(k, v)| (*k, v)).collect()
    }

    fn allow_other_fields(&self) -> bool {
        false
    }
}

pub(crate) struct FieldDefinition {
    pub(crate) required: bool,
    pub(crate) description: Option<Cow<'static, str>>,
    pub(crate) example: Cow<'static, str>,
}

impl FieldDefinition {
    pub(crate) const fn new_full_static(
        required: bool,
        description: &'static str,
        example: &'static str,
    ) -> Self {
        Self {
            required,
            description: Some(Cow::Borrowed(description)),
            example: Cow::Borrowed(example),
        }
    }
}
