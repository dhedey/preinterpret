use super::*;

define_interface! {
    struct IterableTypeData,
    parent: ValueTypeData,
    pub(crate) mod iterable_interface {
        pub(crate) mod methods {
            fn into_iter(this: IterableValue) -> ExecutionResult<ExpressionIterator> {
                this.into_iterator()
            }

            fn len(this: Spanned<IterableRef>) -> ExecutionResult<usize> {
                this.len()
            }

            fn is_empty(this: Spanned<IterableRef>) -> ExecutionResult<bool> {
                Ok(this.len()? == 0)
            }

            [context] fn zip(this: IterableValue) -> ExecutionResult<ExpressionArray> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, true)
            }

            [context] fn zip_truncated(this: IterableValue) -> ExecutionResult<ExpressionArray> {
                let iterator = this.into_iterator()?;
                ZipIterators::new_from_iterator(iterator, context.span_range())?.run_zip(context.interpreter, false)
            }

            fn intersperse(this: IterableValue, separator: ExpressionValue, settings: Option<IntersperseSettings>) -> ExecutionResult<ExpressionArray> {
                run_intersperse(this, separator, settings.unwrap_or_default())
            }

            [context] fn to_vec(this: IterableValue) -> ExecutionResult<Vec<ExpressionValue>> {
                let error_span_range = context.span_range();
                let mut counter = context.interpreter.start_iteration_counter(&error_span_range);
                let iterator = this.into_iterator()?;
                let max_hint = iterator.size_hint().1;
                let mut vec = if let Some(max) = max_hint {
                    Vec::with_capacity(max)
                } else {
                    Vec::new()
                };
                for item in iterator {
                    counter.increment_and_check()?;
                    vec.push(item);
                }
                Ok(vec)
            }
        }
        pub(crate) mod unary_operations {
        }
        interface_items {
        }
    }
}

// If you add a new variant, also update:
// * ResolvableArgumentOwned for IterableValue
// * FromResolved for IterableRef
// * The parent of the value's TypeData to be IterableTypeData
pub(crate) enum IterableValue {
    Iterator(ExpressionIterator),
    Array(ExpressionArray),
    Stream(ExpressionStream),
    Object(ExpressionObject),
    Range(ExpressionRange),
    String(ExpressionString),
}

impl IterableValue {
    pub(crate) fn into_iterator(self) -> ExecutionResult<ExpressionIterator> {
        Ok(match self {
            IterableValue::Array(value) => ExpressionIterator::new_for_array(value),
            IterableValue::Stream(value) => ExpressionIterator::new_for_stream(value),
            IterableValue::Iterator(value) => value,
            IterableValue::Range(value) => ExpressionIterator::new_for_range(value)?,
            IterableValue::Object(value) => ExpressionIterator::new_for_object(value)?,
            IterableValue::String(value) => ExpressionIterator::new_for_string(value)?,
        })
    }
}

pub(crate) enum IterableRef<'a> {
    Iterator(Ref<'a, ExpressionIterator>),
    Array(Ref<'a, ExpressionArray>),
    Stream(Ref<'a, OutputStream>),
    Range(Ref<'a, ExpressionRange>),
    Object(Ref<'a, ExpressionObject>),
    String(Ref<'a, str>),
}

impl FromResolved for IterableRef<'static> {
    type ValueType = IterableTypeData;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Shared;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(match value.kind() {
            ValueKind::Iterator => IterableRef::Iterator(FromResolved::from_resolved(value)?),
            ValueKind::Array => IterableRef::Array(FromResolved::from_resolved(value)?),
            ValueKind::Stream => IterableRef::Stream(FromResolved::from_resolved(value)?),
            ValueKind::Range => IterableRef::Range(FromResolved::from_resolved(value)?),
            ValueKind::Object => IterableRef::Object(FromResolved::from_resolved(value)?),
            ValueKind::String => IterableRef::String(FromResolved::from_resolved(value)?),
            _ => {
                return value.execution_err(
                    "Expected iterable (iterator, array, object, stream, range or string)",
                )
            }
        })
    }
}

impl Spanned<IterableRef<'_>> {
    pub(crate) fn len(&self) -> ExecutionResult<usize> {
        match &self.value {
            IterableRef::Iterator(iterator) => iterator.len(self.span_range),
            IterableRef::Array(value) => Ok(value.items.len()),
            IterableRef::Stream(value) => Ok(value.len()),
            IterableRef::Range(value) => value.len(self.span_range),
            IterableRef::Object(value) => Ok(value.entries.len()),
            // NB - this is different to string.len() which counts bytes
            IterableRef::String(value) => Ok(value.chars().count()),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ExpressionIterator {
    iterator: ExpressionIteratorInner,
}

impl ExpressionIterator {
    pub(crate) fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize> {
        let (min, max) = self.size_hint();
        if max == Some(min) {
            Ok(min)
        } else {
            error_span_range.execution_err("Iterator has an inexact length")
        }
    }

    #[allow(unused)]
    pub(crate) fn new_any(
        iterator: impl Iterator<Item = ExpressionValue> + 'static + Clone,
    ) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Other(Box::new(iterator)),
        }
    }

    pub(crate) fn new_for_array(array: ExpressionArray) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Vec(array.items.into_iter()),
        }
    }

    pub(crate) fn new_for_stream(stream: ExpressionStream) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Stream(stream.value.into_iter()),
        }
    }

    pub(crate) fn new_for_range(range: ExpressionRange) -> ExecutionResult<Self> {
        let iterator = range.inner.into_iterable()?.resolve_iterator()?;
        Ok(Self {
            iterator: ExpressionIteratorInner::Other(iterator),
        })
    }

    pub(crate) fn new_for_object(object: ExpressionObject) -> ExecutionResult<Self> {
        // We have to collect to vec and back to make it clonable
        let iterator = object
            .entries
            .into_iter()
            .map(|(k, v)| vec![k.into_value(), v.value].into_value())
            .collect::<Vec<_>>()
            .into_iter();
        Ok(Self {
            iterator: ExpressionIteratorInner::Vec(iterator),
        })
    }

    pub(crate) fn new_for_string(string: ExpressionString) -> ExecutionResult<Self> {
        // We have to collect to vec and back to make the iterator owned
        // That's because value.chars() creates a `Chars<'_>` iterator which
        // borrows from the string, which we don't allow in a Boxed iterator
        let iterator = string
            .value
            .chars()
            .map(|c| c.into_value())
            .collect::<Vec<_>>()
            .into_iter();
        Ok(Self {
            iterator: ExpressionIteratorInner::Vec(iterator),
        })
    }

    pub(crate) fn new_custom(iterator: Box<dyn CustomExpressionIterator>) -> Self {
        Self {
            iterator: ExpressionIteratorInner::Other(iterator),
        }
    }

    pub(crate) fn singleton_value(mut self) -> Option<ExpressionValue> {
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
                return output.execution_err(format!("Only a maximum of {} items can be output to a stream from an iterator, to protect you from infinite loops. This can't currently be reconfigured with the iteration limit.", LIMIT));
            }
            item.output_to(grouping, output)?;
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

    pub(crate) fn any_iterator_to_string<T: Borrow<ExpressionValue>>(
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
                    return behaviour.error_span_range.execution_err(format!("To protect against infinite loops, only a maximum of {} items can be output to a string from an iterator. You can use .to_vec() to avoid this limit. This can't currently be reconfigured with the iteration limit.", behaviour.iterator_limit));
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
            item.concat_recursive_into(output, behaviour)?;
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

impl ToExpressionValue for ExpressionIteratorInner {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator { iterator: self })
    }
}

