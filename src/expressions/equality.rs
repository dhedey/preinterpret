use super::*;

// ============================================================================
// Equality Context - Controls behavior of value equality comparisons
// ============================================================================

/// A segment in the path to the current comparison location.
#[derive(Clone, Debug)]
pub(crate) enum PathSegment {
    ArrayIndex(usize),
    ObjectKey(String),
    IteratorIndex(usize),
    RangeStart,
    RangeEnd,
}

impl PathSegment {
    fn fmt_path(path: &[PathSegment]) -> String {
        let mut result = String::new();
        for segment in path {
            match segment {
                PathSegment::ArrayIndex(i) => result.push_str(&format!("[{}]", i)),
                PathSegment::ObjectKey(k) => {
                    if syn::parse_str::<syn::Ident>(k).is_ok() {
                        result.push_str(&format!(".{}", k))
                    } else {
                        result.push_str(&format!("[{:?}]", k))
                    }
                }
                PathSegment::IteratorIndex(i) => result.push_str(&format!("[{}]", i)),
                PathSegment::RangeStart => result.push_str(".start"),
                PathSegment::RangeEnd => result.push_str(".end"),
            }
        }
        result
    }
}

/// Context trait for controlling equality comparison behavior.
///
/// Different implementations allow for:
/// - Simple equality: returns `false` on type mismatch (like JS `===`)
/// - Typed equality: errors on type mismatch, tracks path for error messages
/// - Assert equality: errors on any difference, useful for assert_eq
pub(crate) trait EqualityContext {
    /// The result type returned by equality comparisons.
    type Result;

    /// Values are equal.
    fn values_equal(&mut self) -> Self::Result;

    /// Values of the same type are not equal.
    fn leaf_values_not_equal<T: Debug + ?Sized>(&mut self, lhs: &T, rhs: &T) -> Self::Result;

    /// Values have different kinds.
    fn kind_mismatch<L: HasLeafKind, R: HasLeafKind>(&mut self, lhs: &L, rhs: &R) -> Self::Result;

    /// Range values have different structures.
    fn range_structure_mismatch(
        &mut self,
        lhs: RangeStructure,
        rhs: RangeStructure,
    ) -> Self::Result;

    /// Arrays or iterators have different lengths.
    fn lengths_unequal(&mut self, lhs_len: Option<usize>, rhs_len: Option<usize>) -> Self::Result;

    /// Object is missing a key that the other has.
    fn missing_key(&mut self, key: &str, missing_on: MissingSide) -> Self::Result;

    fn iteration_limit_exceeded(&mut self, limit: usize) -> Self::Result {
        let message = format!("iteration limit {} exceeded", limit);
        self.leaf_values_not_equal(&message, &message)
    }

    /// Wrap a comparison within an array index context.
    fn with_array_index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R;

    /// Wrap a comparison within an object key context.
    fn with_object_key<R>(&mut self, key: &str, f: impl FnOnce(&mut Self) -> R) -> R;

    /// Wrap a comparison within an iterator index context.
    fn with_iterator_index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R;

    /// Wrap a comparison within a range start context.
    fn with_range_start<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R;

    /// Wrap a comparison within a range end context.
    fn with_range_end<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R;

    /// Returns true if the result indicates we should stop comparing and return early.
    /// This is true when the result indicates "not equal" or an error occurred.
    fn should_short_circuit(&self, result: &Self::Result) -> bool;
}

/// Simple equality context - returns `false` on type mismatch, no path tracking.
/// This is the most efficient option when you just need a bool result.
pub(crate) struct SimpleEquality;

impl EqualityContext for SimpleEquality {
    type Result = bool;

    #[inline]
    fn values_equal(&mut self) -> bool {
        true
    }

    #[inline]
    fn leaf_values_not_equal<T: Debug + ?Sized>(&mut self, _lhs: &T, _rhs: &T) -> bool {
        false
    }

