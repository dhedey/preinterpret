use super::*;
use crate::internal_prelude::*;

#[derive(Copy, Clone)]
pub(crate) enum FloatValue {
    Untyped(UntypedFloat),
    F32(f32),
    F64(f64),
}

impl IntoValue for FloatValue {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}

impl FloatValue {
    pub(super) fn for_litfloat(lit: &syn::LitFloat) -> ParseResult<Owned<Self>> {
        Ok(match lit.suffix() {
            "" => Self::Untyped(UntypedFloat::new_from_lit_float(lit)?),
            "f32" => Self::F32(lit.base10_parse()?),
            "f64" => Self::F64(lit.base10_parse()?),
            suffix => {
                return lit.span().parse_err(format!(
                    "The literal suffix {suffix} is not supported in preinterpret expressions"
                ));
            }
        }
        .into_owned(lit.span()))
    }

    pub(super) fn to_literal(&self, span: Span) -> Literal {
        self.to_unspanned_literal().with_span(span)
    }

    pub(crate) fn resolve_untyped_to_match(
        this: Owned<FloatValue>,
        target: &FloatValue,
    ) -> ExecutionResult<Self> {
        let (value, span_range) = this.deconstruct();
        match value {
            FloatValue::Untyped(this) => this.into_owned(span_range).into_kind(target.kind()),
            other => Ok(other),
        }
    }

    pub(crate) fn assign_op<R>(
        mut left: Assignee<FloatValue>,
        right: R,
        context: BinaryOperationCallContext,
        op: fn(BinaryOperationCallContext, Owned<FloatValue>, R) -> ExecutionResult<FloatValue>,
    ) -> ExecutionResult<()> {
        let left_value = core::mem::replace(&mut *left, FloatValue::F32(0.0));
        let result = op(context, left_value.into_owned(left.span_range()), right)?;
        *left = result;
        Ok(())
    }

    fn to_unspanned_literal(&self) -> Literal {
        match self {
            FloatValue::Untyped(float) => float.to_unspanned_literal(),
            FloatValue::F32(float) => Literal::f32_suffixed(*float),
            FloatValue::F64(float) => Literal::f64_suffixed(*float),
        }
    }
}

impl HasValueKind for FloatValue {
    type SpecificKind = FloatKind;

    fn kind(&self) -> FloatKind {
        match self {
            Self::Untyped(_) => FloatKind::Untyped,
            Self::F32(_) => FloatKind::F32,
            Self::F64(_) => FloatKind::F64,
        }
    }
}

impl FloatValue {
    /// Aligns types for comparison - converts untyped to match the other's type.
    /// Unlike integers, float conversion never fails (may lose precision).
    fn align_types(mut lhs: Self, mut rhs: Self) -> (Self, Self) {
        match (&lhs, &rhs) {
            (FloatValue::Untyped(l), typed) if !matches!(typed, FloatValue::Untyped(_)) => {
                lhs = l.into_kind_infallible(typed.kind());
            }
            (typed, FloatValue::Untyped(r)) if !matches!(typed, FloatValue::Untyped(_)) => {
                rhs = r.into_kind_infallible(lhs.kind());
            }
            _ => {} // Both same type or both untyped - no conversion needed
        }
        (lhs, rhs)
    }
}

impl ValuesEqual for FloatValue {
    /// Handles type coercion between typed and untyped floats.
    /// Uses Rust's float `==`, so `NaN != NaN`.
    fn values_equal<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        // Align types (untyped -> typed conversion)
        let (lhs, rhs) = Self::align_types(*self, *other);

        // After alignment, compare directly
        let equal = match (lhs, rhs) {
            (FloatValue::Untyped(l), FloatValue::Untyped(r)) => {
                l.into_fallback() == r.into_fallback()
            }
            (FloatValue::F32(l), FloatValue::F32(r)) => l == r,
            (FloatValue::F64(l), FloatValue::F64(r)) => l == r,
            // Different typed floats are never equal
            _ => return ctx.not_equal(self, other),
        };

        if equal {
            ctx.equal()
        } else {
            ctx.not_equal(self, other)
        }
    }
}

