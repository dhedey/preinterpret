use super::*;

#[derive(Clone)]
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
        .into_owned(lit.span_range()))
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.to_unspanned_literal().with_span(span)
    }

    pub(crate) fn resolve_untyped_to_match_other(
        this: Owned<IntegerValue>,
        other: &Value,
    ) -> ExecutionResult<Self> {
        let (value, span_range) = this.deconstruct();
        match (value, other) {
            (IntegerValue::Untyped(this), Value::Integer(other)) => {
                this.into_kind(other.kind(), span_range)
            }
            (value, _) => Ok(value),
        }
    }

    pub(crate) fn resolve_untyped_to_match(
        this: Owned<IntegerValue>,
        target: &IntegerValue,
    ) -> ExecutionResult<Self> {
        let (value, span_range) = this.deconstruct();
        match value {
            IntegerValue::Untyped(this) => this.into_kind(target.kind(), span_range),
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
        let result = op(context, left_value.into_owned(left.span_range()), right)?;
        *left = result;
        Ok(())
    }

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            IntegerValue::Untyped(int) => int.to_unspanned_literal(),
            IntegerValue::U8(int) => Literal::u8_suffixed(*int),
            IntegerValue::U16(int) => Literal::u16_suffixed(*int),
            IntegerValue::U32(int) => Literal::u32_suffixed(*int),
            IntegerValue::U64(int) => Literal::u64_suffixed(*int),
            IntegerValue::U128(int) => Literal::u128_suffixed(*int),
            IntegerValue::Usize(int) => Literal::usize_suffixed(*int),
            IntegerValue::I8(int) => Literal::i8_suffixed(*int),
            IntegerValue::I16(int) => Literal::i16_suffixed(*int),
            IntegerValue::I32(int) => Literal::i32_suffixed(*int),
            IntegerValue::I64(int) => Literal::i64_suffixed(*int),
            IntegerValue::I128(int) => Literal::i128_suffixed(*int),
            IntegerValue::Isize(int) => Literal::isize_suffixed(*int),
        }
    }

    /// Convert to fallback integer for comparison
    fn to_fallback(&self) -> Option<FallbackInteger> {
        Some(match self {
            IntegerValue::Untyped(x) => x.into_fallback(),
            IntegerValue::U8(x) => (*x).into(),
            IntegerValue::U16(x) => (*x).into(),
            IntegerValue::U32(x) => (*x).into(),
            IntegerValue::U64(x) => (*x).into(),
            IntegerValue::U128(x) => (*x).try_into().ok()?,
            IntegerValue::Usize(x) => (*x).try_into().ok()?,
            IntegerValue::I8(x) => (*x).into(),
            IntegerValue::I16(x) => (*x).into(),
            IntegerValue::I32(x) => (*x).into(),
            IntegerValue::I64(x) => (*x).into(),
            IntegerValue::I128(x) => *x,
            IntegerValue::Isize(x) => (*x).try_into().ok()?,
        })
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

impl ValuesEqual for IntegerValue {
    /// Handles type coercion between typed and untyped integers.
    /// E.g., `5 == 5u32` returns true.
    fn values_equal<C: EqualityContext>(
        &self,
        other: &Self,
        _ctx: &mut C,
    ) -> Result<bool, C::Error> {
        Ok(match (self, other) {
            // Same type comparisons
            (IntegerValue::Untyped(l), IntegerValue::Untyped(r)) => {
                l.into_fallback() == r.into_fallback()
            }
            (IntegerValue::U8(l), IntegerValue::U8(r)) => l == r,
            (IntegerValue::U16(l), IntegerValue::U16(r)) => l == r,
            (IntegerValue::U32(l), IntegerValue::U32(r)) => l == r,
            (IntegerValue::U64(l), IntegerValue::U64(r)) => l == r,
            (IntegerValue::U128(l), IntegerValue::U128(r)) => l == r,
            (IntegerValue::Usize(l), IntegerValue::Usize(r)) => l == r,
            (IntegerValue::I8(l), IntegerValue::I8(r)) => l == r,
            (IntegerValue::I16(l), IntegerValue::I16(r)) => l == r,
            (IntegerValue::I32(l), IntegerValue::I32(r)) => l == r,
            (IntegerValue::I64(l), IntegerValue::I64(r)) => l == r,
            (IntegerValue::I128(l), IntegerValue::I128(r)) => l == r,
            (IntegerValue::Isize(l), IntegerValue::Isize(r)) => l == r,
            // Untyped vs typed - compare via fallback
            (IntegerValue::Untyped(l), r) => {
                let l_fallback = l.into_fallback();
                r.to_fallback()
                    .map(|r_fallback| l_fallback == r_fallback)
                    .unwrap_or(false)
            }
            (l, IntegerValue::Untyped(r)) => {
                let r_fallback = r.into_fallback();
                l.to_fallback()
                    .map(|l_fallback| l_fallback == r_fallback)
                    .unwrap_or(false)
            }
            // Different typed integers are never equal
            _ => false,
        })
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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
                match lhs.value {
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
                match lhs.value {
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

            fn lt(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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

            fn le(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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

            fn gt(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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

            fn ge(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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

            fn eq(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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

            fn ne(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<bool> {
                match IntegerValue::resolve_untyped_to_match(left, &right)? {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum IntegerKind {
    Untyped,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

impl IsSpecificValueKind for IntegerKind {
    fn display_name(&self) -> &'static str {
        match self {
            IntegerKind::Untyped => "untyped integer",
            IntegerKind::I8 => "i8",
            IntegerKind::I16 => "i16",
            IntegerKind::I32 => "i32",
            IntegerKind::I64 => "i64",
            IntegerKind::I128 => "i128",
            IntegerKind::Isize => "isize",
            IntegerKind::U8 => "u8",
            IntegerKind::U16 => "u16",
            IntegerKind::U32 => "u32",
            IntegerKind::U64 => "u64",
            IntegerKind::U128 => "u128",
            IntegerKind::Usize => "usize",
        }
    }

    fn articled_display_name(&self) -> &'static str {
        match self {
            IntegerKind::Untyped => "an untyped integer",
            IntegerKind::I8 => "an i8",
            IntegerKind::I16 => "an i16",
            IntegerKind::I32 => "an i32",
            IntegerKind::I64 => "an i64",
            IntegerKind::I128 => "an i128",
            IntegerKind::Isize => "an isize",
            IntegerKind::U8 => "a u8",
            IntegerKind::U16 => "a u16",
            IntegerKind::U32 => "a u32",
            IntegerKind::U64 => "a u64",
            IntegerKind::U128 => "a u128",
            IntegerKind::Usize => "a usize",
        }
    }
}

impl From<IntegerKind> for ValueKind {
    fn from(kind: IntegerKind) -> Self {
        ValueKind::Integer(kind)
    }
}

impl IntegerKind {
    pub(super) fn method_resolver(&self) -> &'static dyn MethodResolver {
        static UNTYPED: UntypedIntegerTypeData = UntypedIntegerTypeData;
        static I8: I8TypeData = I8TypeData;
        static I16: I16TypeData = I16TypeData;
        static I32: I32TypeData = I32TypeData;
        static I64: I64TypeData = I64TypeData;
        static I128: I128TypeData = I128TypeData;
        static ISIZE: IsizeTypeData = IsizeTypeData;
        static U8: U8TypeData = U8TypeData;
        static U16: U16TypeData = U16TypeData;
        static U32: U32TypeData = U32TypeData;
        static U64: U64TypeData = U64TypeData;
        static U128: U128TypeData = U128TypeData;
        static USIZE: UsizeTypeData = UsizeTypeData;
        match self {
            IntegerKind::Untyped => &UNTYPED,
            IntegerKind::I8 => &I8,
            IntegerKind::I16 => &I16,
            IntegerKind::I32 => &I32,
            IntegerKind::I64 => &I64,
            IntegerKind::I128 => &I128,
            IntegerKind::Isize => &ISIZE,
            IntegerKind::U8 => &U8,
            IntegerKind::U16 => &U16,
            IntegerKind::U32 => &U32,
            IntegerKind::U64 => &U64,
            IntegerKind::U128 => &U128,
            IntegerKind::Usize => &USIZE,
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
        let coerced = match integer.clone() {
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
