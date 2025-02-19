use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionObject {
    pub(crate) entries: BTreeMap<String, ExpressionValue>,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(crate) span_range: SpanRange,
}

impl ExpressionObject {
    pub(super) fn handle_unary_operation(
        self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        Ok(match operation.operation {
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => {
                return operation.unsupported(self)
            }
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::DebugString => {
                    operation.output(self.entries).into_debug_string_value()?
                }
                CastTarget::String
                | CastTarget::Stream
                | CastTarget::Group
                | CastTarget::Boolean
                | CastTarget::Char
                | CastTarget::Integer(_)
                | CastTarget::Float(_) => {
                    return operation.unsupported(self);
                }
            },
        })
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        _rhs: Self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn into_indexed(
        self,
        access: IndexAccess,
        index: ExpressionValue,
    ) -> ExecutionResult<ExpressionValue> {
        let span_range = SpanRange::new_between(self.span_range, access.span_range());
        let key = index.expect_string("An object key")?.value;
        Ok(self.into_entry_or_none(&key, span_range))
    }

    pub(super) fn into_property(self, access: PropertyAccess) -> ExecutionResult<ExpressionValue> {
        let span_range = SpanRange::new_between(self.span_range, access.span_range());
        let key = access.property.to_string();
        Ok(self.into_entry_or_none(&key, span_range))
    }

    fn into_entry_or_none(mut self, key: &str, span_range: SpanRange) -> ExpressionValue {
        match self.entries.remove(key) {
            Some(value) => value.with_span_range(span_range),
            None => ExpressionValue::None(span_range),
        }
    }

    pub(super) fn index_mut(
        &mut self,
        _access: IndexAccess,
        index: ExpressionValue,
    ) -> ExecutionResult<&mut ExpressionValue> {
        let key = index.expect_string("An object key")?.value;
        Ok(self.mut_entry_or_create(key))
    }

    pub(super) fn property_mut(
        &mut self,
        access: PropertyAccess,
    ) -> ExecutionResult<&mut ExpressionValue> {
        Ok(self.mut_entry_or_create(access.property.to_string()))
    }

    fn mut_entry_or_create(&mut self, key: String) -> &mut ExpressionValue {
        use std::collections::btree_map::*;
        match self.entries.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(ExpressionValue::None(self.span_range)),
        }
    }

    pub(crate) fn concat_recursive_into(
        self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        if behaviour.output_array_structure {
            if self.entries.is_empty() {
                output.push_str("{}");
                return Ok(());
            }
            output.push('{');
            if behaviour.add_space_between_token_trees {
                output.push(' ');
            }
        }
        let mut is_first = true;
        for (key, value) in self.entries {
            if !is_first && behaviour.output_array_structure {
                output.push(',');
            }
            if !is_first && behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            if syn::parse_str::<Ident>(&key).is_ok() {
                output.push_str(&key);
            } else {
                output.push('[');
                output.push_str(format!("{:?}", key).as_str());
                output.push(']');
            }
            output.push(':');
            if behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            value.concat_recursive_into(output, behaviour)?;
            is_first = false;
        }
        if behaviour.output_array_structure {
            if behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            output.push('}');
        }
        Ok(())
    }
}

impl HasSpanRange for ExpressionObject {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl HasValueType for ExpressionObject {
    fn value_type(&self) -> &'static str {
        self.entries.value_type()
    }
}

impl HasValueType for BTreeMap<String, ExpressionValue> {
    fn value_type(&self) -> &'static str {
        "object"
    }
}

impl ToExpressionValue for BTreeMap<String, ExpressionValue> {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Object(ExpressionObject {
            entries: self,
            span_range,
        })
    }
}
