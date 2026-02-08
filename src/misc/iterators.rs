use super::*;

// ============================================================================
// PreinterpretIterator trait — associated Item, no Clone/'static required
// ============================================================================

/// An iterator protocol that takes `&mut Interpreter` on each `next()` call.
/// This allows iterators like `Map` and `Filter` to evaluate closures during iteration.
///
/// Unlike `Iterator`, this has an associated `Item` type and passes the interpreter
/// through on each call. Types that don't need the interpreter (like `StdIteratorAdapter`)
/// simply ignore it.
pub(crate) trait PreinterpretIterator {
    type Item;
    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<Self::Item>>;
    fn do_size_hint(&self) -> (usize, Option<usize>);

    fn do_len(&self, error_span_range: SpanRange) -> FunctionResult<usize> {
        let (min, max) = self.do_size_hint();
        if max == Some(min) {
            Ok(min)
        } else {
            error_span_range.value_err("Iterator has an inexact length")
        }
    }

    fn do_map<F, B>(self, f: F) -> MapIterator<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item, &mut Interpreter) -> FunctionResult<B>,
    {
        MapIterator::new(self, f)
    }

    fn do_take(self, count: usize) -> TakeIterator<Self>
    where
        Self: Sized,
    {
        TakeIterator::new(self, count)
    }

    fn do_skip(self, count: usize) -> SkipIterator<Self>
    where
        Self: Sized,
    {
        SkipIterator::new(self, count)
    }

    fn boxed(self) -> Box<dyn BoxedIterator<Item = Self::Item>>
    where
        Self: Sized + 'static + Clone,
    {
        Box::new(self)
    }

    fn do_into_iter<'i>(self, interpreter: &'i mut Interpreter) -> PreinterpretToIterator<'i, Self>
    where
        Self: Sized + 'i,
    {
        PreinterpretToIterator {
            inner: self,
            errored: false,
            interpreter,
        }
    }

    fn do_collect<T: FromIterator<Self::Item>>(self, interpreter: &mut Interpreter) -> FunctionResult<T>
    where
        Self: Sized,
    {
        self.do_into_iter(interpreter).collect()
    }
}

impl<I: Iterator> PreinterpretIterator for I {
    type Item = I::Item;
    fn do_next(&mut self, _: &mut Interpreter) -> FunctionResult<Option<Self::Item>> {
        Ok(Iterator::next(self))
    }
    fn do_size_hint(&self) -> (usize, Option<usize>) {
        Iterator::size_hint(self)
    }
}

impl<Item> PreinterpretIterator for Box<dyn PreinterpretIterator<Item = Item>> {
    type Item = Item;

    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<Self::Item>> {
        (**self).do_next(interpreter)
    }

    fn do_size_hint(&self) -> (usize, Option<usize>) {
        (**self).do_size_hint()
    }
}

impl<Item> PreinterpretIterator for Box<dyn BoxedIterator<Item = Item>> {
    type Item = Item;

    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<Self::Item>> {
        (**self).do_next(interpreter)
    }

    fn do_size_hint(&self) -> (usize, Option<usize>) {
        (**self).do_size_hint()
    }
}

pub(crate) struct PreinterpretToIterator<'a, I> {
    inner: I,
    errored: bool,
    interpreter: &'a mut Interpreter,
}

impl<I: PreinterpretIterator> Iterator for PreinterpretToIterator<'_, I> {
    type Item = FunctionResult<I::Item>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }
        match self.inner.do_next(self.interpreter) {
            Ok(Some(item)) => Some(Ok(item)),
            Ok(None) => None,
            Err(e) => {
                self.errored = true;
                Some(Err(e))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.do_size_hint()
    }
}

// ============================================================================
// BoxedIterator — object-safe sub-trait with clone_box, stored in IteratorValue
// ============================================================================

/// An object-safe sub-trait with clone_box, stored in IteratorValue.
pub(crate) trait BoxedIterator: 'static + PreinterpretIterator {
    fn clone_box(&self) -> Box<dyn BoxedIterator<Item = Self::Item>>;
}

impl<T: ?Sized + PreinterpretIterator + 'static + Clone> BoxedIterator for T {
    fn clone_box(&self) -> Box<dyn BoxedIterator<Item = Self::Item>> {
        Box::new(self.clone())
    }
}

impl<I: 'static> Clone for Box<dyn BoxedIterator<Item = I>> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

// ============================================================================
// Generic MapIterator
// ============================================================================

/// An iterator that applies a function to each item of the inner iterator.
#[derive(Clone)]
pub(crate) struct MapIterator<I, F> {
    inner: I,
    f: F,
}

impl<I, F> MapIterator<I, F> {
    pub(crate) fn new(inner: I, f: F) -> Self {
        Self { inner, f }
    }
}

