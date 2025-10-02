use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionRange {
    pub(crate) inner: Box<ExpressionRangeInner>,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(crate) span_range: SpanRange,
}

impl ExpressionRange {
    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        match &*self.inner {
            ExpressionRangeInner::Range {
                start_inclusive,
                end_exclusive,
                ..
            } => {
                start_inclusive.concat_recursive_into(output, behaviour)?;
                output.push_str("..");
                end_exclusive.concat_recursive_into(output, behaviour)?;
            }
            ExpressionRangeInner::RangeFrom {
                start_inclusive, ..
            } => {
                start_inclusive.concat_recursive_into(output, behaviour)?;
                output.push_str("..");
            }
            ExpressionRangeInner::RangeTo { end_exclusive, .. } => {
                output.push_str("..");
                end_exclusive.concat_recursive_into(output, behaviour)?;
            }
            ExpressionRangeInner::RangeFull { .. } => {
                output.push_str("..");
            }
            ExpressionRangeInner::RangeInclusive {
                start_inclusive,
                end_inclusive,
                ..
            } => {
                start_inclusive.concat_recursive_into(output, behaviour)?;
                output.push_str("..=");
                end_inclusive.concat_recursive_into(output, behaviour)?;
            }
            ExpressionRangeInner::RangeToInclusive { end_inclusive, .. } => {
                output.push_str("..=");
                end_inclusive.concat_recursive_into(output, behaviour)?;
            }
        }
        Ok(())
    }

    pub(crate) fn resolve_to_index_range(
        &self,
        array: &ExpressionArray,
    ) -> ExecutionResult<std::ops::Range<usize>> {
        let mut start = 0;
        let mut end = array.items.len();
        Ok(match &*self.inner {
            ExpressionRangeInner::Range {
                start_inclusive,
                end_exclusive,
                ..
            } => {
                start = array.resolve_valid_index(start_inclusive, false)?;
                end = array.resolve_valid_index(end_exclusive, true)?;
                start..end
            }
            ExpressionRangeInner::RangeFrom {
                start_inclusive, ..
            } => {
                start = array.resolve_valid_index(start_inclusive, false)?;
                start..array.items.len()
            }
            ExpressionRangeInner::RangeTo { end_exclusive, .. } => {
                end = array.resolve_valid_index(end_exclusive, true)?;
                start..end
            }
            ExpressionRangeInner::RangeFull { .. } => start..end,
            ExpressionRangeInner::RangeInclusive {
                start_inclusive,
                end_inclusive,
                ..
            } => {
                start = array.resolve_valid_index(start_inclusive, false)?;
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(end_inclusive, false)? + 1;
                start..end
            }
            ExpressionRangeInner::RangeToInclusive { end_inclusive, .. } => {
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(end_inclusive, false)? + 1;
                start..end
            }
        })
    }
}

impl HasSpanRange for ExpressionRange {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl HasValueType for ExpressionRange {
    fn value_type(&self) -> &'static str {
        self.inner.value_type()
    }
}

/// A representation of the Rust [range expression].
///
/// [range expression]: https://doc.rust-lang.org/reference/expressions/range-expr.html
#[derive(Clone)]
pub(crate) enum ExpressionRangeInner {
    /// `start .. end`
    Range {
        start_inclusive: ExpressionValue,
        token: Token![..],
        end_exclusive: ExpressionValue,
    },
    /// `start ..`
    RangeFrom {
        start_inclusive: ExpressionValue,
        token: Token![..],
    },
    /// `.. end`
    RangeTo {
        token: Token![..],
        end_exclusive: ExpressionValue,
    },
    /// `..` (used inside arrays)
    RangeFull { token: Token![..] },
    /// `start ..= end`
    RangeInclusive {
        start_inclusive: ExpressionValue,
        token: Token![..=],
        end_inclusive: ExpressionValue,
    },
    /// `..= end`
    RangeToInclusive {
        token: Token![..=],
        end_inclusive: ExpressionValue,
    },
}

