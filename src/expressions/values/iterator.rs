use super::*;

define_leaf_type! {
    pub(crate) IteratorType => AnyType(AnyValueContent::Iterator),
    content: IteratorValue,
    kind: pub(crate) IteratorKind,
    type_name: "iterator",
    articled_value_name: "an iterator",
    dyn_impls: {
        IterableType: impl IsIterable {
            fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue> {
                Ok(*self)
            }

            fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize> {
                self.len(error_span_range)
            }
        }
    },
}

#[derive(Clone)]
pub(crate) struct IteratorValue {
    iterator: IteratorValueInner,
}

impl IteratorValue {
    fn new(iterator: IteratorValueInner) -> Self {
        Self { iterator }
    }

    #[allow(unused)]
    pub(crate) fn new_any(iterator: impl Iterator<Item = AnyValue> + 'static + Clone) -> Self {
        Self::new_custom(Box::new(iterator))
    }

    pub(crate) fn new_for_array(array: ArrayValue) -> Self {
        Self::new_vec(array.items.into_iter())
    }

    pub(crate) fn new_for_stream(stream: OutputStream) -> Self {
        Self::new(IteratorValueInner::Stream(Box::new(stream.into_iter())))
    }

    pub(crate) fn new_for_range(range: RangeValue) -> ExecutionResult<Self> {
        let iterator = range.inner.into_iterable()?.resolve_iterator()?;
        Ok(Self::new_custom(iterator))
    }

    pub(crate) fn new_for_object(object: ObjectValue) -> Self {
        // We have to collect to vec and back to make it clonable
        let iterator = object
            .entries
            .into_iter()
            .map(|(k, v)| vec![k.into_any_value(), v.value].into_any_value())
            .collect::<Vec<_>>()
            .into_iter();
        Self::new_vec(iterator)
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
        Self::new_vec(iterator)
    }

    fn new_vec(iterator: std::vec::IntoIter<AnyValue>) -> Self {
        Self::new(IteratorValueInner::Vec(Box::new(iterator)))
    }

    pub(crate) fn new_custom(iterator: Box<dyn ClonableIterator<Item = AnyValue>>) -> Self {
        Self::new(IteratorValueInner::Other(iterator))
    }

    pub(crate) fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize> {
        let (min, max) = self.size_hint();
        if max == Some(min) {
            Ok(min)
        } else {
            error_span_range.value_err("Iterator has an inexact length")
        }
    }

    pub(crate) fn singleton_value(mut self) -> Option<AnyValue> {
        let first = self.next()?;
        if self.next().is_none() {
            Some(first)
        } else {
            None
        }
    }

    pub(super) fn output_items_to(
        self,
        output: &mut ToStreamContext,
        grouping: Grouping,
    ) -> ExecutionResult<()> {
        const LIMIT: usize = 10_000;
        for (i, item) in self.enumerate() {
            if i > LIMIT {
                return output.debug_err(format!("Only a maximum of {} items can be output to a stream from an iterator, to protect you from infinite loops. This can't currently be reconfigured with the iteration limit.", LIMIT));
            }
            item.as_ref_value().output_to(grouping, output)?;
        }
        Ok(())
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        Self::any_iterator_to_string(
            self.clone(),
            output,
            behaviour,
            "[<iterator>]",
            "[<iterator> ",
            "]",
            true,
        )
    }

    pub(crate) fn any_iterator_to_string<T: Borrow<AnyValue>>(
        iterator: impl Iterator<Item = T>,
        output: &mut String,
        behaviour: &ConcatBehaviour,
        literal_empty: &str,
        literal_start: &str,
        literal_end: &str,
        possibly_unbounded: bool,
    ) -> ExecutionResult<()> {
        let mut is_empty = true;
        let max = iterator.size_hint().1;
        for (i, item) in iterator.enumerate() {
            if i == 0 {
                if behaviour.output_literal_structure {
                    output.push_str(literal_start);
                }
                is_empty = false;
            }
            if possibly_unbounded && i >= behaviour.iterator_limit {
                if behaviour.error_after_iterator_limit {
                    return behaviour.error_span_range.debug_err(format!("To protect against infinite loops, only a maximum of {} items can be output to a string from an iterator. You can use .to_vec() to avoid this limit. This can't currently be reconfigured with the iteration limit.", behaviour.iterator_limit));
                } else {
                    if behaviour.output_literal_structure {
                        match max {
                            Some(max) => output.push_str(&format!(
                                ", ..<{} further items>",
                                max.saturating_sub(i)
                            )),
                            None => output.push_str(", ..<possibly unbounded>"),
                        }
                    }
                    break;
                }
            }
            let item = item.borrow();
            if i != 0 && behaviour.output_literal_structure {
                output.push(',');
            }
            if i != 0 && behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            item.as_ref_value()
                .concat_recursive_into(output, behaviour)?;
        }
        if behaviour.output_literal_structure {
            if is_empty {
                output.push_str(literal_empty);
            } else {
                output.push_str(literal_end);
            }
        }
        Ok(())
    }
}

