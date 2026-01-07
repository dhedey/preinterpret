use super::*;

define_parent_type! {
    pub(crate) FloatType => ValueType(ValueContent::Float),
    content: pub(crate) FloatContent,
    leaf_kind: pub(crate) FloatLeafKind,
    type_kind: ParentTypeKind::Float(pub(crate) FloatTypeKind),
    variants: {
        Untyped => UntypedFloatType,
        F32 => F32Type,
        F64 => F64Type,
    },
    type_name: "float",
    articled_display_name: "a float",
}

pub(crate) type FloatValue = FloatContent<'static, BeOwned>;
pub(crate) type FloatValueRef<'a> = FloatContent<'a, BeRef>;

impl FloatValue {
    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ParseResult<Self> {
        Ok(match lit.suffix() {
            "" => FloatContent::Untyped(UntypedFloat::new_from_lit_float(lit)?),
            "f32" => FloatContent::F32(lit.base10_parse()?),
            "f64" => FloatContent::F64(lit.base10_parse()?),
            suffix => {
                return lit.span().parse_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        })
    }

    pub(crate) fn resolve_untyped_to_match(self, target: FloatValueRef) -> FloatValue {
        match self {
            FloatContent::Untyped(this) => this.into_kind(target.kind()),
            other => other,
        }
    }

    #[allow(dead_code)]
    pub(super) fn to_literal(self, span: Span) -> Literal {
        self.to_unspanned_literal().with_span(span)
    }

    fn to_unspanned_literal(self) -> Literal {
        match self {
            FloatContent::Untyped(float) => float.to_unspanned_literal(),
            FloatContent::F32(float) => Literal::f32_suffixed(float),
            FloatContent::F64(float) => Literal::f64_suffixed(float),
        }
    }
}

// TODO[concepts]: Move to FloatRef<'a> when we can
impl FloatValue {
    /// Outputs this float value to a token stream.
    /// For finite values, outputs a literal. For non-finite values (infinity, NaN),
    /// outputs the equivalent constant path like `f32::INFINITY`.
    pub(super) fn output_to(&self, output: &mut ToStreamContext) {
        let span = output.new_token_span();
        match self {
            FloatContent::Untyped(float) => {
                let f = float.into_fallback();
                if f.is_finite() {
                    output.push_literal(Literal::f64_unsuffixed(f).with_span(span));
                } else if f.is_nan() {
                    // For untyped NaN, we output f64::NAN since FallbackFloat is f64
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f64::NAN));
                } else if f.is_sign_positive() {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f64::INFINITY));
                } else {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f64::NEG_INFINITY));
                }
            }
            FloatContent::F32(f) => {
                if f.is_finite() {
                    output.push_literal(Literal::f32_suffixed(*f).with_span(span));
                } else if f.is_nan() {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f32::NAN));
                } else if f.is_sign_positive() {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f32::INFINITY));
                } else {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f32::NEG_INFINITY));
                }
            }
            FloatContent::F64(f) => {
                if f.is_finite() {
                    output.push_literal(Literal::f64_suffixed(*f).with_span(span));
                } else if f.is_nan() {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f64::NAN));
                } else if f.is_sign_positive() {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f64::INFINITY));
                } else {
                    output.extend_raw_tokens(quote::quote_spanned!(span=> f64::NEG_INFINITY));
                }
            }
        }
    }
}

fn assign_op<R>(
    mut left: Assignee<FloatValue>,
    right: R,
    context: BinaryOperationCallContext,
    op: fn(BinaryOperationCallContext, FloatValue, R) -> ExecutionResult<FloatValue>,
) -> ExecutionResult<()> {
    let left_value = core::mem::replace(&mut *left, FloatContent::F32(0.0));
    let result = op(context, left_value, right)?;
    *left = result;
    Ok(())
}

impl Debug for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatContent::Untyped(v) => write!(f, "{}", v.into_fallback()),
            FloatContent::F32(v) => write!(f, "{:?}", v),
            FloatContent::F64(v) => write!(f, "{:?}", v),
        }
    }
}

