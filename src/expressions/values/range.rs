use syn::RangeLimits;

use super::*;

#[derive(Clone)]
pub(crate) struct RangeValue {
    pub(crate) inner: Box<RangeValueInner>,
}

impl RangeValue {
    pub(crate) fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize> {
        IteratorValue::new_for_range(self.clone())?.len(error_span_range)
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        if !behaviour.use_debug_literal_syntax {
            return IteratorValue::any_iterator_to_string(
                self.clone().inner.into_iterable()?.resolve_iterator()?,
                output,
                behaviour,
                "[<range>]",
                "[<range> ",
                "]",
                true,
            );
        }
        match &*self.inner {
            RangeValueInner::Range {
                start_inclusive,
                end_exclusive,
                ..
            } => {
                start_inclusive.concat_recursive_into(output, behaviour)?;
                output.push_str("..");
                end_exclusive.concat_recursive_into(output, behaviour)?;
            }
            RangeValueInner::RangeFrom {
                start_inclusive, ..
            } => {
                start_inclusive.concat_recursive_into(output, behaviour)?;
                output.push_str("..");
            }
            RangeValueInner::RangeTo { end_exclusive, .. } => {
                output.push_str("..");
                end_exclusive.concat_recursive_into(output, behaviour)?;
            }
            RangeValueInner::RangeFull { .. } => {
                output.push_str("..");
            }
            RangeValueInner::RangeInclusive {
                start_inclusive,
                end_inclusive,
                ..
            } => {
                start_inclusive.concat_recursive_into(output, behaviour)?;
                output.push_str("..=");
                end_inclusive.concat_recursive_into(output, behaviour)?;
            }
            RangeValueInner::RangeToInclusive { end_inclusive, .. } => {
                output.push_str("..=");
                end_inclusive.concat_recursive_into(output, behaviour)?;
            }
        }
        Ok(())
    }
}