impl ToExpressionValue for Box<dyn CustomExpressionIterator> {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator::new_custom(self))
    }
}

impl ToExpressionValue for ExpressionIterator {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Iterator(ExpressionIterator {
            iterator: self.iterator,
        })
    }
}

impl HasValueType for ExpressionIterator {
    fn value_type(&self) -> &'static str {
        "iterator"
    }
}

#[derive(Clone)]
enum ExpressionIteratorInner {
    Vec(<Vec<ExpressionValue> as IntoIterator>::IntoIter),
    Stream(<OutputStream as IntoIterator>::IntoIter),
    Other(Box<dyn CustomExpressionIterator>),
}

impl<T: Iterator<Item = ExpressionValue> + Clone + 'static> CustomExpressionIterator for T {
    fn clone_box(&self) -> Box<dyn CustomExpressionIterator> {
        Box::new(self.clone())
    }
}

pub(crate) trait CustomExpressionIterator: Iterator<Item = ExpressionValue> {
    fn clone_box(&self) -> Box<dyn CustomExpressionIterator>;
}

impl Clone for Box<dyn CustomExpressionIterator> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl Iterator for ExpressionIterator {
    type Item = ExpressionValue;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iterator {
            ExpressionIteratorInner::Vec(iter) => iter.next(),
            ExpressionIteratorInner::Stream(iter) => {
                let item = iter.next()?;
                let stream: OutputStream = item.into();
                Some(stream.coerce_into_value())
            }
            ExpressionIteratorInner::Other(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iterator {
            ExpressionIteratorInner::Vec(iter) => iter.size_hint(),
            ExpressionIteratorInner::Stream(iter) => iter.size_hint(),
            ExpressionIteratorInner::Other(iter) => iter.size_hint(),
        }
    }
}

impl Iterator for Mutable<ExpressionIterator> {
    type Item = ExpressionValue;

    fn next(&mut self) -> Option<Self::Item> {
        let this: &mut ExpressionIterator = &mut *self;
        this.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let this: &ExpressionIterator = self;
        this.size_hint()
    }
}

define_interface! {
    struct IteratorTypeData,
    parent: IterableTypeData,
    pub(crate) mod iterator_interface {
        pub(crate) mod methods {
            fn next(mut this: Mutable<ExpressionIterator>) -> ExpressionValue {
                match this.next() {
                    Some(value) => value,
                    None => ExpressionValue::None,
                }
            }

            fn skip(mut this: ExpressionIterator, n: usize) -> ExpressionIterator {
                // We make this greedy instead of lazy because the Skip iterator is not clonable.
                // We return an iterator for forwards compatibility in case we change it.
                for _ in 0..n {
                    if this.next().is_none() {
                        break;
                    }
                }
                this
            }

            fn take(this: ExpressionIterator, n: usize) -> ExpressionIterator {
                // We collect to a vec to satisfy the clonability requirement,
                // but only return an iterator for forwards compatibility in case we change it.
                let taken = this.take(n).collect::<Vec<_>>();
                ExpressionIterator::new_for_array(ExpressionArray::new(taken))
            }
        }
        pub(crate) mod unary_operations {
            [context] fn cast_singleton_to_value(this: Owned<ExpressionIterator>) -> ExecutionResult<ResolvedValue> {
                let (this, input_span_range) = this.deconstruct();
                match this.singleton_value() {
                    Some(value) => context.operation.evaluate(Owned::new(value, input_span_range)),
                    None => input_span_range.execution_err("Only an iterator with one item can be cast to this value")
                }
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
                        | CastTarget::Float(_) => unary_definitions::cast_singleton_to_value(),
                        _ => return None,
                    },
                })
            }
        }
    }
}
