use super::*;

#[derive(Copy, Clone)]
pub(crate) enum IntegerValue {
    Untyped(UntypedInteger),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
}

impl IntoValue for IntegerValue {
    fn into_value(self) -> Value {
        Value::Integer(self)
    }
}

impl IntegerValue {
    pub(super) fn for_litint(lit: &syn::LitInt) -> ParseResult<Owned<Self>> {
        Ok(match lit.suffix() {
            "" => Self::Untyped(UntypedInteger::new_from_lit_int(lit)?),
            "u8" => Self::U8(lit.base10_parse()?),
            "u16" => Self::U16(lit.base10_parse()?),
            "u32" => Self::U32(lit.base10_parse()?),
            "u64" => Self::U64(lit.base10_parse()?),
            "u128" => Self::U128(lit.base10_parse()?),
            "usize" => Self::Usize(lit.base10_parse()?),
            "i8" => Self::I8(lit.base10_parse()?),
            "i16" => Self::I16(lit.base10_parse()?),
            "i32" => Self::I32(lit.base10_parse()?),
            "i64" => Self::I64(lit.base10_parse()?),
            "i128" => Self::I128(lit.base10_parse()?),
            "isize" => Self::Isize(lit.base10_parse()?),
            suffix => {
                return lit.span().parse_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        }
        .into_owned())
    }

    pub(super) fn to_literal(self, span: Span) -> Literal {
        self.to_unspanned_literal().with_span(span)
    }

    pub(crate) fn resolve_untyped_to_match_other(
        this: Owned<IntegerValue>,
        other: &Value,
        span: SpanRange,
    ) -> ExecutionResult<Self> {
        let value = this.into_inner();
        match (value, other) {
            (IntegerValue::Untyped(this), Value::Integer(other)) => {
                this.into_kind(other.kind(), span)
            }
            (value, _) => Ok(value),
        }
    }

    pub(crate) fn resolve_untyped_to_match(
        this: Owned<IntegerValue>,
        target: &IntegerValue,
        span: SpanRange,
    ) -> ExecutionResult<Self> {
        let value = this.into_inner();
        match value {
            IntegerValue::Untyped(this) => this.into_kind(target.kind(), span),
            other => Ok(other),
        }
    }

    pub(crate) fn assign_op<R>(
        mut left: Assignee<IntegerValue>,
        right: R,
        context: BinaryOperationCallContext,
        op: fn(BinaryOperationCallContext, Owned<IntegerValue>, R) -> ExecutionResult<IntegerValue>,
    ) -> ExecutionResult<()> {
        let left_value = core::mem::replace(&mut *left, IntegerValue::U32(0));
        let result = op(context, left_value.into_owned(), right)?;
        *left = result;
        Ok(())
    }

    fn to_unspanned_literal(self) -> Literal {
        match self {
            IntegerValue::Untyped(int) => int.to_unspanned_literal(),
            IntegerValue::U8(int) => Literal::u8_suffixed(int),
            IntegerValue::U16(int) => Literal::u16_suffixed(int),
            IntegerValue::U32(int) => Literal::u32_suffixed(int),
            IntegerValue::U64(int) => Literal::u64_suffixed(int),
            IntegerValue::U128(int) => Literal::u128_suffixed(int),
            IntegerValue::Usize(int) => Literal::usize_suffixed(int),
            IntegerValue::I8(int) => Literal::i8_suffixed(int),
            IntegerValue::I16(int) => Literal::i16_suffixed(int),
            IntegerValue::I32(int) => Literal::i32_suffixed(int),
            IntegerValue::I64(int) => Literal::i64_suffixed(int),
            IntegerValue::I128(int) => Literal::i128_suffixed(int),
            IntegerValue::Isize(int) => Literal::isize_suffixed(int),
        }
    }
}

impl HasValueKind for IntegerValue {
    type SpecificKind = IntegerKind;