impl Spanned<&RangeValue> {
    pub(crate) fn resolve_to_index_range(
        self,
        array: &ArrayValue,
    ) -> ExecutionResult<std::ops::Range<usize>> {
        let (inner, span_range) = self.deconstruct();
        let mut start = 0;
        let mut end = array.items.len();
        Ok(match &*inner.inner {
            RangeValueInner::Range {
                start_inclusive,
                end_exclusive,
                ..
            } => {
                start = array.resolve_valid_index(start_inclusive.spanned(span_range), false)?;
                end = array.resolve_valid_index(end_exclusive.spanned(span_range), true)?;
                start..end
            }
            RangeValueInner::RangeFrom {
                start_inclusive, ..
            } => {
                start = array.resolve_valid_index(start_inclusive.spanned(span_range), false)?;
                start..array.items.len()
            }
            RangeValueInner::RangeTo { end_exclusive, .. } => {
                end = array.resolve_valid_index(end_exclusive.spanned(span_range), true)?;
                start..end
            }
            RangeValueInner::RangeFull { .. } => start..end,
            RangeValueInner::RangeInclusive {
                start_inclusive,
                end_inclusive,
                ..
            } => {
                start = array.resolve_valid_index(start_inclusive.spanned(span_range), false)?;
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(end_inclusive.spanned(span_range), false)? + 1;
                start..end
            }
            RangeValueInner::RangeToInclusive { end_inclusive, .. } => {
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(end_inclusive.spanned(span_range), false)? + 1;
                start..end
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RangeKind {
    /// `start .. end`
    Range,
    /// `start ..`
    RangeFrom,
    /// `.. end`
    RangeTo,
    /// `..`
    RangeFull,
    /// `start ..= end`
    RangeInclusive,
    /// `..= end`
    RangeToInclusive,
}

impl IsSpecificValueKind for RangeKind {
    fn display_name(&self) -> &'static str {
        match self {
            RangeKind::Range => "range start..end",
            RangeKind::RangeFrom => "range start..",
            RangeKind::RangeTo => "range ..end",
            RangeKind::RangeFull => "range ..",
            RangeKind::RangeInclusive => "range start..=end",
            RangeKind::RangeToInclusive => "range ..=end",
        }
    }

    fn articled_display_name(&self) -> &'static str {
        match self {
            RangeKind::Range => "a range start..end",
            RangeKind::RangeFrom => "a range start..",
            RangeKind::RangeTo => "a range ..end",
            RangeKind::RangeFull => "a range ..",
            RangeKind::RangeInclusive => "a range start..=end",
            RangeKind::RangeToInclusive => "a range ..=end",
        }
    }
}

impl From<RangeKind> for ValueKind {
    fn from(kind: RangeKind) -> Self {
        ValueKind::Range(kind)
    }
}

impl HasValueKind for RangeValue {
    type SpecificKind = RangeKind;

    fn kind(&self) -> RangeKind {
        self.inner.kind()
    }
}

/// A representation of the Rust [range expression].
///
/// [range expression]: https://doc.rust-lang.org/reference/expressions/range-expr.html
#[derive(Clone)]
pub(crate) enum RangeValueInner {
    /// `start .. end`
    Range {
        start_inclusive: Value,
        token: Token![..],
        end_exclusive: Value,
    },
    /// `start ..`
    RangeFrom {
        start_inclusive: Value,
        token: Token![..],
    },
    /// `.. end`
    RangeTo {
        token: Token![..],
        end_exclusive: Value,
    },
    /// `..` (used inside arrays)
    RangeFull { token: Token![..] },
    /// `start ..= end`
    RangeInclusive {
        start_inclusive: Value,
        token: Token![..=],
        end_inclusive: Value,
    },
    /// `..= end`
    RangeToInclusive {
        token: Token![..=],
        end_inclusive: Value,
    },
}

impl RangeValueInner {
    fn kind(&self) -> RangeKind {
        match self {
            Self::Range { .. } => RangeKind::Range,
            Self::RangeFrom { .. } => RangeKind::RangeFrom,
            Self::RangeTo { .. } => RangeKind::RangeTo,
            Self::RangeFull { .. } => RangeKind::RangeFull,
            Self::RangeInclusive { .. } => RangeKind::RangeInclusive,
            Self::RangeToInclusive { .. } => RangeKind::RangeToInclusive,
        }
    }

    pub(super) fn into_iterable(self) -> ExecutionResult<IterableRangeOf<Value>> {
        Ok(match self {
            Self::Range {
                start_inclusive,
                token,
                end_exclusive,
            } => IterableRangeOf::RangeFromTo {
                start: start_inclusive,
                dots: syn::RangeLimits::HalfOpen(token),
                end: end_exclusive,
            },
            Self::RangeInclusive {
                start_inclusive,
                token,
                end_inclusive,
            } => IterableRangeOf::RangeFromTo {
                start: start_inclusive,
                dots: syn::RangeLimits::Closed(token),
                end: end_inclusive,
            },
            Self::RangeFrom {
                start_inclusive,
                token,
            } => IterableRangeOf::RangeFrom {
                start: start_inclusive,
                dots: token,
            },
            other => {
                return other
                    .operator_span_range()
                    .value_err("This range has no start so is not iterable")
            }
        })
    }

    fn operator_span_range(&self) -> SpanRange {
        match self {
            Self::Range { token, .. } => token.span_range(),
            Self::RangeFrom { token, .. } => token.span_range(),
            Self::RangeTo { token, .. } => token.span_range(),
            Self::RangeFull { token } => token.span_range(),
            Self::RangeInclusive { token, .. } => token.span_range(),
            Self::RangeToInclusive { token, .. } => token.span_range(),
        }
    }
}

impl IntoValue for RangeValueInner {
    fn into_value(self) -> Value {
        Value::Range(RangeValue {
            inner: Box::new(self),
        })
    }
}

impl_resolvable_argument_for! {
    RangeTypeData,
    (value, context) -> RangeValue {
        match value {
            Value::Range(value) => Ok(value),
            _ => context.err("a range", value),
        }
    }
}

define_interface! {
    struct RangeTypeData,
    parent: IterableTypeData,
    pub(crate) mod range_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
            [context] fn cast_via_iterator(this: Owned<RangeValue>) -> ExecutionResult<ReturnedValue> {
                let this_iterator = this.try_map(|this, _| IteratorValue::new_for_range(this))?;
                context.operation.evaluate(this_iterator)
            }
        }
        pub(crate) mod binary_operations {}
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast { .. }
                        if IteratorTypeData::resolve_own_unary_operation(operation).is_some() =>
                    {
                        unary_definitions::cast_via_iterator()
                    }
                    _ => return None,
                })
            }
        }
    }
}

pub(super) enum IterableRangeOf<T> {
    // start <= x < end OR start <= x <= end
    RangeFromTo {
        start: T,
        dots: syn::RangeLimits,
        end: T,
    },
    // start <= x
    RangeFrom {
        dots: Token![..],
        start: T,
    },
}