    #[inline]
    fn kind_mismatch<L: HasLeafKind, R: HasLeafKind>(&mut self, _lhs: &L, _rhs: &R) -> bool {
        false
    }

    #[inline]
    fn range_structure_mismatch(&mut self, _lhs: RangeStructure, _rhs: RangeStructure) -> bool {
        false
    }

    #[inline]
    fn lengths_unequal(&mut self, _lhs_len: Option<usize>, _rhs_len: Option<usize>) -> bool {
        false
    }

    #[inline]
    fn missing_key(&mut self, _key: &str, _missing_on: MissingSide) -> bool {
        false
    }

    #[inline]
    fn with_array_index<R>(&mut self, _index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        f(self)
    }

    #[inline]
    fn with_object_key<R>(&mut self, _key: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        f(self)
    }

    #[inline]
    fn with_iterator_index<R>(&mut self, _index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        f(self)
    }

    #[inline]
    fn with_range_start<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        f(self)
    }

    #[inline]
    fn with_range_end<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        f(self)
    }

    #[inline]
    fn should_short_circuit(&self, result: &bool) -> bool {
        !*result
    }
}

/// Typed equality context - errors on type mismatch, tracks path for error messages.
pub(crate) struct TypedEquality {
    path: Vec<PathSegment>,
    error_span: SpanRange,
}

impl TypedEquality {
    pub(crate) fn new(error_span: SpanRange) -> Self {
        Self {
            path: Vec::new(),
            error_span,
        }
    }
}

impl EqualityContext for TypedEquality {
    type Result = ExecutionResult<bool>;

    #[inline]
    fn values_equal(&mut self) -> ExecutionResult<bool> {
        Ok(true)
    }

    #[inline]
    fn leaf_values_not_equal<T: Debug + ?Sized>(
        &mut self,
        _lhs: &T,
        _rhs: &T,
    ) -> ExecutionResult<bool> {
        Ok(false)
    }

    fn kind_mismatch<L: HasLeafKind, R: HasLeafKind>(
        &mut self,
        lhs: &L,
        rhs: &R,
    ) -> ExecutionResult<bool> {
        let path_str = PathSegment::fmt_path(&self.path);
        Err(self.error_span.type_error(format!(
            "lhs{} is {}, but rhs{} is {}",
            path_str,
            lhs.kind().articled_value_name(),
            path_str,
            rhs.kind().articled_value_name(),
        )))
    }

    fn range_structure_mismatch(
        &mut self,
        lhs: RangeStructure,
        rhs: RangeStructure,
    ) -> Self::Result {
        let path_str = PathSegment::fmt_path(&self.path);
        Err(self.error_span.type_error(format!(
            "lhs{} is {}, but rhs{} is {}",
            path_str,
            lhs.articled_value_name(),
            path_str,
            rhs.articled_value_name()
        )))
    }

    #[inline]
    fn lengths_unequal(
        &mut self,
        _lhs_len: Option<usize>,
        _rhs_len: Option<usize>,
    ) -> ExecutionResult<bool> {
        Ok(false)
    }

    #[inline]
    fn missing_key(&mut self, _key: &str, _missing_on: MissingSide) -> ExecutionResult<bool> {
        Ok(false)
    }

    #[inline]
    fn with_array_index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::ArrayIndex(index));
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_object_key<R>(&mut self, key: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::ObjectKey(key.to_string()));
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_iterator_index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::IteratorIndex(index));
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_range_start<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::RangeStart);
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_range_end<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::RangeEnd);
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn should_short_circuit(&self, result: &ExecutionResult<bool>) -> bool {
        // Short-circuit on Ok(false) or Err(_)
        !matches!(result, Ok(true))
    }
}

// ============================================================================
// Debug Equality - Captures detailed information about equality failures
// ============================================================================

/// Which side of the comparison is missing a key.
#[derive(Debug, Clone, Copy)]
pub(crate) enum MissingSide {
    #[allow(dead_code)]
    Lhs,
    Rhs,
}

