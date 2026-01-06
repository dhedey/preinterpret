use super::*;

define_parent_type! {
    pub(crate) IntegerType => ValueType(ValueContent::Integer),
    content: pub(crate) IntegerContent,
    leaf_kind: pub(crate) IntegerLeafKind,
    type_kind: ParentTypeKind::Integer(pub(crate) IntegerTypeKind),
    variants: {
        Untyped => UntypedIntegerType,
        U8 => U8Type,
        U16 => U16Type,
        U32 => U32Type,
        U64 => U64Type,
        U128 => U128Type,
        Usize => UsizeType,
        I8 => I8Type,
        I16 => I16Type,
        I32 => I32Type,
        I64 => I64Type,
        I128 => I128Type,
        Isize => IsizeType,
    },
    type_name: "int",
    articled_display_name: "an integer",
}

pub(crate) type OwnedInteger = QqqOwned<IntegerType>;
pub(crate) type OwnedIntegerContent = IntegerContent<'static, BeOwned>;
pub(crate) type IntegerRef<'a> = QqqRef<'a, IntegerType>;

// TODO[concepts]: Remove when transition is complete
pub(crate) type IntegerValue = OwnedIntegerContent;

impl OwnedInteger {
    pub(super) fn for_litint(lit: &syn::LitInt) -> ParseResult<Self> {
        Ok(match lit.suffix() {
            "" => IntegerContent::Untyped(UntypedInteger::new_from_lit_int(lit)?),
            "u8" => IntegerContent::U8(lit.base10_parse()?),
            "u16" => IntegerContent::U16(lit.base10_parse()?),
            "u32" => IntegerContent::U32(lit.base10_parse()?),
            "u64" => IntegerContent::U64(lit.base10_parse()?),
            "u128" => IntegerContent::U128(lit.base10_parse()?),
            "usize" => IntegerContent::Usize(lit.base10_parse()?),
            "i8" => IntegerContent::I8(lit.base10_parse()?),
            "i16" => IntegerContent::I16(lit.base10_parse()?),
            "i32" => IntegerContent::I32(lit.base10_parse()?),
            "i64" => IntegerContent::I64(lit.base10_parse()?),
            "i128" => IntegerContent::I128(lit.base10_parse()?),
            "isize" => IntegerContent::Isize(lit.base10_parse()?),
            suffix => {
                return lit.span().parse_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        }
        .into_actual())
    }

    pub(crate) fn resolve_untyped_to_match(
        self,
        target: IntegerRef,
    ) -> ExecutionResult<OwnedIntegerContent> {
        match self.0 {
            IntegerContent::Untyped(this) => this.into_owned().try_into_kind(target.kind()),
            other => Ok(other),
        }
    }
}

// TODO[concepts]: Move to OwnedInteger when we can
impl OwnedIntegerContent {
    pub(crate) fn resolve_untyped_to_match_other(
        self,
        span: SpanRange,
        other: &Value,
    ) -> ExecutionResult<OwnedIntegerContent> {
        match (self, other) {
            (IntegerContent::Untyped(this), Value::Integer(other)) => {
                this.spanned(span).into_kind(other.kind())
            }
            (value, _) => Ok(value),
        }
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.to_unspanned_literal().with_span(span)
    }

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            IntegerContent::Untyped(int) => int.to_unspanned_literal(),
            IntegerContent::U8(int) => Literal::u8_suffixed(*int),
            IntegerContent::U16(int) => Literal::u16_suffixed(*int),
            IntegerContent::U32(int) => Literal::u32_suffixed(*int),
            IntegerContent::U64(int) => Literal::u64_suffixed(*int),
            IntegerContent::U128(int) => Literal::u128_suffixed(*int),
            IntegerContent::Usize(int) => Literal::usize_suffixed(*int),
            IntegerContent::I8(int) => Literal::i8_suffixed(*int),
            IntegerContent::I16(int) => Literal::i16_suffixed(*int),
            IntegerContent::I32(int) => Literal::i32_suffixed(*int),
            IntegerContent::I64(int) => Literal::i64_suffixed(*int),
            IntegerContent::I128(int) => Literal::i128_suffixed(*int),
            IntegerContent::Isize(int) => Literal::isize_suffixed(*int),
        }
    }
}