/// Aligns types for comparison - converts untyped to match the other's type.
/// Unlike integers, float conversion never fails (may lose precision).
fn align_types(mut lhs: FloatValue, mut rhs: FloatValue) -> (FloatValue, FloatValue) {
    match (&lhs, &rhs) {
        (FloatContent::Untyped(l), typed) if !matches!(typed, FloatContent::Untyped(_)) => {
            lhs = l.into_kind(typed.kind());
        }
        (typed, FloatContent::Untyped(r)) if !matches!(typed, FloatContent::Untyped(_)) => {
            rhs = r.into_kind(lhs.kind());
        }
        _ => {} // Both same type or both untyped - no conversion needed
    }
    (lhs, rhs)
}

// TODO[concepts]: Should really be over FloatRef<'a>
impl ValuesEqual for FloatValue {
    /// Handles type coercion between typed and untyped floats.
    /// Uses Rust's float `==`, so `NaN != NaN`.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        // Align types (untyped -> typed conversion)
        let (lhs, rhs) = align_types(*self, *other);

        // After alignment, compare directly.
        // Each variant has two lines: same-type comparison, then type-mismatch fallback.
        // This ensures adding a new variant causes a compiler error.
        let equal = match (lhs, rhs) {
            (FloatContent::Untyped(l), FloatContent::Untyped(r)) => {
                l.into_fallback() == r.into_fallback()
            }
            (FloatContent::Untyped(_), _) => return ctx.leaf_values_not_equal(self, other),
            (FloatContent::F32(l), FloatContent::F32(r)) => l == r,
            (FloatContent::F32(_), _) => return ctx.leaf_values_not_equal(self, other),
            (FloatContent::F64(l), FloatContent::F64(r)) => l == r,
            (FloatContent::F64(_), _) => return ctx.leaf_values_not_equal(self, other),
        };

        if equal {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

define_type_features! {
    impl FloatType,
    pub(crate) mod float_interface {
        pub(crate) mod methods {
            fn is_nan(this: FloatValue) -> bool {
                match this {
                    FloatContent::Untyped(x) => x.into_fallback().is_nan(),
                    FloatContent::F32(x) => x.is_nan(),
                    FloatContent::F64(x) => x.is_nan(),
                }
            }

            fn is_infinite(this: FloatValue) -> bool {
                match this {
                    FloatContent::Untyped(x) => x.into_fallback().is_infinite(),
                    FloatContent::F32(x) => x.is_infinite(),
                    FloatContent::F64(x) => x.is_infinite(),
                }
            }

            fn is_finite(this: FloatValue) -> bool {
                match this {
                    FloatContent::Untyped(x) => x.into_fallback().is_finite(),
                    FloatContent::F32(x) => x.is_finite(),
                    FloatContent::F64(x) => x.is_finite(),
                }
            }

            fn is_sign_positive(this: FloatValue) -> bool {
                match this {
                    FloatContent::Untyped(x) => x.into_fallback().is_sign_positive(),
                    FloatContent::F32(x) => x.is_sign_positive(),
                    FloatContent::F64(x) => x.is_sign_positive(),
                }
            }

            fn is_sign_negative(this: FloatValue) -> bool {
                match this {
                    FloatContent::Untyped(x) => x.into_fallback().is_sign_negative(),
                    FloatContent::F32(x) => x.is_sign_negative(),
                    FloatContent::F64(x) => x.is_sign_negative(),
                }
            }
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {
            fn add(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<FloatValue> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_operation(right, |a, b| a + b),
                    FloatContent::F32(left) => left.paired_operation_no_overflow(right, |a, b| a + b),
                    FloatContent::F64(left) => left.paired_operation_no_overflow(right, |a, b| a + b),
                }
            }

            [context] fn add_assign(left: Assignee<FloatValue>, right: Spanned<FloatValue>) -> ExecutionResult<()> {
                assign_op(left, right, context, add)
            }

            fn sub(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<FloatValue> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_operation(right, |a, b| a - b),
                    FloatContent::F32(left) => left.paired_operation_no_overflow(right, |a, b| a - b),
                    FloatContent::F64(left) => left.paired_operation_no_overflow(right, |a, b| a - b),
                }
            }

            [context] fn sub_assign(left: Assignee<FloatValue>, right: Spanned<FloatValue>) -> ExecutionResult<()> {
                assign_op(left, right, context, sub)
            }

            fn mul(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<FloatValue> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_operation(right, |a, b| a * b),
                    FloatContent::F32(left) => left.paired_operation_no_overflow(right, |a, b| a * b),
                    FloatContent::F64(left) => left.paired_operation_no_overflow(right, |a, b| a * b),
                }
            }

            [context] fn mul_assign(left: Assignee<FloatValue>, right: Spanned<FloatValue>) -> ExecutionResult<()> {
                assign_op(left, right, context, mul)
            }

            fn div(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<FloatValue> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_operation(right, |a, b| a / b),
                    FloatContent::F32(left) => left.paired_operation_no_overflow(right, |a, b| a / b),
                    FloatContent::F64(left) => left.paired_operation_no_overflow(right, |a, b| a / b),
                }
            }

            [context] fn div_assign(left: Assignee<FloatValue>, right: Spanned<FloatValue>) -> ExecutionResult<()> {
                assign_op(left, right, context, div)
            }

            fn rem(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<FloatValue> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_operation(right, |a, b| a % b),
                    FloatContent::F32(left) => left.paired_operation_no_overflow(right, |a, b| a % b),
                    FloatContent::F64(left) => left.paired_operation_no_overflow(right, |a, b| a % b),
                }
            }