/// The reason why two values were not equal.
#[derive(Clone)]
pub(crate) enum DebugInequalityReason {
    /// Values of the same type have different values.
    ValueMismatch {
        lhs_display: String,
        rhs_display: String,
    },
    /// Values have incompatible value kinds.
    ValueLeafKindMismatch {
        lhs_kind: AnyValueLeafKind,
        rhs_kind: AnyValueLeafKind,
    },
    /// Ranges have incompatible structures.
    RangeStructureMismatch {
        lhs_structure: RangeStructure,
        rhs_structure: RangeStructure,
    },
    /// Collections have different lengths.
    LengthMismatch {
        lhs_len: Option<usize>,
        rhs_len: Option<usize>,
    },
    /// Object is missing a key on one side.
    MissingKey {
        key: String,
        missing_on: MissingSide,
    },
}

pub(crate) struct DebugEqualityError {
    inner: Box<DebugEqualityErrorInner>,
}

struct DebugEqualityErrorInner {
    path: Vec<PathSegment>,
    reason: DebugInequalityReason,
}

impl DebugEqualityError {
    pub fn format_message(&self) -> String {
        let inner = &self.inner;
        let path_str = PathSegment::fmt_path(&inner.path);

        match &inner.reason {
            DebugInequalityReason::ValueMismatch {
                lhs_display,
                rhs_display,
            } => {
                format!(
                    "lhs{} != rhs{}: {} != {}",
                    path_str, path_str, lhs_display, rhs_display
                )
            }
            DebugInequalityReason::ValueLeafKindMismatch { lhs_kind, rhs_kind } => {
                format!(
                    "lhs{} is {}, but rhs{} is {}",
                    path_str,
                    lhs_kind.articled_value_name(),
                    path_str,
                    rhs_kind.articled_value_name()
                )
            }
            DebugInequalityReason::RangeStructureMismatch {
                lhs_structure: lhs_kind,
                rhs_structure: rhs_kind,
            } => {
                format!(
                    "lhs{} is {}, but rhs{} is {}",
                    path_str,
                    lhs_kind.articled_value_name(),
                    path_str,
                    rhs_kind.articled_value_name()
                )
            }
            DebugInequalityReason::LengthMismatch { lhs_len, rhs_len } => {
                format!(
                    "lhs{} has length {}, but rhs{} has length {}",
                    path_str,
                    lhs_len.unwrap_or(0),
                    path_str,
                    rhs_len.unwrap_or(0)
                )
            }
            DebugInequalityReason::MissingKey { key, missing_on } => match missing_on {
                MissingSide::Lhs => {
                    format!(
                        "lhs{} is missing key {:?}, compared to rhs{}",
                        path_str, key, path_str
                    )
                }
                MissingSide::Rhs => {
                    format!(
                        "lhs{} has extra key {:?}, compared to rhs{}",
                        path_str, key, path_str
                    )
                }
            },
        }
    }
}

impl From<DebugEqualityErrorInner> for DebugEqualityError {
    fn from(inner: DebugEqualityErrorInner) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

/// Debug equality context - captures detailed information about why values differ.
/// This is useful for assertion failures where you want to show exactly where
/// the mismatch occurred.
pub(crate) struct DebugEquality {
    path: Vec<PathSegment>,
}

impl DebugEquality {
    pub(crate) fn new() -> Self {
        Self { path: Vec::new() }
    }
}

impl EqualityContext for DebugEquality {
    type Result = Result<(), DebugEqualityError>;

    #[inline]
    fn values_equal(&mut self) -> Result<(), DebugEqualityError> {
        Ok(())
    }

    #[inline]
    fn leaf_values_not_equal<T: Debug + ?Sized>(
        &mut self,
        lhs: &T,
        rhs: &T,
    ) -> Result<(), DebugEqualityError> {
        Err(DebugEqualityErrorInner {
            path: self.path.clone(),
            reason: DebugInequalityReason::ValueMismatch {
                lhs_display: format!("{:?}", lhs),
                rhs_display: format!("{:?}", rhs),
            },
        })?
    }

