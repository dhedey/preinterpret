use super::*;

define_leaf_type! {
    pub(crate) ArrayType => AnyType(AnyValueContent::Array),
    content: ArrayValue,
    kind: pub(crate) ArrayKind,
    type_name: "array",
    articled_value_name: "an array",
    dyn_impls: {
        IterableType: impl IsIterable {
            fn into_iterator(self: Box<Self>) -> FunctionResult<IteratorValue> {
                Ok(IteratorValue::new_for_array(*self))
            }

            fn iterable_len(&self, _error_span_range: SpanRange) -> FunctionResult<usize> {
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
    ) -> FunctionResult<()> {
        for item in &self.items {
            item.as_ref_value().output_to(grouping, output)?;
        }
        Ok(())
    }

    pub(super) fn into_indexed(
        mut self,
        Spanned(index, span_range): Spanned<AnyValueRef>,
    ) -> FunctionResult<AnyValue> {
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

    pub(super) fn resolve_valid_index(
        &self,
        Spanned(index, span_range): Spanned<AnyValueRef>,
        is_exclusive: bool,
    ) -> FunctionResult<usize> {
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
    ) -> FunctionResult<usize> {
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
        interpreter: &mut Interpreter,
    ) -> FunctionResult<()> {
        any_items_to_string(
            &mut self.items.iter(),
            output,
            behaviour,
            "[]",
            "[",
            "]",
            false, // Output all the vec because it's already in memory
            interpreter,
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

impl IsValueContent for Vec<AnyValue> {
    type Type = ArrayType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for Vec<AnyValue> {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        ArrayValue { items: self }
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
        methods {
            fn push(mut this: Mutable<ArrayValue>, item: AnyValue) -> FunctionResult<()> {
                this.items.push(item);
                Ok(())
            }

            [context] fn to_stream_grouped(this: ArrayValue) -> FunctionResult<OutputStream> {
                let error_span_range = context.span_range();
                context.interpreter.capture_output(|output| {
                    this.output_items_to(&mut ToStreamContext::new(output, error_span_range), Grouping::Grouped)
                })
            }
        }
        unary_operations {
            [context] fn cast_singleton_to_value(Spanned(mut this, span): Spanned<ArrayValue>) -> FunctionResult<ReturnedValue> {
                let length = this.items.len();
                if length == 1 {
                    Ok(context.operation.evaluate(this.items.pop().unwrap().spanned(span), context.interpreter)?.0)
                } else {
                    context.operation.value_err(format!(
                        "Only a singleton array can be cast to this value but the array has {} elements",
                        length,
                    ))
                }
            }
        }
        binary_operations {
            fn add(mut lhs: ArrayValue, rhs: ArrayValue) -> ArrayValue {
                lhs.items.extend(rhs.items);
                lhs
            }

            fn add_assign(mut lhs: Assignee<ArrayValue>, rhs: ArrayValue) {
                lhs.items.extend(rhs.items);
            }
        }
        index_access(ArrayValue) {
            [ctx] fn shared(source: &'a ArrayValue, Spanned(index, span_range): Spanned<AnyValueRef>) {
                match index {
                    AnyValueContent::Integer(integer) => {
                        let idx = source.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                        // SAFETY: ArrayChild correctly describes navigating to an array element
                        Ok(unsafe {
                            MappedRef::new(
                                &source.items[idx],
                                PathExtension::Child(
                                    ChildSpecifier::ArrayChild(idx),
                                    AnyType::type_kind(),
                                ),
                                ctx.output_span_range,
                            )
                        })
                    }
                    AnyValueContent::Range(..) => {
                        // TODO[slice-support] Temporary until we add slice types - we error here
                        span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]")
                    }
                    _ => span_range.type_err("The index must be an integer or a range"),
                }
            }
            [ctx] fn mutable(source: &'a mut ArrayValue, Spanned(index, span_range): Spanned<AnyValueRef>, _auto_create: bool) {
                match index {
                    AnyValueContent::Integer(integer) => {
                        let idx = source.resolve_valid_index_from_integer(Spanned(integer, span_range), false)?;
                        // SAFETY: ArrayChild correctly describes navigating to an array element
                        Ok(unsafe {
                            MappedMut::new(
                                &mut source.items[idx],
                                PathExtension::Child(
                                    ChildSpecifier::ArrayChild(idx),
                                    AnyType::type_kind(),
                                ),
                                ctx.output_span_range,
                            )
                        })
                    }
                    AnyValueContent::Range(..) => {
                        // TODO[slice-support] Temporary until we add slice types - we error here
                        span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]")
                    }
                    _ => span_range.type_err("The index must be an integer or a range"),
                }
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
