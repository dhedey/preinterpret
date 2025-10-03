use super::*;

pub(crate) enum EitherIterator<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Iterator for EitherIterator<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIterator::Left(l) => l.next(),
            EitherIterator::Right(r) => r.next(),
        }
    }
}

pub(crate) enum ZipIterators {
    Array(Vec<ExpressionIterator>, SpanRange),
    Object(Vec<(String, Span, ExpressionIterator)>, SpanRange),
}

impl ZipIterators {
    pub(crate) fn new_from_object(
        object: ExpressionObject,
        span_range: SpanRange,
    ) -> ExecutionResult<Self> {
        let entries = object
            .entries
            .into_iter()
            .take(101)
            .map(|(k, v)| -> ExecutionResult<_> {
                Ok((k, v.key_span, v.value.expect_any_iterator()?))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if entries.len() == 101 {
            return span_range.execution_err("A maximum of 100 iterators are allowed");
        }
        Ok(ZipIterators::Object(entries, span_range))
    }

    pub(crate) fn new_from_iterable(
        value: IterableValue,
        span_range: SpanRange,
    ) -> ExecutionResult<Self> {
        let iterator = value.into_iterator()?;
        let vec = iterator
            .take(101)
            .map(|x| x.expect_any_iterator())
            .collect::<Result<Vec<_>, _>>()?;
        if vec.len() == 101 {
            return span_range.execution_err("A maximum of 100 iterators are allowed");
        }
        Ok(ZipIterators::Array(vec, span_range))
    }

    pub(crate) fn run_zip(
        self,
        interpreter: &mut Interpreter,
        error_on_length_mismatch: bool,
    ) -> ExecutionResult<ExpressionArray> {
        let mut iterators = self;
        let output_span_range = match &iterators {
            ZipIterators::Array(_, span_range) => *span_range,
            ZipIterators::Object(_, span_range) => *span_range,
        };
        let mut output = Vec::new();

        if iterators.len() == 0 {
            return Ok(ExpressionArray {
                items: output,
                span_range: output_span_range,
            });
        }

        let (min_iterator_min_length, max_iterator_max_length) = iterators.size_hint_range();

        if error_on_length_mismatch && Some(min_iterator_min_length) != max_iterator_max_length {
            return output_span_range.execution_err(format!(
                "The iterables have different lengths. The lengths vary from {} to {}. To truncate to the shortest, use `zip_truncated` instead of `zip`",
                min_iterator_min_length,
                match max_iterator_max_length {
                    Some(max_max) => max_max.to_string(),
                    None => "unbounded".to_string(),
                },
            ));
        }

        iterators.zip_into(
            min_iterator_min_length,
            interpreter,
            output_span_range,
            &mut output,
        )?;

        Ok(ExpressionArray {
            items: output,
            span_range: output_span_range,
        })
    }

    /// Panics if called on an empty list of iterators
    fn size_hint_range(&self) -> (usize, Option<usize>) {
        let size_hints: Vec<_> = match self {
            ZipIterators::Array(inner, _) => inner.iter().map(|x| x.size_hint()).collect(),
            ZipIterators::Object(inner, _) => inner.iter().map(|x| x.2.size_hint()).collect(),
        };
        let min_min = size_hints.iter().map(|s| s.0).min().unwrap();
        let max_max = size_hints
            .iter()
            .map(|sh| sh.1)
            .max_by(|a, b| match (a, b) {
                (None, None) => core::cmp::Ordering::Equal,
                (None, Some(_)) => core::cmp::Ordering::Greater,
                (Some(_), None) => core::cmp::Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
            .unwrap();
        (min_min, max_max)
    }

    fn len(&self) -> usize {
        match self {
            ZipIterators::Array(inner, _) => inner.len(),
            ZipIterators::Object(inner, _) => inner.len(),
        }
    }

    /// Panics if count is larger than the minimum length of any iterator
    fn zip_into(
        &mut self,
        count: usize,
        interpreter: &mut Interpreter,
        output_span_range: SpanRange,
        output: &mut Vec<ExpressionValue>,
    ) -> ExecutionResult<()> {
        let mut counter = interpreter.start_iteration_counter(&output_span_range);

        match self {
            ZipIterators::Array(iterators, _) => {
                for _ in 0..count {
                    counter.increment_and_check()?;
                    let mut inner = Vec::with_capacity(iterators.len());
                    for iter in iterators.iter_mut() {
                        inner.push(iter.next().unwrap());
                    }
                    output.push(inner.to_value(output_span_range));
                }
            }
            ZipIterators::Object(iterators, _) => {
                for _ in 0..count {
                    counter.increment_and_check()?;
                    let mut inner = BTreeMap::new();
                    for (key, key_span, iter) in iterators.iter_mut() {
                        inner.insert(
                            key.clone(),
                            ObjectEntry {
                                key_span: *key_span,
                                value: iter.next().unwrap(),
                            },
                        );
                    }
                    output.push(inner.to_value(output_span_range));
                }
            }
        }

        Ok(())
    }
}