fn assign_op<R>(
    mut left: Assignee<OwnedIntegerContent>,
    right: R,
    context: BinaryOperationCallContext,
    op: fn(BinaryOperationCallContext, OwnedInteger, R) -> ExecutionResult<OwnedIntegerContent>,
) -> ExecutionResult<()> {
    let left_value = core::mem::replace(&mut *left, IntegerContent::U32(0));
    let result = op(context, left_value.into_actual(), right)?;
    *left = result;
    Ok(())
}

impl Debug for OwnedIntegerContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegerContent::Untyped(v) => write!(f, "{}", v.into_fallback()),
            IntegerContent::U8(v) => write!(f, "{:?}", v),
            IntegerContent::U16(v) => write!(f, "{:?}", v),
            IntegerContent::U32(v) => write!(f, "{:?}", v),
            IntegerContent::U64(v) => write!(f, "{:?}", v),
            IntegerContent::U128(v) => write!(f, "{:?}", v),
            IntegerContent::Usize(v) => write!(f, "{:?}", v),
            IntegerContent::I8(v) => write!(f, "{:?}", v),
            IntegerContent::I16(v) => write!(f, "{:?}", v),
            IntegerContent::I32(v) => write!(f, "{:?}", v),
            IntegerContent::I64(v) => write!(f, "{:?}", v),
            IntegerContent::I128(v) => write!(f, "{:?}", v),
            IntegerContent::Isize(v) => write!(f, "{:?}", v),
        }
    }
}

/// Aligns types for comparison - converts untyped to match the other's type.
/// Returns `None` if the untyped value doesn't fit in the target type.
fn align_types(
    mut lhs: OwnedIntegerContent,
    mut rhs: OwnedIntegerContent,
) -> Option<(OwnedIntegerContent, OwnedIntegerContent)> {
    match (&lhs, &rhs) {
        (IntegerContent::Untyped(l), typed) if !matches!(typed, IntegerContent::Untyped(_)) => {
            lhs = l.try_into_kind(typed.kind()).ok()?;
        }
        (typed, IntegerContent::Untyped(r)) if !matches!(typed, IntegerContent::Untyped(_)) => {
            rhs = r.try_into_kind(lhs.kind()).ok()?;
        }
        _ => {} // Both same type or both untyped - no conversion needed
    }
    Some((lhs, rhs))
}

// TODO[concepts]: Should really be over IntegerRef<'a>
impl ValuesEqual for OwnedIntegerContent {
    /// Handles type coercion between typed and untyped integers.
    /// E.g., `5 == 5u32` returns true.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        // Align types (untyped -> typed conversion)
        let Some((lhs, rhs)) = align_types(*self, *other) else {
            return ctx.leaf_values_not_equal(self, other);
        };