impl IsValueContent for IteratorValueInner {
    type Type = IteratorType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for IteratorValueInner {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        IteratorValue::new(self)
    }
}

impl IsValueContent for Box<dyn ClonableIterator<Item = AnyValue>> {
    type Type = IteratorType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for Box<dyn ClonableIterator<Item = AnyValue>> {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        IteratorValue::new_custom(self)
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

fn definite_size_hint(size_hint: (usize, Option<usize>)) -> Option<usize> {
    let (min, max) = size_hint;
    if let Some(max) = max {
        if min == max {
            Some(min)
        } else {
            None
        }
    } else {
        None
    }
}

impl ValuesEqual for IteratorValue {
    /// Compares two iterators by cloning and comparing element-by-element.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        let mut lhs_iter = self.clone();
        let mut rhs_iter = other.clone();
        let mut index = 0;
        let lhs_size = definite_size_hint(lhs_iter.size_hint());
        let rhs_size = definite_size_hint(rhs_iter.size_hint());
        const MAX_ITERATIONS: usize = 1000;
        while index < MAX_ITERATIONS {
            match (lhs_iter.next(), rhs_iter.next()) {
                (Some(l), Some(r)) => {
                    let result = ctx.with_iterator_index(index, |ctx| l.test_equality(&r, ctx));
                    if ctx.should_short_circuit(&result) {
                        return result;
                    }
                    index += 1;
                }
                (None, None) => return ctx.values_equal(),
                _ => return ctx.lengths_unequal(lhs_size, rhs_size),
            }
        }
        ctx.iteration_limit_exceeded(MAX_ITERATIONS)
    }
}

#[derive(Clone)]
enum IteratorValueInner {
    // We Box these so that Value is smaller on the stack
    Vec(Box<<Vec<AnyValue> as IntoIterator>::IntoIter>),
    Stream(Box<<OutputStream as IntoIterator>::IntoIter>),
    Other(Box<dyn ClonableIterator<Item = AnyValue>>),
}

impl Iterator for IteratorValue {
    type Item = AnyValue;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iterator {
            IteratorValueInner::Vec(iter) => iter.next(),
            IteratorValueInner::Stream(iter) => {
                let item = iter.next()?;
                let stream: OutputStream = item.into();
                Some(stream.coerce_into_value())
            }
            IteratorValueInner::Other(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iterator {
            IteratorValueInner::Vec(iter) => iter.size_hint(),
            IteratorValueInner::Stream(iter) => iter.size_hint(),
            IteratorValueInner::Other(iter) => iter.size_hint(),
        }
    }
}

impl Iterator for Mutable<IteratorValue> {
    type Item = AnyValue;

    fn next(&mut self) -> Option<Self::Item> {
        let this: &mut IteratorValue = &mut *self;
        this.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let this: &IteratorValue = self;
        this.size_hint()
    }
}

define_type_features! {
    impl IteratorType,
    pub(crate) mod iterator_interface {
        methods {
            fn next(mut this: Mutable<IteratorValue>) -> AnyValue {
                match this.next() {
                    Some(value) => value,
                    None => ().into_any_value(),
                }
            }

            fn skip(mut this: IteratorValue, n: OptionalSuffix<usize>) -> IteratorValue {
                // We make this greedy instead of lazy because the Skip iterator is not clonable.
                // We return an iterator for forwards compatibility in case we change it.
                for _ in 0..n.0 {
                    if this.next().is_none() {
                        break;
                    }
                }
                this
            }

            fn take(this: IteratorValue, n: OptionalSuffix<usize>) -> IteratorValue {
                // We collect to a vec to satisfy the clonability requirement,
                // but only return an iterator for forwards compatibility in case we change it.
                let taken = this.take(n.0).collect::<Vec<_>>();
                IteratorValue::new_for_array(ArrayValue::new(taken))
            }
        }
        unary_operations {
            [context] fn cast_singleton_to_value(Spanned(this, span): Spanned<IteratorValue>) -> ExecutionResult<ReturnedValue> {
                match this.singleton_value() {
                    Some(value) => Ok(context.operation.evaluate(Spanned(value, span))?.0),
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
