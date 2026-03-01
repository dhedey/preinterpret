use super::*;

define_leaf_type! {
    pub(crate) IteratorType => AnyType(AnyValueContent::Iterator),
    content: IteratorValue,
    kind: pub(crate) IteratorKind,
    type_name: "iterator",
    articled_value_name: "an iterator",
    dyn_impls: {
        IterableType: impl IsIterable {
            fn into_iterator(self: Box<Self>) -> FunctionResult<IteratorValue> {
                Ok(*self)
            }

            fn iterable_len(&self, error_span_range: SpanRange) -> FunctionResult<usize> {
                self.do_len(error_span_range)
            }
        }
    },
}

pub(crate) type ValueIterator = Box<dyn BoxedIterator<Item = AnyValue>>;

#[derive(Clone)]
pub(crate) struct IteratorValue {
    inner: ValueIterator,
}

impl IteratorValue {
    pub(crate) fn new(iterator: ValueIterator) -> Self {
        Self { inner: iterator }
    }

    #[allow(unused)]
    pub(crate) fn new_any(iterator: impl Iterator<Item = AnyValue> + 'static + Clone) -> Self {
        Self::new(Box::new(iterator))
    }

    pub(crate) fn new_for_array(array: ArrayValue) -> Self {
        Self::new(Box::new(array.items.into_iter()))
    }

    pub(crate) fn new_for_stream(stream: OutputStream) -> Self {
        Self::new(Box::new(StreamValueIterator(stream.into_iter())))
    }

    pub(crate) fn new_for_range(range: RangeValue) -> FunctionResult<Self> {
        let iterator = range.inner.into_iterable()?.resolve_iterator()?;
        Ok(Self::new(iterator))
    }

    pub(crate) fn new_for_object(object: ObjectValue) -> Self {
        // We have to collect to vec and back to make it clonable
        let iterator = object
            .entries
            .into_iter()
            .map(|(k, v)| vec![k.into_any_value(), v.value].into_any_value())
            .collect::<Vec<_>>()
            .into_iter();
        Self::new(Box::new(iterator))
    }

    pub(crate) fn new_for_string_over_chars(string: String) -> Self {
        // We have to collect to vec and back to make the iterator owned
        // That's because value.chars() creates a `Chars<'_>` iterator which
        // borrows from the string, which we don't allow in a Boxed iterator
        // TODO: Replace with the owned iterator here to avoid this clone:
        // https://internals.rust-lang.org/t/is-there-a-good-reason-why-string-has-no-into-chars/19496/5
        let iterator = string
            .chars()
            .map(|c| c.into_any_value())
            .collect::<Vec<_>>()
            .into_iter();
        Self::new(Box::new(iterator))
    }

    /// Returns the inner BoxedIterator box (for creating Map/Filter wrappers).
    pub(crate) fn into_inner(self) -> ValueIterator {
        self.inner
    }