            [context] fn rem_assign(left: Assignee<FloatValue>, right: Spanned<FloatValue>) -> ExecutionResult<()> {
                assign_op(left, right, context, rem)
            }

            fn lt(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_comparison(right, |a, b| a < b),
                    FloatContent::F32(left) => left.paired_comparison(right, |a, b| a < b),
                    FloatContent::F64(left) => left.paired_comparison(right, |a, b| a < b),
                }
            }

            fn le(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_comparison(right, |a, b| a <= b),
                    FloatContent::F32(left) => left.paired_comparison(right, |a, b| a <= b),
                    FloatContent::F64(left) => left.paired_comparison(right, |a, b| a <= b),
                }
            }

            fn gt(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_comparison(right, |a, b| a > b),
                    FloatContent::F32(left) => left.paired_comparison(right, |a, b| a > b),
                    FloatContent::F64(left) => left.paired_comparison(right, |a, b| a > b),
                }
            }

            fn ge(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_comparison(right, |a, b| a >= b),
                    FloatContent::F32(left) => left.paired_comparison(right, |a, b| a >= b),
                    FloatContent::F64(left) => left.paired_comparison(right, |a, b| a >= b),
                }
            }

            fn eq(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_comparison(right, |a, b| a == b),
                    FloatContent::F32(left) => left.paired_comparison(right, |a, b| a == b),
                    FloatContent::F64(left) => left.paired_comparison(right, |a, b| a == b),
                }
            }

            fn ne(left: FloatValue, right: Spanned<FloatValue>) -> ExecutionResult<bool> {
                match left.resolve_untyped_to_match(right.as_ref_value()) {
                    FloatContent::Untyped(left) => left.paired_comparison(right, |a, b| a != b),
                    FloatContent::F32(left) => left.paired_comparison(right, |a, b| a != b),
                    FloatContent::F64(left) => left.paired_comparison(right, |a, b| a != b),
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
                    // Compound assignment operations
                    BinaryOperation::AddAssign { .. } => binary_definitions::add_assign(),
                    BinaryOperation::SubAssign { .. } => binary_definitions::sub_assign(),
                    BinaryOperation::MulAssign { .. } => binary_definitions::mul_assign(),
                    BinaryOperation::DivAssign { .. } => binary_definitions::div_assign(),
                    BinaryOperation::RemAssign { .. } => binary_definitions::rem_assign(),
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
    FloatType,
    (value, context) -> FloatValue {
        match value {
            Value::Float(value) => Ok(value),
            other => context.err("a float", other),
        }
    }
}
