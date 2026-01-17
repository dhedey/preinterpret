use super::*;

define_leaf_type! {
    pub(crate) ArrayType => AnyType(AnyValueContent::Array),
    content: ArrayValue,
    kind: pub(crate) ArrayKind,
    type_name: "array",
    articled_display_name: "an array",
    dyn_impls: {
        IterableType: impl IsIterable {
            fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue> {
                Ok(IteratorValue::new_for_array(*self))
            }

            fn len(&self, _error_span_range: SpanRange) -> ExecutionResult<usize> {
                Ok(self.items.len())
            }
        }
    },
}

#[derive(Clone)]
pub(crate) struct ArrayValue {
    pub(crate) items: Vec<AnyValue>,
}

impl ArrayValue {
    pub(crate) fn new(items: Vec<AnyValue>) -> Self {
        Self { items }
    }

    pub(crate) fn output_items_to(
        &self,
        output: &mut ToStreamContext,
        grouping: Grouping,
    ) -> ExecutionResult<()> {
        for item in &self.items {
            item.as_ref_value().output_to(grouping, output)?;
        }
        Ok(())
    }

    pub(super) fn into_indexed(
        mut self,
        Spanned(index, span_range): Spanned<AnyValueRef>,
    ) -> ExecutionResult<AnyValue> {
        Ok(match index {
            AnyValueContent::Integer(integer) => {
                let index =
                    self.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                std::mem::replace(&mut self.items[index], ().into_any_value())
            }
            AnyValueContent::Range(range) => {
                let range = Spanned(range, span_range).resolve_to_index_range(&self)?;
                let new_items: Vec<_> = self.items.drain(range).collect();
                new_items.into_any_value()
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn index_mut(
        &mut self,
        Spanned(index, span_range): Spanned<AnyValueRef>,
    ) -> ExecutionResult<&mut AnyValue> {
        Ok(match index {
            AnyValueContent::Integer(integer) => {
                let index =
                    self.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                &mut self.items[index]
            }
            AnyValueContent::Range(..) => {
                // Temporary until we add slice types - we error here
                return span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]");
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn index_ref(
        &self,
        Spanned(index, span_range): Spanned<AnyValueRef>,
    ) -> ExecutionResult<&AnyValue> {
        Ok(match index {
            AnyValueContent::Integer(integer) => {
                let index =
                    self.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                &self.items[index]
            }
            AnyValueContent::Range(..) => {
                // Temporary until we add slice types - we error here
                return span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]");
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn resolve_valid_index(
        &self,
        Spanned(index, span_range): Spanned<AnyValueRef>,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        match index {
            AnyValueContent::Integer(int) => {
                self.resolve_valid_index_from_integer(Spanned(int, span_range), is_exclusive)
            }
            _ => span_range.type_err("The index must be an integer"),
        }
    }

    fn resolve_valid_index_from_integer(
        &self,
        Spanned(integer, span): Spanned<IntegerValueRef>,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        let index: OptionalSuffix<usize> =
            Spanned(integer.clone_to_owned_infallible(), span).resolve_as("An array index")?;
        let index = index.0;
        if is_exclusive {
            if index <= self.items.len() {
                Ok(index)
            } else {
                span.value_err(format!(
                    "Exclusive index of {} must be less than or equal to the array length of {}",
                    index,
                    self.items.len()
                ))
            }
        } else if index < self.items.len() {
            Ok(index)
        } else {
            span.value_err(format!(
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

impl IntoAnyValue for Vec<AnyValue> {
    fn into_any_value(self) -> AnyValue {
        ArrayValue { items: self }.into_any_value()
    }
}

impl_resolvable_argument_for! {
    ArrayType,
    (value, context) -> ArrayValue {
        match value {
            AnyValueContent::Array(value) => Ok(value),
            _ => context.err("an array", value),
        }
    }
}

define_type_features! {
    impl ArrayType,
    pub(crate) mod array_interface {
        pub(crate) mod methods {
            fn push(mut this: Mutable<ArrayValue>, item: AnyValue) -> ExecutionResult<()> {
                this.items.push(item);
                Ok(())
            }

            [context] fn to_stream_grouped(this: ArrayValue) -> StreamOutput<impl StreamAppender> [ignore_type_assertion!] {
                let error_span_range = context.span_range();
                StreamOutput::new(move |stream| this.output_items_to(&mut ToStreamContext::new(stream, error_span_range), Grouping::Grouped))
            }
        }
        pub(crate) mod unary_operations {
            [context] fn cast_singleton_to_value(Spanned(mut this, span): Spanned<ArrayValue>) -> ExecutionResult<ReturnedValue> {
                let length = this.items.len();
                if length == 1 {
                    Ok(context.operation.evaluate(this.items.pop().unwrap().spanned(span))?.0)
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
        index_access(ArrayValue) {
            fn shared(source: &'a ArrayValue, index: Spanned<AnyValueRef>) {
                source.index_ref(index)
            }
            fn mutable(source: &'a mut ArrayValue, index: Spanned<AnyValueRef>, _auto_create: bool) {
                source.index_mut(index)
            }
            fn owned(source: ArrayValue, index: Spanned<AnyValueRef>) {
                source.into_indexed(index)
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => return None,
                    UnaryOperation::Cast { target, .. } => if target.is_singleton_target() {
                        unary_definitions::cast_singleton_to_value()
                    } else {
                        return None;
                    }
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