    fn kind_mismatch<L: HasLeafKind, R: HasLeafKind>(
        &mut self,
        lhs: &L,
        rhs: &R,
    ) -> Result<(), DebugEqualityError> {
        Err(DebugEqualityErrorInner {
            path: self.path.clone(),
            reason: DebugInequalityReason::ValueLeafKindMismatch {
                lhs_kind: lhs.value_kind(),
                rhs_kind: rhs.value_kind(),
            },
        })?
    }

    fn range_structure_mismatch(
        &mut self,
        lhs: RangeStructure,
        rhs: RangeStructure,
    ) -> Result<(), DebugEqualityError> {
        Err(DebugEqualityErrorInner {
            path: self.path.clone(),
            reason: DebugInequalityReason::RangeStructureMismatch {
                lhs_structure: lhs,
                rhs_structure: rhs,
            },
        })?
    }

    #[inline]
    fn lengths_unequal(
        &mut self,
        lhs_len: Option<usize>,
        rhs_len: Option<usize>,
    ) -> Result<(), DebugEqualityError> {
        Err(DebugEqualityErrorInner {
            path: self.path.clone(),
            reason: DebugInequalityReason::LengthMismatch { lhs_len, rhs_len },
        })?
    }

    #[inline]
    fn missing_key(
        &mut self,
        key: &str,
        missing_on: MissingSide,
    ) -> Result<(), DebugEqualityError> {
        Err(DebugEqualityErrorInner {
            path: self.path.clone(),
            reason: DebugInequalityReason::MissingKey {
                key: key.to_string(),
                missing_on,
            },
        })?
    }

    #[inline]
    fn with_array_index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::ArrayIndex(index));
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_object_key<R>(&mut self, key: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::ObjectKey(key.to_string()));
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_iterator_index<R>(&mut self, index: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::IteratorIndex(index));
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_range_start<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::RangeStart);
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn with_range_end<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(PathSegment::RangeEnd);
        let result = f(self);
        self.path.pop();
        result
    }

    #[inline]
    fn should_short_circuit(&self, result: &Result<(), DebugEqualityError>) -> bool {
        result.is_err()
    }
}

// ============================================================================
// ValuesEqual trait - Value equality with configurable context
// ============================================================================

/// A trait for comparing values for equality with preinterpret semantics.
///
/// This is NOT the same as Rust's `PartialEq`/`Eq` traits because:
/// - **Type coercion**: Untyped integers/floats can equal typed ones (e.g., `5 == 5u32`)
/// - **Structural comparison**: Arrays, objects, and iterators are compared element-wise
/// - **Token comparison**: Streams and unsupported literals compare via token string representation
/// - **Float semantics**: Floats use Rust's `==`, so `NaN != NaN`
///
/// The comparison behavior is controlled by the `EqualityContext`:
/// - `SimpleEquality`: Returns `false` on type mismatch (like JS `===`)
/// - `TypedEquality`: Errors on type mismatch with path information
/// - `DebugEquality`: Returns detailed error info for assertion messages
pub(crate) trait ValuesEqual: Sized + HasLeafKind {
    /// Compare two values for equality using the given context.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result;

    /// Lenient equality - returns `false` for incompatible types instead of erroring.
    /// Behaves like JavaScript's `===` operator.
    fn lenient_eq(&self, other: &Self) -> bool {
        self.test_equality(other, &mut SimpleEquality)
    }

    /// Strict equality check that errors on incompatible types.
    fn typed_eq(&self, other: &Self, error_span: SpanRange) -> ExecutionResult<bool> {
        self.test_equality(other, &mut TypedEquality::new(error_span))
    }

    /// Debug equality check - returns detailed information about why values differ.
    /// Useful for generating informative assertion failure messages.
    fn debug_eq(&self, other: &Self) -> Result<(), DebugEqualityError> {
        self.test_equality(other, &mut DebugEquality::new())
    }
}