impl ExpressionRangeInner {
    pub(super) fn into_iterable(self) -> ExecutionResult<IterableExpressionRange<ExpressionValue>> {
        Ok(match self {
            Self::Range {
                start_inclusive,
                token,
                end_exclusive,
            } => IterableExpressionRange::RangeFromTo {
                start: start_inclusive,
                dots: syn::RangeLimits::HalfOpen(token),
                end: end_exclusive,
            },
            Self::RangeInclusive {
                start_inclusive,
                token,
                end_inclusive,
            } => IterableExpressionRange::RangeFromTo {
                start: start_inclusive,
                dots: syn::RangeLimits::Closed(token),
                end: end_inclusive,
            },
            Self::RangeFrom {
                start_inclusive,
                token,
            } => IterableExpressionRange::RangeFrom {
                start: start_inclusive,
                dots: token,
            },
            other => {
                return other
                    .operator_span_range()
                    .execution_err("This range has no start so is not iterable")
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

impl HasValueType for ExpressionRangeInner {
    fn value_type(&self) -> &'static str {
        match self {
            Self::Range { .. } => "range start..end",
            Self::RangeFrom { .. } => "range start..",
            Self::RangeTo { .. } => "range ..end",
            Self::RangeFull { .. } => "range ..",
            Self::RangeInclusive { .. } => "range start..=end",
            Self::RangeToInclusive { .. } => "range ..=end",
        }
    }
}

impl ToExpressionValue for ExpressionRangeInner {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Range(ExpressionRange {
            inner: Box::new(self),
            span_range,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RangeTypeData;

impl MethodResolutionTarget for RangeTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);

    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        Some(match operation {
            UnaryOperation::Cast { .. }
                if IteratorTypeData::resolve_own_unary_operation(operation).is_some() =>
            {
                wrap_unary!([Op=operation](this: Owned<ExpressionRange>) -> ExecutionResult<ResolvedValue> {
                    let this_iterator = this.try_map(|this, _| ExpressionIterator::new_for_range(this))?;
                    operation.evaluate(this_iterator)
                })
            }
            _ => return None,
        })
    }
}

pub(super) enum IterableExpressionRange<T> {
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

impl IterableExpressionRange<ExpressionValue> {
    pub(super) fn resolve_iterator(self) -> ExecutionResult<Box<dyn CustomExpressionIterator>> {
        match self {
            Self::RangeFromTo { start, dots, end } => {
                let output_span_range =
                    SpanRange::new_between(start.span_range().start(), end.span_range().end());
                let pair = start.expect_value_pair(&dots, end)?;
                match pair {
                    ExpressionValuePair::Integer(pair) => match pair {
                        ExpressionIntegerValuePair::Untyped(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::U8(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::U16(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::U32(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::U64(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::U128(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::Usize(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::I8(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::I16(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::I32(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::I64(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::I128(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValuePair::Isize(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }
                                .resolve(output_span_range)
                        }
                    },
                    ExpressionValuePair::CharPair(start, end) => {
                        IterableExpressionRange::RangeFromTo {
                            start: start.value,
                            dots,
                            end: end.value,
                        }
                        .resolve(output_span_range)
                    }
                    _ => dots
                        .execution_err("The range must be between two integers or two characters"),
                }
            }
            Self::RangeFrom { start, dots } => {
                let output_span_range =
                    SpanRange::new_between(start.span_range().start(), dots.span_range().end());
                match start {
                    ExpressionValue::Integer(start) => match start.value {
                        ExpressionIntegerValue::Untyped(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::U8(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::U16(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::U32(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::U64(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::U128(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::Usize(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::I8(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::I16(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::I32(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::I64(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::I128(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                        ExpressionIntegerValue::Isize(start) => {
                            IterableExpressionRange::RangeFrom { start, dots }
                                .resolve(output_span_range)
                        }
                    },
                    ExpressionValue::Char(start) => IterableExpressionRange::RangeFrom {
                        start: start.value,
                        dots,
                    }
                    .resolve(output_span_range),
                    _ => dots.execution_err("The range must be from an integer or a character"),
                }
            }
        }
    }
}

impl IterableExpressionRange<UntypedInteger> {
    fn resolve(
        self,
        output_span_range: SpanRange,
    ) -> ExecutionResult<Box<dyn CustomExpressionIterator>> {
        match self {
            Self::RangeFromTo { start, dots, end } => {
                let start = start.parse_fallback()?;
                let end = end.parse_fallback()?;
                Ok(match dots {
                    syn::RangeLimits::HalfOpen { .. } => Box::new((start..end).map(move |x| {
                        UntypedInteger::from_fallback(x).to_value(output_span_range)
                    })),
                    syn::RangeLimits::Closed { .. } => Box::new((start..=end).map(move |x| {
                        UntypedInteger::from_fallback(x).to_value(output_span_range)
                    })),
                })
            }
            Self::RangeFrom { start, .. } => {
                let start = start.parse_fallback()?;
                Ok(Box::new((start..).map(move |x| {
                    UntypedInteger::from_fallback(x).to_value(output_span_range)
                })))
            }
        }
    }
}

macro_rules! define_range_resolvers {
    (
        $($the_type:ident),* $(,)?
    ) => {$(
        impl IterableExpressionRange<$the_type> {
            fn resolve(self, output_span_range: SpanRange) -> ExecutionResult<Box<dyn CustomExpressionIterator>> {
                match self {
                    Self::RangeFromTo { start, dots, end } => {
                        Ok(match dots {
                            syn::RangeLimits::HalfOpen { .. } => {
                                Box::new((start..end).map(move |x| x.to_value(output_span_range)))
                            }
                            syn::RangeLimits::Closed { .. } => {
                                Box::new((start..=end).map(move |x| x.to_value(output_span_range)))
                            }
                        })
                    },
                    Self::RangeFrom { start, .. } => {
                        Ok(Box::new((start..).map(move |x| x.to_value(output_span_range))))
                    },
                }
            }
        }
    )*};
}

define_range_resolvers! {
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char,
}
