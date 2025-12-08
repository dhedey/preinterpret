use super::*;

#[derive(Clone)]
pub(crate) struct ArrayValue {
    pub(crate) items: Vec<Value>,
}

impl ArrayValue {
    pub(crate) fn new(items: Vec<Value>) -> Self {
        Self { items }
    }

    pub(crate) fn output_items_to(
        &self,
        output: &mut ToStreamContext,
        grouping: Grouping,
    ) -> ExecutionResult<()> {
        for item in &self.items {
            item.output_to(grouping, output)?;
        }
        Ok(())
    }

    pub(super) fn into_indexed(mut self, index: Spanned<&Value>) -> ExecutionResult<Value> {
        let span_range = index.span_range();
        Ok(match &*index {
            Value::Integer(integer) => {
                let idx =
                    self.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                std::mem::replace(&mut self.items[idx], Value::None)
            }
            Value::Range(range) => {
                let range = Spanned(range, span_range).resolve_to_index_range(&self)?;
                let new_items: Vec<_> = self.items.drain(range).collect();
                new_items.into_value()
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn index_mut(&mut self, index: Spanned<&Value>) -> ExecutionResult<&mut Value> {
        let span_range = index.span_range();
        Ok(match &*index {
            Value::Integer(integer) => {
                let idx =
                    self.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                &mut self.items[idx]
            }
            Value::Range(..) => {
                // Temporary until we add slice types - we error here
                return span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]");
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn index_ref(&self, index: Spanned<&Value>) -> ExecutionResult<&Value> {
        let span_range = index.span_range();
        Ok(match &*index {
            Value::Integer(integer) => {
                let idx =
                    self.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                &self.items[idx]
            }
            Value::Range(..) => {
                // Temporary until we add slice types - we error here
                return span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]");
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn resolve_valid_index(
        &self,
        index: Spanned<&Value>,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        let span_range = index.span_range();
        match &*index {
            Value::Integer(int) => {
                self.resolve_valid_index_from_integer(Spanned(int, span_range), is_exclusive)
            }
            _ => span_range.type_err("The index must be an integer"),
        }
    }

    fn resolve_valid_index_from_integer(
        &self,
        integer: Spanned<&IntegerValue>,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        let index: usize = (**integer)
            .into_owned_value()
            .resolve_as("An array index")?;
        if is_exclusive {
            if index <= self.items.len() {
                Ok(index)
            } else {
                integer.value_err(format!(
                    "Exclusive index of {} must be less than or equal to the array length of {}",
                    index,
                    self.items.len()
                ))
            }
        } else if index < self.items.len() {
            Ok(index)
        } else {
            integer.value_err(format!(
                "Inclusive index of {} must be less than the array length of {}",
                index,
                self.items.len()
            ))
        }
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        IteratorValue::any_iterator_to_string(
            self.items.iter(),
            output,
            behaviour,
            "[]",
            "[",
            "]",
            false, // Output all the vec because it's already in memory
        )
    }
}

impl HasValueKind for ArrayValue {
    type SpecificKind = ValueKind;

    fn kind(&self) -> ValueKind {
        ValueKind::Array
    }
}

impl ValuesEqual for ArrayValue {
    /// Recursively compares two arrays element-by-element.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.items.len() != other.items.len() {
            return ctx.lengths_unequal(Some(self.items.len()), Some(other.items.len()));
        }
        for (i, (l, r)) in self.items.iter().zip(other.items.iter()).enumerate() {
            let result = ctx.with_array_index(i, |ctx| l.test_equality(r, ctx));
            if ctx.should_short_circuit(&result) {
                return result;
            }
        }
        ctx.values_equal()
    }
}

impl IntoValue for Vec<Value> {
    fn into_value(self) -> Value {
        Value::Array(ArrayValue { items: self })
    }
}

impl IntoValue for ArrayValue {
    fn into_value(self) -> Value {
        Value::Array(self)
    }
}

impl_resolvable_argument_for! {
    ArrayTypeData,
    (value, context) -> ArrayValue {
        match value {
            Value::Array(value) => Ok(value),
            _ => context.err("an array", value),
        }
    }
}

define_interface! {
    struct ArrayTypeData,
    parent: IterableTypeData,
    pub(crate) mod array_interface {
        pub(crate) mod methods {
            fn push(mut this: Mutable<ArrayValue>, item: OwnedValue) -> ExecutionResult<()> {
                this.items.push(item.into());
                Ok(())
            }

            [context] fn to_stream_grouped(this: ArrayValue) -> StreamOutput<impl StreamAppender> [ignore_type_assertion!] {
                let error_span_range = context.span_range();
                StreamOutput::new(move |stream| this.output_items_to(&mut ToStreamContext::new(stream, error_span_range), Grouping::Grouped))
            }
        }
        pub(crate) mod unary_operations {
            [context] fn cast_to_numeric(this: Owned<ArrayValue>) -> ExecutionResult<ReturnedValue> {
                let mut this = this.into_inner();
                let length = this.items.len();
                if length == 1 {
                    context.operation.evaluate(this.items.pop().unwrap().into_owned())
                } else {
                    context.operation.value_err(format!(
                        "Only a singleton array can be cast to this value but the array has {} elements",
                        length,
                    ))
                }
            }
        }
        pub(crate) mod binary_operations {
            fn add(mut lhs: ArrayValue, rhs: ArrayValue) -> ArrayValue {
                lhs.items.extend(rhs.items);
                lhs
            }

            fn add_assign(mut lhs: Assignee<ArrayValue>, rhs: ArrayValue) {
                lhs.items.extend(rhs.items);
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => return None,
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::Boolean
                        | CastTarget::Char
                        | CastTarget::Integer(_)
                        | CastTarget::Float(_) => unary_definitions::cast_to_numeric(),
                        _ => return None,
                    },
                })
            }

            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    BinaryOperation::Addition { .. } => binary_definitions::add(),
                    BinaryOperation::AddAssign { .. } => binary_definitions::add_assign(),
                    _ => return None,
                })
            }
        }
    }
}
