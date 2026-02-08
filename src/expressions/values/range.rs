use syn::RangeLimits;

use super::*;

define_leaf_type! {
    pub(crate) RangeType => AnyType(AnyValueContent::Range),
    content: RangeValue,
    kind: pub(crate) RangeKind,
    type_name: "range",
    articled_value_name: "a range",
    dyn_impls: {
        IterableType: impl IsIterable {
            fn into_iterator(self: Box<Self>) -> FunctionResult<IteratorValue> {
                IteratorValue::new_for_range(*self)
            }

            fn iterable_len(&self, error_span_range: SpanRange) -> FunctionResult<usize> {
                self.len(error_span_range)
            }
        }
    },
}

#[derive(Clone)]
pub(crate) struct RangeValue {
    pub(crate) inner: Box<RangeValueInner>,
}

impl RangeValue {
    pub(crate) fn len(&self, error_span_range: SpanRange) -> FunctionResult<usize> {
        IteratorValue::new_for_range(self.clone())?.do_len(error_span_range)
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<()> {
        if !behaviour.use_debug_literal_syntax {
            let mut iter = IteratorValue::new_for_range(self.clone())?;
            return any_items_to_string(
                &mut iter,
                output,
                behaviour,
                "[<range>]",
                "[<range> ",
                "]",
                true,
                interpreter,
            );
        }
        match &*self.inner {
            RangeValueInner::Range {
                start_inclusive,
                end_exclusive,
                ..
            } => {
                start_inclusive.as_ref_value().concat_recursive_into(
                    output,
                    behaviour,
                    interpreter,
                )?;
                output.push_str("..");
                end_exclusive.as_ref_value().concat_recursive_into(
                    output,
                    behaviour,
                    interpreter,
                )?;
            }
            RangeValueInner::RangeFrom {
                start_inclusive, ..
            } => {
                start_inclusive.as_ref_value().concat_recursive_into(
                    output,
                    behaviour,
                    interpreter,
                )?;
                output.push_str("..");
            }
            RangeValueInner::RangeTo { end_exclusive, .. } => {
                output.push_str("..");
                end_exclusive.as_ref_value().concat_recursive_into(
                    output,
                    behaviour,
                    interpreter,
                )?;
            }
            RangeValueInner::RangeFull { .. } => {
                output.push_str("..");
            }
            RangeValueInner::RangeInclusive {
                start_inclusive,
                end_inclusive,
                ..
            } => {
                start_inclusive.as_ref_value().concat_recursive_into(
                    output,
                    behaviour,
                    interpreter,
                )?;
                output.push_str("..=");
                end_inclusive.as_ref_value().concat_recursive_into(
                    output,
                    behaviour,
                    interpreter,
                )?;
            }
            RangeValueInner::RangeToInclusive { end_inclusive, .. } => {
                output.push_str("..=");
                end_inclusive.as_ref_value().concat_recursive_into(
                    output,
                    behaviour,
                    interpreter,
                )?;
            }
        }
        Ok(())
    }
}

impl Spanned<&RangeValue> {
    pub(crate) fn resolve_to_index_range(
        self,
        array: &ArrayValue,
    ) -> FunctionResult<std::ops::Range<usize>> {
        let Spanned(value, span_range) = self;
        let mut start = 0;
        let mut end = array.items.len();
        Ok(match &*value.inner {
            RangeValueInner::Range {
                start_inclusive,
                end_exclusive,
                ..
            } => {
                start = array.resolve_valid_index(
                    Spanned(start_inclusive.as_ref_value(), span_range),
                    false,
                )?;
                end = array
                    .resolve_valid_index(Spanned(end_exclusive.as_ref_value(), span_range), true)?;
                start..end
            }
            RangeValueInner::RangeFrom {
                start_inclusive, ..
            } => {
                start = array.resolve_valid_index(
                    Spanned(start_inclusive.as_ref_value(), span_range),
                    false,
                )?;
                start..array.items.len()
            }
            RangeValueInner::RangeTo { end_exclusive, .. } => {
                end = array
                    .resolve_valid_index(Spanned(end_exclusive.as_ref_value(), span_range), true)?;
                start..end
            }
            RangeValueInner::RangeFull { .. } => start..end,
            RangeValueInner::RangeInclusive {
                start_inclusive,
                end_inclusive,
                ..
            } => {
                start = array.resolve_valid_index(
                    Spanned(start_inclusive.as_ref_value(), span_range),
                    false,
                )?;
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(
                    Spanned(end_inclusive.as_ref_value(), span_range),
                    false,
                )? + 1;
                start..end
            }
            RangeValueInner::RangeToInclusive { end_inclusive, .. } => {
                // +1 is safe because it must be < array length.
                end = array.resolve_valid_index(
                    Spanned(end_inclusive.as_ref_value(), span_range),
                    false,
                )? + 1;
                start..end
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RangeStructure {
    /// `start .. end`
    FromTo,
    /// `start ..`
    From,
    /// `.. end`
    To,
    /// `..`
    Full,
    /// `start ..= end`
    FromToInclusive,
    /// `..= end`
    ToInclusive,
}

impl RangeStructure {
    pub(crate) fn articled_value_name(&self) -> &'static str {
        match self {
            RangeStructure::FromTo => "a range start..end",
            RangeStructure::From => "a range start..",
            RangeStructure::To => "a range ..end",
            RangeStructure::Full => "a range ..",
            RangeStructure::FromToInclusive => "a range start..=end",
            RangeStructure::ToInclusive => "a range ..=end",
        }
    }
}

impl ValuesEqual for RangeValue {
    /// Ranges are equal if they have the same kind and the same bounds.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        match (&*self.inner, &*other.inner) {
            (
                RangeValueInner::Range {
                    start_inclusive: l_start,
                    end_exclusive: l_end,
                    ..
                },
                RangeValueInner::Range {
                    start_inclusive: r_start,
                    end_exclusive: r_end,
                    ..
                },
            ) => {
                let result = ctx.with_range_start(|ctx| l_start.test_equality(r_start, ctx));
                if ctx.should_short_circuit(&result) {
                    return result;
                }
                ctx.with_range_end(|ctx| l_end.test_equality(r_end, ctx))
            }
            (
                RangeValueInner::RangeFrom {
                    start_inclusive: l_start,
                    ..
                },
                RangeValueInner::RangeFrom {
                    start_inclusive: r_start,
                    ..
                },
            ) => ctx.with_range_start(|ctx| l_start.test_equality(r_start, ctx)),
            (
                RangeValueInner::RangeTo {
                    end_exclusive: l_end,
                    ..
                },
                RangeValueInner::RangeTo {
                    end_exclusive: r_end,
                    ..
                },
            ) => ctx.with_range_end(|ctx| l_end.test_equality(r_end, ctx)),
            (RangeValueInner::RangeFull { .. }, RangeValueInner::RangeFull { .. }) => {
                ctx.values_equal()
            }
            (
                RangeValueInner::RangeInclusive {
                    start_inclusive: l_start,
                    end_inclusive: l_end,
                    ..
                },
                RangeValueInner::RangeInclusive {
                    start_inclusive: r_start,
                    end_inclusive: r_end,
                    ..
                },
            ) => {
                let result = ctx.with_range_start(|ctx| l_start.test_equality(r_start, ctx));
                if ctx.should_short_circuit(&result) {
                    return result;
                }
                ctx.with_range_end(|ctx| l_end.test_equality(r_end, ctx))
            }
            (
                RangeValueInner::RangeToInclusive {
                    end_inclusive: l_end,
                    ..
                },
                RangeValueInner::RangeToInclusive {
                    end_inclusive: r_end,
                    ..
                },
            ) => ctx.with_range_end(|ctx| l_end.test_equality(r_end, ctx)),
            _ => ctx.range_structure_mismatch(
                self.inner.structure_kind(),
                other.inner.structure_kind(),
            ),
        }
    }
}

/// A representation of the Rust [range expression].
///
/// [range expression]: https://doc.rust-lang.org/reference/expressions/range-expr.html
#[derive(Clone)]
pub(crate) enum RangeValueInner {
    /// `start .. end`
    Range {
        start_inclusive: AnyValue,
        token: Token![..],
        end_exclusive: AnyValue,
    },
    /// `start ..`
    RangeFrom {
        start_inclusive: AnyValue,
        token: Token![..],
    },
    /// `.. end`
    RangeTo {
        token: Token![..],
        end_exclusive: AnyValue,
    },
    /// `..` (used inside arrays)
    RangeFull { token: Token![..] },
    /// `start ..= end`
    RangeInclusive {
        start_inclusive: AnyValue,
        token: Token![..=],
        end_inclusive: AnyValue,
    },
    /// `..= end`
    RangeToInclusive {
        token: Token![..=],
        end_inclusive: AnyValue,
    },
}

impl RangeValueInner {
    fn structure_kind(&self) -> RangeStructure {
        match self {
            Self::Range { .. } => RangeStructure::FromTo,
            Self::RangeFrom { .. } => RangeStructure::From,
            Self::RangeTo { .. } => RangeStructure::To,
            Self::RangeFull { .. } => RangeStructure::Full,
            Self::RangeInclusive { .. } => RangeStructure::FromToInclusive,
            Self::RangeToInclusive { .. } => RangeStructure::ToInclusive,
        }
    }

    pub(super) fn into_iterable(self) -> FunctionResult<IterableRangeOf<AnyValue>> {
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

impl IsValueContent for RangeValueInner {
    type Type = RangeType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for RangeValueInner {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        RangeValue {
            inner: Box::new(self),
        }
    }
}

impl_resolvable_argument_for! {
    RangeType,
    (value, context) -> RangeValue {
        match value {
            AnyValue::Range(value) => Ok(value),
            _ => context.err("a range", value),
        }
    }
}

define_type_features! {
    impl RangeType,
    pub(crate) mod range_interface {
        unary_operations {
            [context] fn cast_via_iterator(Spanned(this, span): Spanned<RangeValue>) -> FunctionResult<ReturnedValue> {
                let this_iterator = IteratorValue::new_for_range(this)?;
                Ok(context.operation.evaluate(Spanned(this_iterator, span), context.interpreter)?.0)
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast { .. }
                        if IteratorType::resolve_own_unary_operation(operation).is_some() =>
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

fn resolve_range<T: ResolvableOwned<AnyValue> + ResolvableRange>(
    start: T,
    dots: syn::RangeLimits,
    end: Option<Spanned<AnyValue>>,
) -> FunctionResult<ValueIterator> {
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
    fn resolve(definition: IterableRangeOf<Self>) -> FunctionResult<ValueIterator>;
}

impl IterableRangeOf<AnyValue> {
    pub(super) fn resolve_iterator(self) -> FunctionResult<ValueIterator> {
        let (start, dots, end) = match self {
            Self::RangeFromTo { start, dots, end } => {
                (start, dots, Some(end.spanned(dots.span_range())))
            }
            Self::RangeFrom { start, dots } => (start, RangeLimits::HalfOpen(dots), None),
        };
        match start {
            AnyValue::Integer(mut start) => {
                if let Some(end) = &end {
                    start = IntegerValue::resolve_untyped_to_match_other(
                        start.spanned(dots.span_range()),
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
            AnyValue::Char(start) => resolve_range(start, dots, end),
            _ => dots.value_err("The range must be between two integers or two characters"),
        }
    }
}

impl ResolvableRange for UntypedInteger {
    fn resolve(definition: IterableRangeOf<UntypedInteger>) -> FunctionResult<ValueIterator> {
        match definition {
            IterableRangeOf::RangeFromTo { start, dots, end } => {
                let start = start.into_fallback();
                let end = end.into_fallback();
                Ok(match dots {
                    syn::RangeLimits::HalfOpen { .. } => Box::new(
                        (start..end)
                            .map(move |x| UntypedInteger::from_fallback(x).into_any_value()),
                    ),
                    syn::RangeLimits::Closed { .. } => Box::new(
                        (start..=end)
                            .map(move |x| UntypedInteger::from_fallback(x).into_any_value()),
                    ),
                })
            }
            IterableRangeOf::RangeFrom { start, .. } => {
                let start = start.into_fallback();
                Ok(Box::new((start..).map(move |x| {
                    UntypedInteger::from_fallback(x).into_any_value()
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
            fn resolve(definition: IterableRangeOf<Self>) -> FunctionResult<ValueIterator> {
                match definition {
                    IterableRangeOf::RangeFromTo { start, dots, end } => {
                        Ok(match dots {
                            syn::RangeLimits::HalfOpen { .. } => {
                                Box::new((start..end).map(move |x| x.into_any_value()))
                            }
                            syn::RangeLimits::Closed { .. } => {
                                Box::new((start..=end).map(move |x| x.into_any_value()))
                            }
                        })
                    },
                    IterableRangeOf::RangeFrom { start, .. } => {
                        Ok(Box::new((start..).map(move |x| x.into_any_value())))
                    },
                }
            }
        }
    )*};
}

define_range_resolvers! {
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char,
}