    pub(crate) fn singleton_value(
        mut self,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<Option<AnyValue>> {
        let first = match self.do_next(interpreter)? {
            Some(v) => v,
            None => return Ok(None),
        };
        if self.do_next(interpreter)?.is_none() {
            Ok(Some(first))
        } else {
            Ok(None)
        }
    }

    pub(super) fn output_items_to(
        mut self,
        output: &mut ToStreamContext,
        grouping: Grouping,
    ) -> FunctionResult<()> {
        const LIMIT: usize = 10_000;
        let mut i = 0;
        loop {
            let item = match output.with_interpreter(|interpreter| self.do_next(interpreter))? {
                Some(item) => item,
                None => break,
            };
            if i > LIMIT {
                return output.debug_err(format!("Only a maximum of {} items can be output to a stream from an iterator, to protect you from infinite loops. This can't currently be reconfigured with the iteration limit.", LIMIT));
            }
            item.as_ref_value().output_to(grouping, output)?;
            i += 1;
        }
        Ok(())
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<()> {
        if behaviour.use_debug_literal_syntax {
            any_items_to_string(
                &mut self.clone(),
                output,
                behaviour,
                "[<iterator>]",
                "[<iterator> ",
                "]",
                true,
                interpreter,
            )
        } else {
            output.push_str("Iterator[?]");
            Ok(())
        }
    }
}

#[derive(Clone)]
struct StreamValueIterator(OutputStreamIntoIter);

impl PreinterpretIterator for StreamValueIterator {
    type Item = AnyValue;
    fn do_next(&mut self, _: &mut Interpreter) -> FunctionResult<Option<AnyValue>> {
        Ok(Iterator::next(&mut self.0).map(|segment| {
            let stream: OutputStream = segment.into();
            stream.coerce_into_value()
        }))
    }
    fn do_size_hint(&self) -> (usize, Option<usize>) {
        Iterator::size_hint(&self.0)
    }
}

impl PreinterpretIterator for IteratorValue {
    type Item = AnyValue;
    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<AnyValue>> {
        self.inner.do_next(interpreter)
    }
    fn do_size_hint(&self) -> (usize, Option<usize>) {
        self.inner.do_size_hint()
    }
}

impl IsValueContent for ValueIterator {
    type Type = IteratorType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for ValueIterator {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        IteratorValue::new(self)
    }
}

impl_resolvable_argument_for! {
    IteratorType,
    (value, context) -> IteratorValue {
        match value {
            AnyValue::Iterator(value) => Ok(value),
            _ => context.err("an iterator", value),
        }
    }
}

impl ValuesEqual for IteratorValue {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if std::ptr::eq(self, other) {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(&"Iterator[?]", &"Iterator[?]")
        }
    }
}

define_type_features! {
    impl IteratorType,
    pub(crate) mod iterator_interface {
        methods {
            [context] fn next(mut this: Mutable<IteratorValue>) -> FunctionResult<AnyValue> {
                match this.do_next(context.interpreter)? {
                    Some(value) => Ok(value),
                    None => Ok(().into_any_value()),
                }
            }

            fn skip(this: IteratorValue, n: OptionalSuffix<usize>) -> ValueIterator {
                this.do_skip(n.0).boxed()
            }

            fn take(this: IteratorValue, n: OptionalSuffix<usize>) -> ValueIterator {
                this.do_take(n.0).boxed()
            }

            [context] fn map(this: IteratorValue, function: FunctionValue) -> IteratorValue {
                let inner = this.into_inner();
                let span = context.output_span_range;
                let f = move |item: AnyValue, interpreter: &mut Interpreter| -> FunctionResult<AnyValue> {
                    let mut ctx = FunctionCallContext { interpreter, output_span_range: span };
                    let argument = Spanned(ArgumentValue::Owned(item), span);
                    let result = function.clone().invoke(vec![argument], &mut ctx)?;
                    let owned = RequestedOwnership::owned()
                        .map_from_returned(result)?
                        .0
                        .expect_owned();
                    Ok(owned)
                };
                IteratorValue::new(Box::new(MapIterator::new(inner, f)))
            }

            [context] fn filter(this: IteratorValue, function: FunctionValue) -> IteratorValue {
                let inner = this.into_inner();
                let span = context.output_span_range;
                let f = move |item: &AnyValue, interpreter: &mut Interpreter| -> FunctionResult<bool> {
                    let mut ctx = FunctionCallContext { interpreter, output_span_range: span };
                    let argument = Spanned(ArgumentValue::Owned(item.clone()), span);
                    let result = function.clone().invoke(vec![argument], &mut ctx)?;
                    let owned = RequestedOwnership::owned()
                        .map_from_returned(result)?
                        .0
                        .expect_owned();
                    let keep: bool = owned
                        .spanned(span)
                        .resolve_as("The result of a filter predicate")?;
                    Ok(keep)
                };
                IteratorValue::new(Box::new(FilterIterator::new(inner, f)))
            }
        }
        unary_operations {
            [context] fn cast_singleton_to_value(Spanned(this, span): Spanned<IteratorValue>) -> FunctionResult<ReturnedValue> {
                match this.singleton_value(context.interpreter)? {
                    Some(value) => Ok(context.operation.evaluate(Spanned(value, span), context.interpreter)?.0),
                    None => span.value_err("Only an iterator with one item can be cast to this value"),
                }
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
        }
    }
}