    fn kind(&self) -> IntegerKind {
        match self {
            Self::Untyped(_) => IntegerKind::Untyped,
            Self::U8(_) => IntegerKind::U8,
            Self::U16(_) => IntegerKind::U16,
            Self::U32(_) => IntegerKind::U32,
            Self::U64(_) => IntegerKind::U64,
            Self::U128(_) => IntegerKind::U128,
            Self::Usize(_) => IntegerKind::Usize,
            Self::I8(_) => IntegerKind::I8,
            Self::I16(_) => IntegerKind::I16,
            Self::I32(_) => IntegerKind::I32,
            Self::I64(_) => IntegerKind::I64,
            Self::I128(_) => IntegerKind::I128,
            Self::Isize(_) => IntegerKind::Isize,
        }
    }
}

impl Debug for IntegerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Untyped(v) => write!(f, "{}", v.into_fallback()),
            Self::U8(v) => write!(f, "{:?}", v),
            Self::U16(v) => write!(f, "{:?}", v),
            Self::U32(v) => write!(f, "{:?}", v),
            Self::U64(v) => write!(f, "{:?}", v),
            Self::U128(v) => write!(f, "{:?}", v),
            Self::Usize(v) => write!(f, "{:?}", v),
            Self::I8(v) => write!(f, "{:?}", v),
            Self::I16(v) => write!(f, "{:?}", v),
            Self::I32(v) => write!(f, "{:?}", v),
            Self::I64(v) => write!(f, "{:?}", v),
            Self::I128(v) => write!(f, "{:?}", v),
            Self::Isize(v) => write!(f, "{:?}", v),
        }
    }
}

impl IntegerValue {
    /// Aligns types for comparison - converts untyped to match the other's type.
    /// Returns `None` if the untyped value doesn't fit in the target type.
    fn align_types(mut lhs: Self, mut rhs: Self) -> Option<(Self, Self)> {
        match (&lhs, &rhs) {
            (IntegerValue::Untyped(l), typed) if !matches!(typed, IntegerValue::Untyped(_)) => {
                lhs = l.try_into_kind(typed.kind())?;
            }
            (typed, IntegerValue::Untyped(r)) if !matches!(typed, IntegerValue::Untyped(_)) => {
                rhs = r.try_into_kind(lhs.kind())?;
            }
            _ => {} // Both same type or both untyped - no conversion needed
        }
        Some((lhs, rhs))
    }
}

impl ValuesEqual for IntegerValue {
    /// Handles type coercion between typed and untyped integers.
    /// E.g., `5 == 5u32` returns true.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        // Align types (untyped -> typed conversion)
        let Some((lhs, rhs)) = Self::align_types(*self, *other) else {
            return ctx.leaf_values_not_equal(self, other);
        };

