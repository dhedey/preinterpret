use super::*;

impl<I: Iterator + Clone + 'static> ClonableIterator for I {
    fn clone_box(&self) -> Box<dyn ClonableIterator<Item = Self::Item>> {
        Box::new(self.clone())
    }
}

pub(crate) trait ClonableIterator: Iterator {
    fn clone_box(&self) -> Box<dyn ClonableIterator<Item = Self::Item>>;
}

impl<T> Clone for Box<dyn ClonableIterator<Item = T>> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

#[derive(Clone)]
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
                Ok((
                    k,
                    v.key_span,
                    v.value
                        .into_owned(span_range)
                        .expect_any_iterator("Each zip input")?
                        .into_inner(),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if entries.len() == 101 {
            return span_range.value_err("A maximum of 100 iterators are allowed");
        }
        Ok(ZipIterators::Object(entries, span_range))
    }

    pub(crate) fn new_from_iterator(
        iterator: ExpressionIterator,
        span_range: SpanRange,
    ) -> ExecutionResult<Self> {
        let vec = iterator
            .take(101)
            .map(|x| {
                x.into_owned(span_range)
                    .expect_any_iterator("Each zip input")
                    .map(|x| x.into_inner())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if vec.len() == 101 {
            return span_range.value_err("A maximum of 100 iterators are allowed");
        }
        Ok(ZipIterators::Array(vec, span_range))
    }

    pub(crate) fn run_zip(
        self,
        interpreter: &mut Interpreter,
        error_on_length_mismatch: bool,
    ) -> ExecutionResult<ExpressionArray> {
        let mut iterators = self;
        let error_span_range = match &iterators {
            ZipIterators::Array(_, span_range) => *span_range,
            ZipIterators::Object(_, span_range) => *span_range,
        };
        let mut output = Vec::new();

        if iterators.len() == 0 {
            return Ok(ExpressionArray::new(output));
        }

        let (min_iterator_min_length, max_iterator_max_length) = iterators.size_hint_range();

        if error_on_length_mismatch && Some(min_iterator_min_length) != max_iterator_max_length {
            return error_span_range.value_err(format!(
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
            error_span_range,
            &mut output,
        )?;

        Ok(ExpressionArray::new(output))
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
        error_span_range: SpanRange,
        output: &mut Vec<ExpressionValue>,
    ) -> ExecutionResult<()> {
        let mut counter = interpreter.start_iteration_counter(&error_span_range);

        match self {
            ZipIterators::Array(iterators, _) => {
                for _ in 0..count {
                    counter.increment_and_check()?;
                    let mut inner = Vec::with_capacity(iterators.len());
                    for iter in iterators.iter_mut() {
                        inner.push(iter.next().unwrap());
                    }
                    output.push(inner.into_value());
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
                    output.push(inner.into_value());
                }
            }
        }

        Ok(())
    }
}

define_optional_object! {
    pub(crate) struct IntersperseSettings {
        add_trailing: bool = false => ("false", "Whether to add the separator after the last item (default: false)"),
        final_separator: ExpressionValue => ("%[or]", "Define a different final separator (default: same as normal separator)"),
    }
}

pub(crate) fn run_intersperse(
    items: IterableValue,
    separator: ExpressionValue,
    settings: IntersperseSettings,
) -> ExecutionResult<ExpressionArray> {
    let mut output = Vec::new();

    let mut items = items.into_iterator()?.peekable();

    let mut this_item = match items.next() {
        Some(next) => next,
        None => return Ok(ExpressionArray { items: output }),
    };

    let mut appender = SeparatorAppender {
        separator,
        final_separator: settings.final_separator,
        add_trailing: settings.add_trailing,
    };

    loop {
        output.push(this_item);
        let next_item = items.next();
        match next_item {
            Some(next_item) => {
                let remaining = if items.peek().is_some() {
                    RemainingItemCount::MoreThanOne
                } else {
                    RemainingItemCount::ExactlyOne
                };
                appender.add_separator(remaining, &mut output)?;
                this_item = next_item;
            }
            None => {
                appender.add_separator(RemainingItemCount::None, &mut output)?;
                break;
            }
        }
    }

    Ok(ExpressionArray { items: output })
}

struct SeparatorAppender {
    separator: ExpressionValue,
    final_separator: Option<ExpressionValue>,
    add_trailing: bool,
}

impl SeparatorAppender {
    fn add_separator(
        &mut self,
        remaining: RemainingItemCount,
        output: &mut Vec<ExpressionValue>,
    ) -> ExecutionResult<()> {
        match self.separator(remaining) {
            TrailingSeparator::Normal => output.push(self.separator.clone()),
            TrailingSeparator::Final => match self.final_separator.take() {
                Some(final_separator) => output.push(final_separator),
                None => output.push(self.separator.clone()),
            },
            TrailingSeparator::None => {}
        }
        Ok(())
    }

    fn separator(&self, remaining_item_count: RemainingItemCount) -> TrailingSeparator {
        match remaining_item_count {
            RemainingItemCount::None => {
                if self.add_trailing {
                    TrailingSeparator::Final
                } else {
                    TrailingSeparator::None
                }
            }
            RemainingItemCount::ExactlyOne => {
                if self.add_trailing {
                    TrailingSeparator::Normal
                } else {
                    TrailingSeparator::Final
                }
            }
            RemainingItemCount::MoreThanOne => TrailingSeparator::Normal,
        }
    }
}

enum RemainingItemCount {
    None,
    ExactlyOne,
    MoreThanOne,
}

enum TrailingSeparator {
    Normal,
    Final,
    None,
}

define_optional_object! {
    pub(crate) struct SplitSettings {
        drop_empty_start: bool = false => ("false", "If true, a leading separator does not yield in an empty item at the start (default: false)"),
        drop_empty_middle: bool = false => ("false", "If true, adjacent separators do not yield an empty item between them (default: false)"),
        drop_empty_end: bool = true => ("true", "If true, a trailing separator does not yield an empty item at the end (default: true)"),
    }
}

pub(crate) fn handle_split(
    input: OutputStream,
    separator: &OutputStream,
    settings: SplitSettings,
) -> ExecutionResult<ExpressionArray> {
    unsafe {
        // RUST-ANALYZER SAFETY: This is as safe as we can get.
        // Typically the separator won't contain none-delimited groups, so we're OK
        input.parse_with(move |input| {
            let mut output = Vec::new();
            let mut current_item = OutputStream::new();

            // Special case separator.len() == 0 to avoid an infinite loop
            if separator.is_empty() {
                while !input.is_empty() {
                    current_item.push_raw_token_tree(input.parse()?);
                    let complete_item = core::mem::replace(&mut current_item, OutputStream::new());
                    output.push(complete_item.into_value());
                }
                return Ok(ExpressionArray::new(output));
            }

            let mut drop_empty_next = settings.drop_empty_start;
            while !input.is_empty() {
                let separator_fork = input.fork();
                let mut ignored_transformer_output = OutputStream::new();
                if separator
                    .parse_exact_match(&separator_fork, &mut ignored_transformer_output)
                    .is_err()
                {
                    current_item.push_raw_token_tree(input.parse()?);
                    continue;
                }
                // This is guaranteed to progress the parser because the separator is non-empty
                input.advance_to(&separator_fork);
                if !current_item.is_empty() || !drop_empty_next {
                    let complete_item = core::mem::replace(&mut current_item, OutputStream::new());
                    output.push(complete_item.into_value());
                }
                drop_empty_next = settings.drop_empty_middle;
            }
            if !current_item.is_empty() || !settings.drop_empty_end {
                output.push(current_item.into_value());
            }
            Ok(ExpressionArray::new(output))
        })
    }
}