define_interface! {
    struct FloatTypeData,
    parent: ValueTypeData,
    pub(crate) mod float_interface {
        pub(crate) mod methods {
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {
            fn add(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<FloatValue> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_operation(right, |a, b| a + b),
                    FloatValue::F32(left) => left.paired_operation_no_overflow(right, |a, b| a + b),
                    FloatValue::F64(left) => left.paired_operation_no_overflow(right, |a, b| a + b),
                }
            }

            [context] fn add_assign(left: Assignee<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<()> {
                FloatValue::assign_op(left, right, context, add)
            }

            fn sub(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<FloatValue> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_operation(right, |a, b| a - b),
                    FloatValue::F32(left) => left.paired_operation_no_overflow(right, |a, b| a - b),
                    FloatValue::F64(left) => left.paired_operation_no_overflow(right, |a, b| a - b),
                }
            }

            [context] fn sub_assign(left: Assignee<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<()> {
                FloatValue::assign_op(left, right, context, sub)
            }

            fn mul(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<FloatValue> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_operation(right, |a, b| a * b),
                    FloatValue::F32(left) => left.paired_operation_no_overflow(right, |a, b| a * b),
                    FloatValue::F64(left) => left.paired_operation_no_overflow(right, |a, b| a * b),
                }
            }

            [context] fn mul_assign(left: Assignee<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<()> {
                FloatValue::assign_op(left, right, context, mul)
            }

            fn div(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<FloatValue> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_operation(right, |a, b| a / b),
                    FloatValue::F32(left) => left.paired_operation_no_overflow(right, |a, b| a / b),
                    FloatValue::F64(left) => left.paired_operation_no_overflow(right, |a, b| a / b),
                }
            }

            [context] fn div_assign(left: Assignee<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<()> {
                FloatValue::assign_op(left, right, context, div)
            }

            fn rem(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<FloatValue> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_operation(right, |a, b| a % b),
                    FloatValue::F32(left) => left.paired_operation_no_overflow(right, |a, b| a % b),
                    FloatValue::F64(left) => left.paired_operation_no_overflow(right, |a, b| a % b),
                }
            }

            [context] fn rem_assign(left: Assignee<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<()> {
                FloatValue::assign_op(left, right, context, rem)
            }

            fn lt(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<bool> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_comparison(right, |a, b| a < b),
                    FloatValue::F32(left) => left.paired_comparison(right, |a, b| a < b),
                    FloatValue::F64(left) => left.paired_comparison(right, |a, b| a < b),
                }
            }

            fn le(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<bool> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_comparison(right, |a, b| a <= b),
                    FloatValue::F32(left) => left.paired_comparison(right, |a, b| a <= b),
                    FloatValue::F64(left) => left.paired_comparison(right, |a, b| a <= b),
                }
            }

            fn gt(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<bool> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_comparison(right, |a, b| a > b),
                    FloatValue::F32(left) => left.paired_comparison(right, |a, b| a > b),
                    FloatValue::F64(left) => left.paired_comparison(right, |a, b| a > b),
                }
            }

            fn ge(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<bool> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_comparison(right, |a, b| a >= b),
                    FloatValue::F32(left) => left.paired_comparison(right, |a, b| a >= b),
                    FloatValue::F64(left) => left.paired_comparison(right, |a, b| a >= b),
                }
            }

            fn eq(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<bool> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_comparison(right, |a, b| a == b),
                    FloatValue::F32(left) => left.paired_comparison(right, |a, b| a == b),
                    FloatValue::F64(left) => left.paired_comparison(right, |a, b| a == b),
                }
            }

            fn ne(left: Owned<FloatValue>, right: Owned<FloatValue>) -> ExecutionResult<bool> {
                match FloatValue::resolve_untyped_to_match(left, &right)? {
                    FloatValue::Untyped(left) => left.paired_comparison(right, |a, b| a != b),
                    FloatValue::F32(left) => left.paired_comparison(right, |a, b| a != b),
                    FloatValue::F64(left) => left.paired_comparison(right, |a, b| a != b),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FloatKind {
    Untyped,
    F32,
    F64,
}

impl IsSpecificValueKind for FloatKind {
    fn display_name(&self) -> &'static str {
        match self {
            FloatKind::Untyped => "untyped float",
            FloatKind::F32 => "f32",
            FloatKind::F64 => "f64",
        }
    }

    fn articled_display_name(&self) -> &'static str {
        match self {
            FloatKind::Untyped => "an untyped float",
            FloatKind::F32 => "an f32",
            FloatKind::F64 => "an f64",
        }
    }
}

impl From<FloatKind> for ValueKind {
    fn from(kind: FloatKind) -> Self {
        ValueKind::Float(kind)
    }
}

impl FloatKind {
    pub(super) fn method_resolver(&self) -> &'static dyn MethodResolver {
        static UNTYPED: UntypedFloatTypeData = UntypedFloatTypeData;
        static F32: F32TypeData = F32TypeData;
        static F64: F64TypeData = F64TypeData;
        match self {
            FloatKind::Untyped => &UNTYPED,
            FloatKind::F32 => &F32,
            FloatKind::F64 => &F64,
        }
    }
}

impl_resolvable_argument_for! {
    FloatTypeData,
    (value, context) -> FloatValue {
        match value {
            Value::Float(value) => Ok(value),
            other => context.err("a float", other),
        }
    }
}