impl<I, F, O> PreinterpretIterator for MapIterator<I, F>
where
    I: PreinterpretIterator,
    F: FnMut(I::Item, &mut Interpreter) -> FunctionResult<O>,
{
    type Item = O;
    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<O>> {
        match self.inner.do_next(interpreter)? {
            Some(item) => Ok(Some((self.f)(item, interpreter)?)),
            None => Ok(None),
        }
    }
    fn do_size_hint(&self) -> (usize, Option<usize>) {
        self.inner.do_size_hint()
    }
}

// ============================================================================
// Generic FilterIterator
// ============================================================================

/// An iterator that filters items using a predicate function.
#[derive(Clone)]
pub(crate) struct FilterIterator<I, F> {
    inner: I,
    f: F,
}

impl<I, F> FilterIterator<I, F> {
    pub(crate) fn new(inner: I, f: F) -> Self {
        Self { inner, f }
    }
}

impl<I, F> PreinterpretIterator for FilterIterator<I, F>
where
    I: PreinterpretIterator,
    I::Item: Clone,
    F: FnMut(&I::Item, &mut Interpreter) -> FunctionResult<bool>,
{
    type Item = I::Item;
    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<I::Item>> {
        loop {
            let item = match self.inner.do_next(interpreter)? {
                Some(item) => item,
                None => return Ok(None),
            };
            if (self.f)(&item, interpreter)? {
                return Ok(Some(item));
            }
        }
    }
    fn do_size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.do_size_hint().1)
    }
}

// ============================================================================
// SkipIterator — lazy skip
// ============================================================================

/// An iterator that lazily skips the first `remaining` items.
#[derive(Clone)]
pub(crate) struct SkipIterator<I> {
    inner: I,
    remaining: usize,
}

impl<I> SkipIterator<I> {
    pub(crate) fn new(inner: I, count: usize) -> Self {
        Self {
            inner,
            remaining: count,
        }
    }
}

impl<I: PreinterpretIterator> PreinterpretIterator for SkipIterator<I> {
    type Item = I::Item;
    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<I::Item>> {
        while self.remaining > 0 {
            self.remaining -= 1;
            if self.inner.do_next(interpreter)?.is_none() {
                return Ok(None);
            }
        }
        self.inner.do_next(interpreter)
    }
    fn do_size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.inner.do_size_hint();
        (
            lo.saturating_sub(self.remaining),
            hi.map(|h| h.saturating_sub(self.remaining)),
        )
    }
}

// ============================================================================
// TakeIterator — lazy take
// ============================================================================

/// An iterator that yields at most `remaining` items from the inner iterator.
#[derive(Clone)]
pub(crate) struct TakeIterator<I> {
    inner: I,
    remaining: usize,
}

impl<I> TakeIterator<I> {
    pub(crate) fn new(inner: I, count: usize) -> Self {
        Self {
            inner,
            remaining: count,
        }
    }
}

impl<I: PreinterpretIterator> PreinterpretIterator for TakeIterator<I> {
    type Item = I::Item;
    fn do_next(&mut self, interpreter: &mut Interpreter) -> FunctionResult<Option<I::Item>> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        self.inner.do_next(interpreter)
    }
    fn do_size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.inner.do_size_hint();
        (
            lo.min(self.remaining),
            Some(match hi {
                Some(h) => h.min(self.remaining),
                None => self.remaining,
            }),
        )
    }
}