fn resolve_range<T: ResolvableOwned<Value> + ResolvableRange>(
    start: T,
    dots: syn::RangeLimits,
    end: Option<OwnedValue>,
) -> ExecutionResult<Box<dyn ClonableIterator<Item = Value>>> {
    let definition = match (end, dots) {
        (Some(end), dots) => {
            let end = end.resolve_as("The end of this range bound")?;
            IterableRangeOf::RangeFromTo { start, dots, end }
        }
        (None, RangeLimits::HalfOpen(dots)) => IterableRangeOf::RangeFrom { start, dots },
        (None, RangeLimits::Closed(_)) => {
            return dots.value_err("The range '..=' requires an end value")
        }
    };
    T::resolve(definition)
}

trait ResolvableRange: Sized {
    fn resolve(
        definition: IterableRangeOf<Self>,
    ) -> ExecutionResult<Box<dyn ClonableIterator<Item = Value>>>;
}

impl IterableRangeOf<Value> {
    pub(super) fn resolve_iterator(
        self,
    ) -> ExecutionResult<Box<dyn ClonableIterator<Item = Value>>> {
        let (start, dots, end) = match self {
            Self::RangeFromTo { start, dots, end } => {
                (start, dots, Some(end.into_owned(dots.span_range())))
            }
            Self::RangeFrom { start, dots } => (start, RangeLimits::HalfOpen(dots), None),
        };
        let span_range = dots.span_range();
        match start {
            Value::Integer(mut start) => {
                if let Some(end) = &end {
                    start = IntegerValue::resolve_untyped_to_match_other(
                        start.into_owned(span_range),
                        end,
                    )?;
                }
                match start {
                    IntegerValue::Untyped(start) => resolve_range(start, dots, end),
                    IntegerValue::U8(start) => resolve_range(start, dots, end),
                    IntegerValue::U16(start) => resolve_range(start, dots, end),
                    IntegerValue::U32(start) => resolve_range(start, dots, end),
                    IntegerValue::U64(start) => resolve_range(start, dots, end),
                    IntegerValue::U128(start) => resolve_range(start, dots, end),
                    IntegerValue::Usize(start) => resolve_range(start, dots, end),
                    IntegerValue::I8(start) => resolve_range(start, dots, end),
                    IntegerValue::I16(start) => resolve_range(start, dots, end),
                    IntegerValue::I32(start) => resolve_range(start, dots, end),
                    IntegerValue::I64(start) => resolve_range(start, dots, end),
                    IntegerValue::I128(start) => resolve_range(start, dots, end),
                    IntegerValue::Isize(start) => resolve_range(start, dots, end),
                }
            }
            Value::Char(start) => resolve_range(start.value, dots, end),
            _ => dots.value_err("The range must be between two integers or two characters"),
        }
    }
}

impl ResolvableRange for UntypedInteger {
    fn resolve(
        definition: IterableRangeOf<UntypedInteger>,
    ) -> ExecutionResult<Box<dyn ClonableIterator<Item = Value>>> {
        match definition {
            IterableRangeOf::RangeFromTo { start, dots, end } => {
                let start = start.parse_fallback()?;
                let end = end.parse_fallback()?;
                Ok(match dots {
                    syn::RangeLimits::HalfOpen { .. } => Box::new(
                        (start..end).map(move |x| UntypedInteger::from_fallback(x).into_value()),
                    ),
                    syn::RangeLimits::Closed { .. } => Box::new(
                        (start..=end).map(move |x| UntypedInteger::from_fallback(x).into_value()),
                    ),
                })
            }
            IterableRangeOf::RangeFrom { start, .. } => {
                let start = start.parse_fallback()?;
                Ok(Box::new((start..).map(move |x| {
                    UntypedInteger::from_fallback(x).into_value()
                })))
            }
        }
    }
}

macro_rules! define_range_resolvers {
    (
        $($the_type:ident),* $(,)?
    ) => {$(
        impl ResolvableRange for $the_type {
            fn resolve(definition: IterableRangeOf<Self>) -> ExecutionResult<Box<dyn ClonableIterator<Item = Value>>> {
                match definition {
                    IterableRangeOf::RangeFromTo { start, dots, end } => {
                        Ok(match dots {
                            syn::RangeLimits::HalfOpen { .. } => {
                                Box::new((start..end).map(move |x| x.into_value()))
                            }
                            syn::RangeLimits::Closed { .. } => {
                                Box::new((start..=end).map(move |x| x.into_value()))
                            }
                        })
                    },
                    IterableRangeOf::RangeFrom { start, .. } => {
                        Ok(Box::new((start..).map(move |x| x.into_value())))
                    },
                }
            }
        }
    )*};
}

define_range_resolvers! {
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char,
}
