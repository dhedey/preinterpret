use super::*;

#[derive(Clone)]
pub(crate) struct RangeExpression {
    pub(crate) inner: Box<ExpressionRangeInner>,
}

impl RangeExpression {
    pub(crate) fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize> {
        IteratorExpression::new_for_range(self.clone())?.len(error_span_range)
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        if !behaviour.use_debug_literal_syntax {
            return IteratorExpression::any_iterator_to_string(
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
}

impl Spanned<&RangeExpression> {
    pub(crate) fn resolve_to_index_range(
        self,
        array: &ArrayExpression,
    ) -> ExecutionResult<std::ops::Range<usize>> {
        let (inner, span_range) = self.deconstruct();
        let mut start = 0;
        let mut end = array.items.len();
        Ok(match &*inner.inner {
            ExpressionRangeInner::Range {
                start_inclusive,
                end_exclusive,
                ..
            } => {
                start = array.resolve_valid_index(start_inclusive.spanned(span_range), false)?;
                end = array.resolve_valid_index(end_exclusive.spanned(span_range), true)?;
                start..end
            }
            ExpressionRangeInner::RangeFrom {
                start_inclusive, ..
            } => {
                start = array.resolve_valid_index(start_inclusive.spanned(span_range), false)?;
                start..array.items.len()
            }
            ExpressionRangeInner::RangeTo { end_exclusive, .. } => {
                end = array.resolve_valid_index(end_exclusive.spanned(span_range), true)?;
                start..end
            }
            ExpressionRangeInner::RangeFull { .. } => start..end,
            ExpressionRangeInner::RangeInclusive {
                start_inclusive,
                end_inclusive,
                ..
            } => {
                start = array.resolve_valid_index(start_inclusive.spanned(span_range), false)?;
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(end_inclusive.spanned(span_range), false)? + 1;
                start..end
            }
            ExpressionRangeInner::RangeToInclusive { end_inclusive, .. } => {
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(end_inclusive.spanned(span_range), false)? + 1;
                start..end
            }
        })
    }
}

impl HasValueType for RangeExpression {
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
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Range(RangeExpression {
            inner: Box::new(self),
        })
    }
}

define_interface! {
    struct RangeTypeData,
    parent: IterableTypeData,
    pub(crate) mod range_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
            [context] fn cast_via_iterator(this: Owned<RangeExpression>) -> ExecutionResult<ResolvedValue> {
                let this_iterator = this.try_map(|this, _| IteratorExpression::new_for_range(this))?;
                context.operation.evaluate(this_iterator)
            }
        }
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
    pub(super) fn resolve_iterator(
        self,
    ) -> ExecutionResult<Box<dyn ClonableIterator<Item = ExpressionValue>>> {
        match self {
            Self::RangeFromTo { start, dots, end } => {
                let pair = start.expect_value_pair(&dots, end)?;
                match pair {
                    ExpressionValuePair::Integer(pair) => match pair {
                        IntegerExpressionValuePair::Untyped(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::U8(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::U16(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::U32(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::U64(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::U128(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::Usize(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::I8(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::I16(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::I32(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::I64(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::I128(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                        IntegerExpressionValuePair::Isize(start, end) => {
                            IterableExpressionRange::RangeFromTo { start, dots, end }.resolve()
                        }
                    },
                    ExpressionValuePair::CharPair(start, end) => {
                        IterableExpressionRange::RangeFromTo {
                            start: start.value,
                            dots,
                            end: end.value,
                        }
                        .resolve()
                    }
                    _ => dots.value_err("The range must be between two integers or two characters"),
                }
            }
            Self::RangeFrom { start, dots } => match start {
                ExpressionValue::Integer(start) => match start.value {
                    IntegerExpressionValue::Untyped(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::U8(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::U16(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::U32(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::U64(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::U128(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::Usize(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::I8(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::I16(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::I32(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::I64(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::I128(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                    IntegerExpressionValue::Isize(start) => {
                        IterableExpressionRange::RangeFrom { start, dots }.resolve()
                    }
                },
                ExpressionValue::Char(start) => IterableExpressionRange::RangeFrom {
                    start: start.value,
                    dots,
                }
                .resolve(),
                _ => dots.type_err("The range must be from an integer or a character"),
            },
        }
    }
}

impl IterableExpressionRange<UntypedInteger> {
    fn resolve(self) -> ExecutionResult<Box<dyn ClonableIterator<Item = ExpressionValue>>> {
        match self {
            Self::RangeFromTo { start, dots, end } => {
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
            Self::RangeFrom { start, .. } => {
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
        impl IterableExpressionRange<$the_type> {
            fn resolve(self) -> ExecutionResult<Box<dyn ClonableIterator<Item = ExpressionValue>>> {
                match self {
                    Self::RangeFromTo { start, dots, end } => {
                        Ok(match dots {
                            syn::RangeLimits::HalfOpen { .. } => {
                                Box::new((start..end).map(move |x| x.into_value()))
                            }
                            syn::RangeLimits::Closed { .. } => {
                                Box::new((start..=end).map(move |x| x.into_value()))
                            }
                        })
                    },
                    Self::RangeFrom { start, .. } => {
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
