use super::*;

define_leaf_type! {
    pub(crate) ObjectType => AnyType(AnyValueContent::Object),
    content: ObjectValue,
    kind: pub(crate) ObjectKind,
    type_name: "object",
    articled_value_name: "an object",
    dyn_impls: {
        IterableType: impl IsIterable {
            fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue> {
                Ok(IteratorValue::new_for_object(*self))
            }

            fn len(&self, _error_span_range: SpanRange) -> ExecutionResult<usize> {
                Ok(self.entries.len())
            }
        }
    },
}

#[derive(Clone)]
pub(crate) struct ObjectValue {
    pub(crate) entries: BTreeMap<String, ObjectEntry>,
}

impl_resolvable_argument_for! {
    ObjectType,
    (value, context) -> ObjectValue {
        match value {
            AnyValue::Object(value) => Ok(value),
            _ => context.err("an object", value),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ObjectEntry {
    #[allow(unused)]
    pub(crate) key_span: Span,
    pub(crate) value: AnyValue,
}

impl ObjectValue {
    pub(super) fn into_indexed(mut self, index: Spanned<AnyValueRef>) -> ExecutionResult<AnyValue> {
        let key = index.downcast_resolve("An object key")?;
        Ok(self.remove_or_none(key))
    }

    pub(super) fn into_property(mut self, access: &PropertyAccess) -> ExecutionResult<AnyValue> {
        let key = access.property.to_string();
        Ok(self.remove_or_none(&key))
    }

    pub(crate) fn remove_or_none(&mut self, key: &str) -> AnyValue {
        match self.entries.remove(key) {
            Some(entry) => entry.value,
            None => ().into_any_value(),
        }
    }

    pub(crate) fn remove_no_none(&mut self, key: &str) -> Option<AnyValue> {
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
        index: Spanned<AnyValueRef>,
        auto_create: bool,
    ) -> ExecutionResult<&mut AnyValue> {
        let index: Spanned<&str> = index.downcast_resolve("An object key")?;
        self.mut_entry(index.map(|s| s.to_string()), auto_create)
    }

    pub(super) fn index_ref(&self, index: Spanned<AnyValueRef>) -> ExecutionResult<&AnyValue> {
        let key: Spanned<&str> = index.downcast_resolve("An object key")?;
        match self.entries.get(*key) {
            Some(entry) => Ok(&entry.value),
            None => Ok(&AnyValue::None(())),
        }
    }

    pub(super) fn property_mut(
        &mut self,
        access: &PropertyAccess,
        auto_create: bool,
    ) -> ExecutionResult<&mut AnyValue> {
        self.mut_entry(
            access.property.to_string().spanned(access.property.span()),
            auto_create,
        )
    }

    pub(super) fn property_ref(&self, access: &PropertyAccess) -> ExecutionResult<&AnyValue> {
        let key = access.property.to_string();
        match self.entries.get(&key) {
            Some(entry) => Ok(&entry.value),
            None => Ok(&AnyValue::None(())),
        }
    }

    fn mut_entry(
        &mut self,
        Spanned(key, key_span): Spanned<String>,
        auto_create: bool,
    ) -> ExecutionResult<&mut AnyValue> {
        use std::collections::btree_map::*;
        Ok(match self.entries.entry(key) {
            Entry::Occupied(entry) => &mut entry.into_mut().value,
            Entry::Vacant(entry) => {
                if auto_create {
                    &mut entry
                        .insert(ObjectEntry {
                            key_span: key_span.join_into_span_else_start(),
                            value: ().into_any_value(),
                        })
                        .value
                } else {
                    return key_span.value_err(format!(
                        "There is no pre-existing entry with key `{}` available to mutate",
                        entry.into_key()
                    ));
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
            entry
                .value
                .as_ref_value()
                .concat_recursive_into(output, behaviour)?;
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

impl ValuesEqual for ObjectValue {
    /// Recursively compares two objects.
    /// Objects are equal if they have the same keys and all values are equal.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.entries.len() != other.entries.len() {
            return ctx.lengths_unequal(Some(self.entries.len()), Some(other.entries.len()));
        }
        for (key, lhs_entry) in self.entries.iter() {
            match other.entries.get(key) {
                Some(rhs_entry) => {
                    let result = ctx.with_object_key(key, |ctx| {
                        lhs_entry.value.test_equality(&rhs_entry.value, ctx)
                    });
                    if ctx.should_short_circuit(&result) {
                        return result;
                    }
                }
                None => return ctx.missing_key(key, MissingSide::Rhs),
            }
        }
        ctx.values_equal()
    }
}

impl Spanned<&ObjectValue> {
    pub(crate) fn validate(&self, validation: &impl ObjectValidate) -> ExecutionResult<()> {
        let mut missing_fields = Vec::new();
        for (field_name, _) in validation.required_fields() {
            match self.entries.get(field_name) {
                None
                | Some(ObjectEntry {
                    value: AnyValue::None(_),
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

impl IsValueContent for BTreeMap<String, ObjectEntry> {
    type Type = ObjectType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for BTreeMap<String, ObjectEntry> {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        ObjectValue { entries: self }
    }
}

define_type_features! {
    impl ObjectType,
    pub(crate) mod object_interface {
        methods {
            [context] fn zip(this: ObjectValue) -> ExecutionResult<ArrayValue> {
                ZipIterators::new_from_object(this, context.span_range())?.run_zip(context.interpreter, true)
            }

            [context] fn zip_truncated(this: ObjectValue) -> ExecutionResult<ArrayValue> {
                ZipIterators::new_from_object(this, context.span_range())?.run_zip(context.interpreter, false)
            }
        }
        property_access(ObjectValue) {
            [ctx] fn shared(source: &'a ObjectValue) {
                source.property_ref(ctx.property)
            }
            [ctx] fn mutable(source: &'a mut ObjectValue, auto_create: bool) {
                source.property_mut(ctx.property, auto_create)
            }
            [ctx] fn owned(source: ObjectValue) {
                source.into_property(ctx.property)
            }
        }
        index_access(ObjectValue) {
            fn shared(source: &'a ObjectValue, index: Spanned<AnyValueRef>) {
                source.index_ref(index)
            }
            fn mutable(source: &'a mut ObjectValue, index: Spanned<AnyValueRef>, auto_create: bool) {
                source.index_mut(index, auto_create)
            }
            fn owned(source: ObjectValue, index: Spanned<AnyValueRef>) {
                source.into_indexed(index)
            }
        }
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