        // After alignment, compare directly.
        // Each variant has two lines: same-type comparison, then type-mismatch fallback.
        // This ensures adding a new variant causes a compiler error.
        let equal = match (lhs, rhs) {
            (IntegerValue::Untyped(l), IntegerValue::Untyped(r)) => {
                l.into_fallback() == r.into_fallback()
            }
            (IntegerValue::Untyped(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::U8(l), IntegerValue::U8(r)) => l == r,
            (IntegerValue::U8(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::U16(l), IntegerValue::U16(r)) => l == r,
            (IntegerValue::U16(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::U32(l), IntegerValue::U32(r)) => l == r,
            (IntegerValue::U32(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::U64(l), IntegerValue::U64(r)) => l == r,
            (IntegerValue::U64(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::U128(l), IntegerValue::U128(r)) => l == r,
            (IntegerValue::U128(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::Usize(l), IntegerValue::Usize(r)) => l == r,
            (IntegerValue::Usize(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::I8(l), IntegerValue::I8(r)) => l == r,
            (IntegerValue::I8(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::I16(l), IntegerValue::I16(r)) => l == r,
            (IntegerValue::I16(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::I32(l), IntegerValue::I32(r)) => l == r,
            (IntegerValue::I32(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::I64(l), IntegerValue::I64(r)) => l == r,
            (IntegerValue::I64(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::I128(l), IntegerValue::I128(r)) => l == r,
            (IntegerValue::I128(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerValue::Isize(l), IntegerValue::Isize(r)) => l == r,
            (IntegerValue::Isize(_), _) => return ctx.leaf_values_not_equal(self, other),
        };

        if equal {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

define_interface! {
    struct IntegerTypeData,
    parent: ValueTypeData,
    pub(crate) mod integer_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {
            [context] fn add(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_add),
                    IntegerValue::U8(left) => left.paired_operation(right, context, u8::checked_add),
                    IntegerValue::U16(left) => left.paired_operation(right, context, u16::checked_add),
                    IntegerValue::U32(left) => left.paired_operation(right, context, u32::checked_add),
                    IntegerValue::U64(left) => left.paired_operation(right, context, u64::checked_add),
                    IntegerValue::U128(left) => left.paired_operation(right, context, u128::checked_add),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, usize::checked_add),
                    IntegerValue::I8(left) => left.paired_operation(right, context, i8::checked_add),
                    IntegerValue::I16(left) => left.paired_operation(right, context, i16::checked_add),
                    IntegerValue::I32(left) => left.paired_operation(right, context, i32::checked_add),
                    IntegerValue::I64(left) => left.paired_operation(right, context, i64::checked_add),
                    IntegerValue::I128(left) => left.paired_operation(right, context, i128::checked_add),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, isize::checked_add),
                }
            }

            [context] fn add_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, add)
            }

            [context] fn sub(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_sub),
                    IntegerValue::U8(left) => left.paired_operation(right, context, u8::checked_sub),
                    IntegerValue::U16(left) => left.paired_operation(right, context, u16::checked_sub),
                    IntegerValue::U32(left) => left.paired_operation(right, context, u32::checked_sub),
                    IntegerValue::U64(left) => left.paired_operation(right, context, u64::checked_sub),
                    IntegerValue::U128(left) => left.paired_operation(right, context, u128::checked_sub),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, usize::checked_sub),
                    IntegerValue::I8(left) => left.paired_operation(right, context, i8::checked_sub),
                    IntegerValue::I16(left) => left.paired_operation(right, context, i16::checked_sub),
                    IntegerValue::I32(left) => left.paired_operation(right, context, i32::checked_sub),
                    IntegerValue::I64(left) => left.paired_operation(right, context, i64::checked_sub),
                    IntegerValue::I128(left) => left.paired_operation(right, context, i128::checked_sub),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, isize::checked_sub),
                }
            }

            [context] fn sub_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, sub)
            }

            [context] fn mul(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_mul),
                    IntegerValue::U8(left) => left.paired_operation(right, context, u8::checked_mul),
                    IntegerValue::U16(left) => left.paired_operation(right, context, u16::checked_mul),
                    IntegerValue::U32(left) => left.paired_operation(right, context, u32::checked_mul),
                    IntegerValue::U64(left) => left.paired_operation(right, context, u64::checked_mul),
                    IntegerValue::U128(left) => left.paired_operation(right, context, u128::checked_mul),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, usize::checked_mul),
                    IntegerValue::I8(left) => left.paired_operation(right, context, i8::checked_mul),
                    IntegerValue::I16(left) => left.paired_operation(right, context, i16::checked_mul),
                    IntegerValue::I32(left) => left.paired_operation(right, context, i32::checked_mul),
                    IntegerValue::I64(left) => left.paired_operation(right, context, i64::checked_mul),
                    IntegerValue::I128(left) => left.paired_operation(right, context, i128::checked_mul),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, isize::checked_mul),
                }
            }

            [context] fn mul_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, mul)
            }

            [context] fn div(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_div),
                    IntegerValue::U8(left) => left.paired_operation(right, context, u8::checked_div),
                    IntegerValue::U16(left) => left.paired_operation(right, context, u16::checked_div),
                    IntegerValue::U32(left) => left.paired_operation(right, context, u32::checked_div),
                    IntegerValue::U64(left) => left.paired_operation(right, context, u64::checked_div),
                    IntegerValue::U128(left) => left.paired_operation(right, context, u128::checked_div),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, usize::checked_div),
                    IntegerValue::I8(left) => left.paired_operation(right, context, i8::checked_div),
                    IntegerValue::I16(left) => left.paired_operation(right, context, i16::checked_div),
                    IntegerValue::I32(left) => left.paired_operation(right, context, i32::checked_div),
                    IntegerValue::I64(left) => left.paired_operation(right, context, i64::checked_div),
                    IntegerValue::I128(left) => left.paired_operation(right, context, i128::checked_div),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, isize::checked_div),
                }
            }

            [context] fn div_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, div)
            }

            [context] fn rem(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_rem),
                    IntegerValue::U8(left) => left.paired_operation(right, context, u8::checked_rem),
                    IntegerValue::U16(left) => left.paired_operation(right, context, u16::checked_rem),
                    IntegerValue::U32(left) => left.paired_operation(right, context, u32::checked_rem),
                    IntegerValue::U64(left) => left.paired_operation(right, context, u64::checked_rem),
                    IntegerValue::U128(left) => left.paired_operation(right, context, u128::checked_rem),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, usize::checked_rem),
                    IntegerValue::I8(left) => left.paired_operation(right, context, i8::checked_rem),
                    IntegerValue::I16(left) => left.paired_operation(right, context, i16::checked_rem),
                    IntegerValue::I32(left) => left.paired_operation(right, context, i32::checked_rem),
                    IntegerValue::I64(left) => left.paired_operation(right, context, i64::checked_rem),
                    IntegerValue::I128(left) => left.paired_operation(right, context, i128::checked_rem),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, isize::checked_rem),
                }
            }

            [context] fn rem_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, rem)
            }

            [context] fn bitxor(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::U8(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::U16(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::U32(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::U64(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::U128(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::I8(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::I16(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::I32(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::I64(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::I128(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                }
            }

            [context] fn bitxor_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, bitxor)
            }

            [context] fn bitand(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::U8(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::U16(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::U32(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::U64(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::U128(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::I8(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::I16(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::I32(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::I64(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::I128(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                }
            }

            [context] fn bitand_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, bitand)
            }

            [context] fn bitor(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::U8(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::U16(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::U32(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::U64(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::U128(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::Usize(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::I8(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::I16(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::I32(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::I64(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::I128(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerValue::Isize(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                }
            }

            [context] fn bitor_assign(lhs: Assignee<IntegerValue>, rhs: Owned<IntegerValue>) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, bitor)
            }

            [context] fn shift_left(lhs: Owned<IntegerValue>, right: CoercedToU32) -> ExecutionResult<IntegerValue> {
                let CoercedToU32(right) = right;
                match lhs.into_inner() {
                    IntegerValue::Untyped(left) => left.shift_operation(right, context, FallbackInteger::checked_shl),
                    IntegerValue::U8(left) => left.shift_operation(right, context, u8::checked_shl),
                    IntegerValue::U16(left) => left.shift_operation(right, context, u16::checked_shl),
                    IntegerValue::U32(left) => left.shift_operation(right, context, u32::checked_shl),
                    IntegerValue::U64(left) => left.shift_operation(right, context, u64::checked_shl),
                    IntegerValue::U128(left) => left.shift_operation(right, context, u128::checked_shl),
                    IntegerValue::Usize(left) => left.shift_operation(right, context, usize::checked_shl),
                    IntegerValue::I8(left) => left.shift_operation(right, context, i8::checked_shl),
                    IntegerValue::I16(left) => left.shift_operation(right, context, i16::checked_shl),
                    IntegerValue::I32(left) => left.shift_operation(right, context, i32::checked_shl),
                    IntegerValue::I64(left) => left.shift_operation(right, context, i64::checked_shl),
                    IntegerValue::I128(left) => left.shift_operation(right, context, i128::checked_shl),
                    IntegerValue::Isize(left) => left.shift_operation(right, context, isize::checked_shl),
                }
            }

            [context] fn shift_left_assign(lhs: Assignee<IntegerValue>, rhs: CoercedToU32) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, shift_left)
            }

            [context] fn shift_right(lhs: Owned<IntegerValue>, right: CoercedToU32) -> ExecutionResult<IntegerValue> {
                let CoercedToU32(right) = right;
                match lhs.into_inner() {
                    IntegerValue::Untyped(left) => left.shift_operation(right, context, FallbackInteger::checked_shr),
                    IntegerValue::U8(left) => left.shift_operation(right, context, u8::checked_shr),
                    IntegerValue::U16(left) => left.shift_operation(right, context, u16::checked_shr),
                    IntegerValue::U32(left) => left.shift_operation(right, context, u32::checked_shr),
                    IntegerValue::U64(left) => left.shift_operation(right, context, u64::checked_shr),
                    IntegerValue::U128(left) => left.shift_operation(right, context, u128::checked_shr),
                    IntegerValue::Usize(left) => left.shift_operation(right, context, usize::checked_shr),
                    IntegerValue::I8(left) => left.shift_operation(right, context, i8::checked_shr),
                    IntegerValue::I16(left) => left.shift_operation(right, context, i16::checked_shr),
                    IntegerValue::I32(left) => left.shift_operation(right, context, i32::checked_shr),
                    IntegerValue::I64(left) => left.shift_operation(right, context, i64::checked_shr),
                    IntegerValue::I128(left) => left.shift_operation(right, context, i128::checked_shr),
                    IntegerValue::Isize(left) => left.shift_operation(right, context, isize::checked_shr),
                }
            }

            [context] fn shift_right_assign(lhs: Assignee<IntegerValue>, rhs: CoercedToU32) -> ExecutionResult<()> {
                IntegerValue::assign_op(lhs, rhs, context, shift_right)
            }

            [context] fn lt(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::U8(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::U16(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::U32(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::U64(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::U128(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::Usize(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::I8(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::I16(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::I32(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::I64(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::I128(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerValue::Isize(left) => left.paired_comparison(right, |a, b| a < b),
                }
            }

            [context] fn le(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::U8(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::U16(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::U32(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::U64(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::U128(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::Usize(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::I8(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::I16(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::I32(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::I64(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::I128(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerValue::Isize(left) => left.paired_comparison(right, |a, b| a <= b),
                }
            }

            [context] fn gt(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::U8(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::U16(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::U32(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::U64(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::U128(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::Usize(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::I8(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::I16(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::I32(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::I64(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::I128(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerValue::Isize(left) => left.paired_comparison(right, |a, b| a > b),
                }
            }

            [context] fn ge(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::U8(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::U16(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::U32(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::U64(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::U128(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::Usize(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::I8(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::I16(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::I32(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::I64(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::I128(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerValue::Isize(left) => left.paired_comparison(right, |a, b| a >= b),
                }
            }

            [context] fn eq(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::U8(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::U16(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::U32(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::U64(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::U128(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::Usize(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::I8(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::I16(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::I32(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::I64(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::I128(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerValue::Isize(left) => left.paired_comparison(right, |a, b| a == b),
                }
            }

            [context] fn ne(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                let right = Spanned(right, context.output_span_range);
                match IntegerValue::resolve_untyped_to_match(left, &right, context.output_span_range)? {
                    IntegerValue::Untyped(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::U8(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::U16(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::U32(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::U64(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::U128(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::Usize(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::I8(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::I16(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::I32(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::I64(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::I128(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerValue::Isize(left) => left.paired_comparison(right, |a, b| a != b),
                }
            }
        }
        interface_items {
            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    // Arithmetic operations
                    BinaryOperation::Addition { .. } => binary_definitions::add(),
                    BinaryOperation::Subtraction { .. } => binary_definitions::sub(),
                    BinaryOperation::Multiplication { .. } => binary_definitions::mul(),
                    BinaryOperation::Division { .. } => binary_definitions::div(),
                    BinaryOperation::Remainder { .. } => binary_definitions::rem(),
                    // Bitwise operations
                    BinaryOperation::BitXor { .. } => binary_definitions::bitxor(),
                    BinaryOperation::BitAnd { .. } => binary_definitions::bitand(),
                    BinaryOperation::BitOr { .. } => binary_definitions::bitor(),
                    BinaryOperation::ShiftLeft { .. } => binary_definitions::shift_left(),
                    BinaryOperation::ShiftRight { .. } => binary_definitions::shift_right(),
                    // Compound assignment operations
                    BinaryOperation::AddAssign { .. } => binary_definitions::add_assign(),
                    BinaryOperation::SubAssign { .. } => binary_definitions::sub_assign(),
                    BinaryOperation::MulAssign { .. } => binary_definitions::mul_assign(),
                    BinaryOperation::DivAssign { .. } => binary_definitions::div_assign(),
                    BinaryOperation::RemAssign { .. } => binary_definitions::rem_assign(),
                    BinaryOperation::BitXorAssign { .. } => binary_definitions::bitxor_assign(),
                    BinaryOperation::BitAndAssign { .. } => binary_definitions::bitand_assign(),
                    BinaryOperation::BitOrAssign { .. } => binary_definitions::bitor_assign(),
                    BinaryOperation::ShlAssign { .. } => binary_definitions::shift_left_assign(),
                    BinaryOperation::ShrAssign { .. } => binary_definitions::shift_right_assign(),
                    // Comparison operations
                    BinaryOperation::LessThan { .. } => binary_definitions::lt(),
                    BinaryOperation::LessThanOrEqual { .. } => binary_definitions::le(),
                    BinaryOperation::GreaterThan { .. } => binary_definitions::gt(),
                    BinaryOperation::GreaterThanOrEqual { .. } => binary_definitions::ge(),
                    BinaryOperation::Equal { .. } => binary_definitions::eq(),
                    BinaryOperation::NotEqual { .. } => binary_definitions::ne(),
                    _ => return None,
                })
            }
        }
    }
}

impl_resolvable_argument_for! {
    IntegerTypeData,
    (value, context) -> IntegerValue {
        match value {
            Value::Integer(value) => Ok(value),
            other => context.err("an integer", other),
        }
    }
}

pub(crate) struct CoercedToU32(pub(crate) u32);

impl ResolvableArgumentTarget for CoercedToU32 {
    type ValueType = IntegerTypeData;
}

impl ResolvableOwned<Value> for CoercedToU32 {
    fn resolve_from_value(input_value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
        let integer = match input_value {
            Value::Integer(value) => value,
            other => return context.err("an integer", other),
        };
        let coerced = match integer {
            IntegerValue::U8(x) => Some(x as u32),
            IntegerValue::U16(x) => Some(x as u32),
            IntegerValue::U32(x) => Some(x),
            IntegerValue::U64(x) => x.try_into().ok(),
            IntegerValue::U128(x) => x.try_into().ok(),
            IntegerValue::Usize(x) => x.try_into().ok(),
            IntegerValue::I8(x) => x.try_into().ok(),
            IntegerValue::I16(x) => x.try_into().ok(),
            IntegerValue::I32(x) => x.try_into().ok(),
            IntegerValue::I64(x) => x.try_into().ok(),
            IntegerValue::I128(x) => x.try_into().ok(),
            IntegerValue::Isize(x) => x.try_into().ok(),
            IntegerValue::Untyped(x) => x.into_fallback().try_into().ok(),
        };
        match coerced {
            Some(value) => Ok(CoercedToU32(value)),
            None => context.err("a u32-compatible integer", Value::Integer(integer)),
        }
    }
}