        // After alignment, compare directly.
        // Each variant has two lines: same-type comparison, then type-mismatch fallback.
        // This ensures adding a new variant causes a compiler error.
        let equal = match (lhs, rhs) {
            (IntegerContent::Untyped(l), IntegerContent::Untyped(r)) => {
                l.into_fallback() == r.into_fallback()
            }
            (IntegerContent::Untyped(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::U8(l), IntegerContent::U8(r)) => l == r,
            (IntegerContent::U8(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::U16(l), IntegerContent::U16(r)) => l == r,
            (IntegerContent::U16(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::U32(l), IntegerContent::U32(r)) => l == r,
            (IntegerContent::U32(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::U64(l), IntegerContent::U64(r)) => l == r,
            (IntegerContent::U64(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::U128(l), IntegerContent::U128(r)) => l == r,
            (IntegerContent::U128(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::Usize(l), IntegerContent::Usize(r)) => l == r,
            (IntegerContent::Usize(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::I8(l), IntegerContent::I8(r)) => l == r,
            (IntegerContent::I8(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::I16(l), IntegerContent::I16(r)) => l == r,
            (IntegerContent::I16(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::I32(l), IntegerContent::I32(r)) => l == r,
            (IntegerContent::I32(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::I64(l), IntegerContent::I64(r)) => l == r,
            (IntegerContent::I64(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::I128(l), IntegerContent::I128(r)) => l == r,
            (IntegerContent::I128(_), _) => return ctx.leaf_values_not_equal(self, other),
            (IntegerContent::Isize(l), IntegerContent::Isize(r)) => l == r,
            (IntegerContent::Isize(_), _) => return ctx.leaf_values_not_equal(self, other),
        };

        if equal {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

define_type_features! {
    impl IntegerType,
    pub(crate) mod integer_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {
            [context] fn add(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_add),
                    IntegerContent::U8(left) => left.paired_operation(right, context, u8::checked_add),
                    IntegerContent::U16(left) => left.paired_operation(right, context, u16::checked_add),
                    IntegerContent::U32(left) => left.paired_operation(right, context, u32::checked_add),
                    IntegerContent::U64(left) => left.paired_operation(right, context, u64::checked_add),
                    IntegerContent::U128(left) => left.paired_operation(right, context, u128::checked_add),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, usize::checked_add),
                    IntegerContent::I8(left) => left.paired_operation(right, context, i8::checked_add),
                    IntegerContent::I16(left) => left.paired_operation(right, context, i16::checked_add),
                    IntegerContent::I32(left) => left.paired_operation(right, context, i32::checked_add),
                    IntegerContent::I64(left) => left.paired_operation(right, context, i64::checked_add),
                    IntegerContent::I128(left) => left.paired_operation(right, context, i128::checked_add),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, isize::checked_add),
                }
            }

            [context] fn add_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, add)
            }

            [context] fn sub(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_sub),
                    IntegerContent::U8(left) => left.paired_operation(right, context, u8::checked_sub),
                    IntegerContent::U16(left) => left.paired_operation(right, context, u16::checked_sub),
                    IntegerContent::U32(left) => left.paired_operation(right, context, u32::checked_sub),
                    IntegerContent::U64(left) => left.paired_operation(right, context, u64::checked_sub),
                    IntegerContent::U128(left) => left.paired_operation(right, context, u128::checked_sub),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, usize::checked_sub),
                    IntegerContent::I8(left) => left.paired_operation(right, context, i8::checked_sub),
                    IntegerContent::I16(left) => left.paired_operation(right, context, i16::checked_sub),
                    IntegerContent::I32(left) => left.paired_operation(right, context, i32::checked_sub),
                    IntegerContent::I64(left) => left.paired_operation(right, context, i64::checked_sub),
                    IntegerContent::I128(left) => left.paired_operation(right, context, i128::checked_sub),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, isize::checked_sub),
                }
            }

            [context] fn sub_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, sub)
            }

            [context] fn mul(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_mul),
                    IntegerContent::U8(left) => left.paired_operation(right, context, u8::checked_mul),
                    IntegerContent::U16(left) => left.paired_operation(right, context, u16::checked_mul),
                    IntegerContent::U32(left) => left.paired_operation(right, context, u32::checked_mul),
                    IntegerContent::U64(left) => left.paired_operation(right, context, u64::checked_mul),
                    IntegerContent::U128(left) => left.paired_operation(right, context, u128::checked_mul),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, usize::checked_mul),
                    IntegerContent::I8(left) => left.paired_operation(right, context, i8::checked_mul),
                    IntegerContent::I16(left) => left.paired_operation(right, context, i16::checked_mul),
                    IntegerContent::I32(left) => left.paired_operation(right, context, i32::checked_mul),
                    IntegerContent::I64(left) => left.paired_operation(right, context, i64::checked_mul),
                    IntegerContent::I128(left) => left.paired_operation(right, context, i128::checked_mul),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, isize::checked_mul),
                }
            }

            [context] fn mul_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, mul)
            }

            [context] fn div(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_div),
                    IntegerContent::U8(left) => left.paired_operation(right, context, u8::checked_div),
                    IntegerContent::U16(left) => left.paired_operation(right, context, u16::checked_div),
                    IntegerContent::U32(left) => left.paired_operation(right, context, u32::checked_div),
                    IntegerContent::U64(left) => left.paired_operation(right, context, u64::checked_div),
                    IntegerContent::U128(left) => left.paired_operation(right, context, u128::checked_div),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, usize::checked_div),
                    IntegerContent::I8(left) => left.paired_operation(right, context, i8::checked_div),
                    IntegerContent::I16(left) => left.paired_operation(right, context, i16::checked_div),
                    IntegerContent::I32(left) => left.paired_operation(right, context, i32::checked_div),
                    IntegerContent::I64(left) => left.paired_operation(right, context, i64::checked_div),
                    IntegerContent::I128(left) => left.paired_operation(right, context, i128::checked_div),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, isize::checked_div),
                }
            }

            [context] fn div_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, div)
            }

            [context] fn rem(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, FallbackInteger::checked_rem),
                    IntegerContent::U8(left) => left.paired_operation(right, context, u8::checked_rem),
                    IntegerContent::U16(left) => left.paired_operation(right, context, u16::checked_rem),
                    IntegerContent::U32(left) => left.paired_operation(right, context, u32::checked_rem),
                    IntegerContent::U64(left) => left.paired_operation(right, context, u64::checked_rem),
                    IntegerContent::U128(left) => left.paired_operation(right, context, u128::checked_rem),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, usize::checked_rem),
                    IntegerContent::I8(left) => left.paired_operation(right, context, i8::checked_rem),
                    IntegerContent::I16(left) => left.paired_operation(right, context, i16::checked_rem),
                    IntegerContent::I32(left) => left.paired_operation(right, context, i32::checked_rem),
                    IntegerContent::I64(left) => left.paired_operation(right, context, i64::checked_rem),
                    IntegerContent::I128(left) => left.paired_operation(right, context, i128::checked_rem),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, isize::checked_rem),
                }
            }

            [context] fn rem_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, rem)
            }

            [context] fn bitxor(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::U8(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::U16(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::U32(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::U64(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::U128(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::I8(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::I16(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::I32(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::I64(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::I128(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, |a, b| Some(a ^ b)),
                }
            }

            [context] fn bitxor_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, bitxor)
            }

            [context] fn bitand(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::U8(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::U16(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::U32(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::U64(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::U128(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::I8(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::I16(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::I32(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::I64(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::I128(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, |a, b| Some(a & b)),
                }
            }

            [context] fn bitand_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, bitand)
            }

            [context] fn bitor(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<OwnedIntegerContent> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::U8(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::U16(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::U32(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::U64(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::U128(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::Usize(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::I8(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::I16(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::I32(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::I64(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::I128(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                    IntegerContent::Isize(left) => left.paired_operation(right, context, |a, b| Some(a | b)),
                }
            }

            [context] fn bitor_assign(left: Assignee<OwnedIntegerContent>, right: Spanned<OwnedInteger>) -> ExecutionResult<()> {
                assign_op(left, right, context, bitor)
            }

            [context] fn shift_left(left: OwnedInteger, CoercedToU32(right): CoercedToU32) -> ExecutionResult<OwnedIntegerContent> {
                match left.0 {
                    IntegerContent::Untyped(left) => left.shift_operation(right, context, FallbackInteger::checked_shl),
                    IntegerContent::U8(left) => left.shift_operation(right, context, u8::checked_shl),
                    IntegerContent::U16(left) => left.shift_operation(right, context, u16::checked_shl),
                    IntegerContent::U32(left) => left.shift_operation(right, context, u32::checked_shl),
                    IntegerContent::U64(left) => left.shift_operation(right, context, u64::checked_shl),
                    IntegerContent::U128(left) => left.shift_operation(right, context, u128::checked_shl),
                    IntegerContent::Usize(left) => left.shift_operation(right, context, usize::checked_shl),
                    IntegerContent::I8(left) => left.shift_operation(right, context, i8::checked_shl),
                    IntegerContent::I16(left) => left.shift_operation(right, context, i16::checked_shl),
                    IntegerContent::I32(left) => left.shift_operation(right, context, i32::checked_shl),
                    IntegerContent::I64(left) => left.shift_operation(right, context, i64::checked_shl),
                    IntegerContent::I128(left) => left.shift_operation(right, context, i128::checked_shl),
                    IntegerContent::Isize(left) => left.shift_operation(right, context, isize::checked_shl),
                }
            }

            [context] fn shift_left_assign(left: Assignee<OwnedIntegerContent>, right: CoercedToU32) -> ExecutionResult<()> {
                assign_op(left, right, context, shift_left)
            }

            [context] fn shift_right(left: OwnedInteger, CoercedToU32(right): CoercedToU32) -> ExecutionResult<OwnedIntegerContent> {
                match left.0 {
                    IntegerContent::Untyped(left) => left.shift_operation(right, context, FallbackInteger::checked_shr),
                    IntegerContent::U8(left) => left.shift_operation(right, context, u8::checked_shr),
                    IntegerContent::U16(left) => left.shift_operation(right, context, u16::checked_shr),
                    IntegerContent::U32(left) => left.shift_operation(right, context, u32::checked_shr),
                    IntegerContent::U64(left) => left.shift_operation(right, context, u64::checked_shr),
                    IntegerContent::U128(left) => left.shift_operation(right, context, u128::checked_shr),
                    IntegerContent::Usize(left) => left.shift_operation(right, context, usize::checked_shr),
                    IntegerContent::I8(left) => left.shift_operation(right, context, i8::checked_shr),
                    IntegerContent::I16(left) => left.shift_operation(right, context, i16::checked_shr),
                    IntegerContent::I32(left) => left.shift_operation(right, context, i32::checked_shr),
                    IntegerContent::I64(left) => left.shift_operation(right, context, i64::checked_shr),
                    IntegerContent::I128(left) => left.shift_operation(right, context, i128::checked_shr),
                    IntegerContent::Isize(left) => left.shift_operation(right, context, isize::checked_shr),
                }
            }

            [context] fn shift_right_assign(left: Assignee<OwnedIntegerContent>, right: CoercedToU32) -> ExecutionResult<()> {
                assign_op(left, right, context, shift_right)
            }

            fn lt(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::U8(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::U16(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::U32(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::U64(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::U128(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::Usize(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::I8(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::I16(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::I32(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::I64(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::I128(left) => left.paired_comparison(right, |a, b| a < b),
                    IntegerContent::Isize(left) => left.paired_comparison(right, |a, b| a < b),
                }
            }

            fn le(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::U8(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::U16(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::U32(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::U64(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::U128(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::Usize(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::I8(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::I16(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::I32(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::I64(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::I128(left) => left.paired_comparison(right, |a, b| a <= b),
                    IntegerContent::Isize(left) => left.paired_comparison(right, |a, b| a <= b),
                }
            }

            fn gt(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::U8(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::U16(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::U32(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::U64(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::U128(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::Usize(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::I8(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::I16(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::I32(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::I64(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::I128(left) => left.paired_comparison(right, |a, b| a > b),
                    IntegerContent::Isize(left) => left.paired_comparison(right, |a, b| a > b),
                }
            }

            fn ge(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::U8(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::U16(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::U32(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::U64(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::U128(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::Usize(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::I8(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::I16(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::I32(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::I64(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::I128(left) => left.paired_comparison(right, |a, b| a >= b),
                    IntegerContent::Isize(left) => left.paired_comparison(right, |a, b| a >= b),
                }
            }

            fn eq(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::U8(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::U16(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::U32(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::U64(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::U128(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::Usize(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::I8(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::I16(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::I32(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::I64(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::I128(left) => left.paired_comparison(right, |a, b| a == b),
                    IntegerContent::Isize(left) => left.paired_comparison(right, |a, b| a == b),
                }
            }

            fn ne(left: OwnedInteger, right: Spanned<OwnedInteger>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref())? {
                    IntegerContent::Untyped(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::U8(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::U16(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::U32(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::U64(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::U128(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::Usize(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::I8(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::I16(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::I32(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::I64(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::I128(left) => left.paired_comparison(right, |a, b| a != b),
                    IntegerContent::Isize(left) => left.paired_comparison(right, |a, b| a != b),
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
    IntegerType,
    (value, context) -> OwnedIntegerContent {
        match value {
            Value::Integer(value) => Ok(value),
            other => context.err("an integer", other),
        }
    }
}

pub(crate) struct CoercedToU32(pub(crate) u32);

impl ResolvableArgumentTarget for CoercedToU32 {
    type ValueType = IntegerType;
}

impl ResolvableOwned<Value> for CoercedToU32 {
    fn resolve_from_value(input_value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
        let integer = match input_value {
            Value::Integer(value) => value,
            other => return context.err("an integer", other),
        };
        let coerced = match integer {
            IntegerContent::U8(x) => Some(x as u32),
            IntegerContent::U16(x) => Some(x as u32),
            IntegerContent::U32(x) => Some(x),
            IntegerContent::U64(x) => x.try_into().ok(),
            IntegerContent::U128(x) => x.try_into().ok(),
            IntegerContent::Usize(x) => x.try_into().ok(),
            IntegerContent::I8(x) => x.try_into().ok(),
            IntegerContent::I16(x) => x.try_into().ok(),
            IntegerContent::I32(x) => x.try_into().ok(),
            IntegerContent::I64(x) => x.try_into().ok(),
            IntegerContent::I128(x) => x.try_into().ok(),
            IntegerContent::Isize(x) => x.try_into().ok(),
            IntegerContent::Untyped(x) => x.into_fallback().try_into().ok(),
        };
        match coerced {
            Some(value) => Ok(CoercedToU32(value)),
            None => context.err("a u32-compatible integer", Value::Integer(integer)),
        }
    }
}