// ============================================================================
// Unified to_string for any PreinterpretIterator
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub(crate) fn any_items_to_string<T: Borrow<AnyValue>>(
    iterator: &mut impl PreinterpretIterator<Item = T>,
    output: &mut String,
    behaviour: &ConcatBehaviour,
    literal_empty: &str,
    literal_start: &str,
    literal_end: &str,
    possibly_unbounded: bool,
    interpreter: &mut Interpreter,
) -> FunctionResult<()> {
    let mut is_empty = true;
    let max = iterator.do_size_hint().1;
    let mut i = 0;
    loop {
        let item = match iterator.do_next(interpreter)? {
            Some(item) => item,
            None => break,
        };
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
            .concat_recursive_into(output, behaviour, interpreter)?;
        i += 1;
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
    Array(Vec<IteratorValue>, SpanRange),
    Object(Vec<(String, Span, IteratorValue)>, SpanRange),
}

impl ZipIterators {
    pub(crate) fn new_from_object(
        object: ObjectValue,
        span_range: SpanRange,
    ) -> FunctionResult<Self> {
        let entries = object
            .entries
            .into_iter()
            .take(101)
            .map(|(k, v)| -> FunctionResult<_> {
                Ok((
                    k,
                    v.key_span,
                    v.value
                        .spanned(span_range)
                        .resolve_any_iterator("Each zip input")?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if entries.len() == 101 {
            return span_range.value_err("A maximum of 100 iterators are allowed");
        }
        Ok(ZipIterators::Object(entries, span_range))
    }

    pub(crate) fn new_from_iterator(
        iterator: IteratorValue,
        span_range: SpanRange,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<Self> {
        let vec: Vec<_> = iterator.do_take(101)
            .do_map(|x, _| x.spanned(span_range).resolve_any_iterator("Each zip input"))
            .do_collect(interpreter)?;
        if vec.len() == 101 {
            return span_range.value_err("A maximum of 100 iterators are allowed");
        }
        Ok(ZipIterators::Array(vec, span_range))
    }

    pub(crate) fn run_zip(
        self,
        interpreter: &mut Interpreter,
        error_on_length_mismatch: bool,
    ) -> FunctionResult<ArrayValue> {
        let mut iterators = self;
        let error_span_range = match &iterators {
            ZipIterators::Array(_, span_range) => *span_range,
            ZipIterators::Object(_, span_range) => *span_range,
        };
        let mut output = Vec::new();

        if iterators.len() == 0 {
            return Ok(ArrayValue::new(output));
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

        Ok(ArrayValue::new(output))
    }

    /// Panics if called on an empty list of iterators
    fn size_hint_range(&self) -> (usize, Option<usize>) {
        let size_hints: Vec<_> = match self {
            ZipIterators::Array(inner, _) => inner.iter().map(|x| x.do_size_hint()).collect(),
            ZipIterators::Object(inner, _) => inner.iter().map(|x| x.2.do_size_hint()).collect(),
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
        output: &mut Vec<AnyValue>,
    ) -> FunctionResult<()> {
        let mut counter = interpreter.start_iteration_counter(&error_span_range);

        match self {
            ZipIterators::Array(iterators, _) => {
                for _ in 0..count {
                    counter.increment_and_check()?;
                    let mut inner = Vec::with_capacity(iterators.len());
                    for iter in iterators.iter_mut() {
                        inner.push(iter.do_next(interpreter)?.unwrap());
                    }
                    output.push(inner.into_any_value());
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
                                value: iter.do_next(interpreter)?.unwrap(),
                            },
                        );
                    }
                    output.push(inner.into_any_value());
                }
            }
        }

        Ok(())
    }
}

define_optional_object! {
    pub(crate) struct IntersperseSettings {
        add_trailing: bool = false => ("false", "Whether to add the separator after the last item (default: false)"),
        final_separator: AnyValue => ("%[or]", "Define a different final separator (default: same as normal separator)"),
    }
}

pub(crate) fn run_intersperse(
    items: Box<dyn IsIterable>,
    separator: AnyValue,
    settings: IntersperseSettings,
    interpreter: &mut Interpreter,
) -> FunctionResult<ArrayValue> {
    let mut output = Vec::new();

    // Collect items eagerly since we need lookahead for separator logic.
    let mut iterator = items.into_iterator()?;
    let mut collected = Vec::new();
    while let Some(item) = iterator.do_next(interpreter)? {
        collected.push(item);
    }

    let mut collected_iter = collected.into_iter().peekable();

    let mut this_item = match collected_iter.next() {
        Some(next) => next,
        None => return Ok(ArrayValue { items: output }),
    };

    let mut appender = SeparatorAppender {
        separator,
        final_separator: settings.final_separator,
        add_trailing: settings.add_trailing,
    };

    loop {
        output.push(this_item);
        let next_item = collected_iter.next();
        match next_item {
            Some(next_item) => {
                let remaining = if collected_iter.peek().is_some() {
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

    Ok(ArrayValue { items: output })
}

struct SeparatorAppender {
    separator: AnyValue,
    final_separator: Option<AnyValue>,
    add_trailing: bool,
}

impl SeparatorAppender {
    fn add_separator(
        &mut self,
        remaining: RemainingItemCount,
        output: &mut Vec<AnyValue>,
    ) -> FunctionResult<()> {
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
) -> FunctionResult<ArrayValue> {
    input.parse_with(move |input| {
        let mut output = Vec::new();
        let mut current_item = OutputStream::new();

        // Special case separator.len() == 0 to avoid an infinite loop
        if separator.is_empty() {
            while !input.is_empty() {
                current_item.push_raw_token_tree(input.parse()?);
                let complete_item = core::mem::replace(&mut current_item, OutputStream::new());
                output.push(complete_item.into_any_value());
            }
            return Ok(ArrayValue::new(output));
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
                output.push(complete_item.into_any_value());
            }
            drop_empty_next = settings.drop_empty_middle;
        }
        if !current_item.is_empty() || !settings.drop_empty_end {
            output.push(current_item.into_any_value());
        }
        Ok(ArrayValue::new(output))
    })
}
