use super::*;

// ============================================================================
// Equality Context - Controls behavior of value equality comparisons
// ============================================================================

/// A segment in the path to the current comparison location.
#[derive(Clone, Debug)]
#[allow(dead_code)] // Infrastructure for TypedEquality path tracking
pub(crate) enum PathSegment {
    ArrayIndex(usize),
    ObjectKey(String),
    IteratorIndex(usize),
    RangeStart,
    RangeEnd,
}

impl PathSegment {
    #[allow(dead_code)] // Used by TypedEquality for error messages
    fn fmt_path(path: &[PathSegment]) -> String {
        let mut result = String::new();
        for segment in path {
            match segment {
                PathSegment::ArrayIndex(i) => result.push_str(&format!("[{}]", i)),
                PathSegment::ObjectKey(k) => {
                    if result.is_empty() {
                        result.push_str(k);
                    } else {
                        result.push_str(&format!(".{}", k));
                    }
                }
                PathSegment::IteratorIndex(i) => result.push_str(&format!("<iter[{}]>", i)),
                PathSegment::RangeStart => result.push_str(".start"),
                PathSegment::RangeEnd => result.push_str(".end"),
            }
        }
        if result.is_empty() {
            "<root>".to_string()
        } else {
            result
        }
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
    fn equal(&mut self) -> Self::Result;

    /// Values of the same type are not equal.
    fn not_equal<T: HasValueKind>(&mut self, lhs: &T, rhs: &T) -> Self::Result;

    /// Values have different types.
    fn type_mismatch<L: HasValueKind, R: HasValueKind>(&mut self, lhs: &L, rhs: &R)
        -> Self::Result;

    /// Arrays or iterators have different lengths.
    fn lengths_unequal(&mut self, lhs_len: usize, rhs_len: usize) -> Self::Result;

    /// Object is missing a key that the other has.
    fn missing_key(&mut self, key: &str) -> Self::Result;

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
    fn equal(&mut self) -> bool {
        true
    }

    #[inline]
    fn not_equal<T: HasValueKind>(&mut self, _lhs: &T, _rhs: &T) -> bool {
        false
    }

    #[inline]
    fn type_mismatch<L: HasValueKind, R: HasValueKind>(&mut self, _lhs: &L, _rhs: &R) -> bool {
        false
    }

    #[inline]
    fn lengths_unequal(&mut self, _lhs_len: usize, _rhs_len: usize) -> bool {
        false
    }

    #[inline]
    fn missing_key(&mut self, _key: &str) -> bool {
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
#[allow(dead_code)] // Infrastructure for strict equality comparisons
pub(crate) struct TypedEquality {
    pub(crate) path: Vec<PathSegment>,
    pub(crate) error_span: SpanRange,
}

impl TypedEquality {
    #[allow(dead_code)] // Infrastructure for strict equality comparisons
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
    fn equal(&mut self) -> ExecutionResult<bool> {
        Ok(true)
    }

    #[inline]
    fn not_equal<T: HasValueKind>(&mut self, _lhs: &T, _rhs: &T) -> ExecutionResult<bool> {
        Ok(false)
    }

    fn type_mismatch<L: HasValueKind, R: HasValueKind>(
        &mut self,
        lhs: &L,
        rhs: &R,
    ) -> ExecutionResult<bool> {
        let path_str = if self.path.is_empty() {
            String::new()
        } else {
            format!(" at {}", PathSegment::fmt_path(&self.path))
        };
        Err(self.error_span.type_error(format!(
            "Cannot compare {} with {}{}",
            lhs.articled_value_type(),
            rhs.articled_value_type(),
            path_str
        )))
    }

    #[inline]
    fn lengths_unequal(&mut self, _lhs_len: usize, _rhs_len: usize) -> ExecutionResult<bool> {
        Ok(false)
    }

    #[inline]
    fn missing_key(&mut self, _key: &str) -> ExecutionResult<bool> {
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
pub(crate) trait ValuesEqual: Sized + HasValueKind {
    /// Compare two values for equality using the given context.
    fn values_equal<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result;

    /// Lenient equality - returns `false` for incompatible types instead of erroring.
    /// Behaves like JavaScript's `===` operator.
    fn values_eq(&self, other: &Self) -> bool {
        self.values_equal(other, &mut SimpleEquality)
    }

    /// Strict equality check that errors on incompatible types.
    #[allow(dead_code)] // Infrastructure for strict equality comparisons
    fn typed_eq(&self, other: &Self, error_span: SpanRange) -> ExecutionResult<bool> {
        self.values_equal(other, &mut TypedEquality::new(error_span))
    }
}

#[derive(Clone)]
pub(crate) enum Value {
    None,
    Integer(IntegerValue),
    Float(FloatValue),
    Boolean(BooleanValue),
    String(StringValue),
    Char(CharValue),
    // Unsupported literal is a type here so that we can parse such a token
    // as a value rather than a stream, and give it better error messages
    UnsupportedLiteral(UnsupportedLiteral),
    Array(ArrayValue),
    Object(ObjectValue),
    Stream(StreamValue),
    Range(RangeValue),
    Iterator(IteratorValue),
    Parser(ParserValue),
}

/// A trait for specific value kinds that can provide a display name.
/// This is implemented by `ValueKind`, `IntegerKind`, `FloatKind`, etc.
pub(crate) trait IsSpecificValueKind: Copy + Into<ValueKind> {
    fn display_name(&self) -> &'static str;

    fn articled_display_name(&self) -> &'static str;
}

/// A trait for types that have a value kind.
pub(crate) trait HasValueKind {
    type SpecificKind: IsSpecificValueKind;

    fn kind(&self) -> Self::SpecificKind;

    fn value_kind(&self) -> ValueKind {
        self.kind().into()
    }

    fn value_type(&self) -> &'static str {
        self.kind().display_name()
    }

    fn articled_value_type(&self) -> &'static str {
        self.kind().articled_display_name()
    }
}

impl<T: HasValueKind> HasValueKind for &T {
    type SpecificKind = T::SpecificKind;

    fn kind(&self) -> Self::SpecificKind {
        (**self).kind()
    }
}

impl<T: HasValueKind> HasValueKind for &mut T {
    type SpecificKind = T::SpecificKind;

    fn kind(&self) -> Self::SpecificKind {
        (**self).kind()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    None,
    Integer(IntegerKind),
    Float(FloatKind),
    Boolean,
    String,
    Char,
    UnsupportedLiteral,
    Array,
    Object,
    Stream,
    Range(RangeKind),
    Iterator,
    Parser,
}

impl IsSpecificValueKind for ValueKind {
    fn display_name(&self) -> &'static str {
        match self {
            ValueKind::None => "None",
            ValueKind::Integer(kind) => kind.display_name(),
            ValueKind::Float(kind) => kind.display_name(),
            ValueKind::Boolean => "bool",
            ValueKind::String => "string",
            ValueKind::Char => "char",
            ValueKind::UnsupportedLiteral => "unsupported literal",
            ValueKind::Array => "array",
            ValueKind::Object => "object",
            ValueKind::Stream => "stream",
            ValueKind::Range(kind) => kind.display_name(),
            ValueKind::Iterator => "iterator",
            ValueKind::Parser => "parser",
        }
    }

    fn articled_display_name(&self) -> &'static str {
        match self {
            // Instead of saying "expected a none value", we can say "expected None"
            ValueKind::None => "None",
            ValueKind::Integer(kind) => kind.articled_display_name(),
            ValueKind::Float(kind) => kind.articled_display_name(),
            ValueKind::Boolean => "a bool",
            ValueKind::String => "a string",
            ValueKind::Char => "a char",
            ValueKind::UnsupportedLiteral => "an unsupported literal",
            ValueKind::Array => "an array",
            ValueKind::Object => "an object",
            ValueKind::Stream => "a stream",
            ValueKind::Range(kind) => kind.articled_display_name(),
            ValueKind::Iterator => "an iterator",
            ValueKind::Parser => "a parser",
        }
    }
}

impl ValueKind {
    fn method_resolver(&self) -> &'static dyn MethodResolver {
        static NONE: NoneTypeData = NoneTypeData;
        static BOOLEAN: BooleanTypeData = BooleanTypeData;
        static STRING: StringTypeData = StringTypeData;
        static CHAR: CharTypeData = CharTypeData;
        static UNSUPPORTED_LITERAL: UnsupportedLiteralTypeData = UnsupportedLiteralTypeData;
        static ARRAY: ArrayTypeData = ArrayTypeData;
        static OBJECT: ObjectTypeData = ObjectTypeData;
        static STREAM: StreamTypeData = StreamTypeData;
        static RANGE: RangeTypeData = RangeTypeData;
        static ITERATOR: IteratorTypeData = IteratorTypeData;
        static PARSER: ParserTypeData = ParserTypeData;
        match self {
            ValueKind::None => &NONE,
            ValueKind::Integer(kind) => kind.method_resolver(),
            ValueKind::Float(kind) => kind.method_resolver(),
            ValueKind::Boolean => &BOOLEAN,
            ValueKind::String => &STRING,
            ValueKind::Char => &CHAR,
            ValueKind::UnsupportedLiteral => &UNSUPPORTED_LITERAL,
            ValueKind::Array => &ARRAY,
            ValueKind::Object => &OBJECT,
            ValueKind::Stream => &STREAM,
            ValueKind::Range(_) => &RANGE,
            ValueKind::Iterator => &ITERATOR,
            ValueKind::Parser => &PARSER,
        }
    }

    /// This should be true for types which users expect to have value
    /// semantics, but false for types which are expensive to clone or
    /// are expected to have reference semantics.
    ///
    /// This indicates if an &x can be converted to an x via cloning
    /// when doing method resolution.
    fn supports_transparent_cloning(&self) -> bool {
        match self {
            ValueKind::None => true,
            ValueKind::Integer(_) => true,
            ValueKind::Float(_) => true,
            ValueKind::Boolean => true,
            // Strings are value-like, so it makes sense to transparently clone them
            ValueKind::String => true,
            ValueKind::Char => true,
            ValueKind::UnsupportedLiteral => false,
            ValueKind::Array => false,
            ValueKind::Object => false,
            ValueKind::Stream => false,
            ValueKind::Range(_) => true,
            ValueKind::Iterator => false,
            // A parser is a handle, so can be cloned transparently.
            // It may fail to be able to be used to parse if the underlying stream is out of scope of course.
            ValueKind::Parser => true,
        }
    }
}

impl MethodResolver for ValueKind {
    fn resolve_method(&self, method_name: &str) -> Option<MethodInterface> {
        self.method_resolver().resolve_method(method_name)
    }

    fn resolve_unary_operation(
        &self,
        operation: &UnaryOperation,
    ) -> Option<UnaryOperationInterface> {
        self.method_resolver().resolve_unary_operation(operation)
    }

    fn resolve_binary_operation(
        &self,
        operation: &BinaryOperation,
    ) -> Option<BinaryOperationInterface> {
        self.method_resolver().resolve_binary_operation(operation)
    }
}

define_interface! {
    struct ValueTypeData,
    parent: ValueTypeData,
    pub(crate) mod value_interface {
        pub(crate) mod methods {
            fn clone(this: CopyOnWriteValue) -> OwnedValue {
                this.into_owned_infallible()
            }

            fn as_mut(this: ArgumentValue) -> ExecutionResult<MutableValue> {
                Ok(match this {
                    ArgumentValue::Owned(owned) => Mutable::new_from_owned(owned),
                    ArgumentValue::CopyOnWrite(copy_on_write) => ArgumentOwnership::Mutable.map_from_copy_on_write(copy_on_write)?.expect_mutable(),
                    ArgumentValue::Mutable(mutable) => mutable,
                    ArgumentValue::Assignee(assignee) => assignee.0,
                    ArgumentValue::Shared(shared) => ArgumentOwnership::Mutable.map_from_shared(shared)?.expect_mutable(),
                })
            }

            // NOTE:
            // All value types can be coerced into SharedValue as an input, so this method does actually do something
            fn as_ref(this: SharedValue) -> SharedValue {
                this
            }

            fn swap(mut a: AssigneeValue, mut b: AssigneeValue) -> () {
                core::mem::swap(a.0.deref_mut(), b.0.deref_mut());
            }

            fn replace(mut a: AssigneeValue, b: Value) -> Value {
                core::mem::replace(a.0.deref_mut(), b)
            }

            fn debug(this: CopyOnWriteValue) -> ExecutionResult<()> {
                let (value, span_range) = this.into_owned_infallible().deconstruct();
                let message = value.concat_recursive(&ConcatBehaviour::debug(span_range))?;
                span_range.debug_err(message)
            }

            fn to_debug_string(this: CopyOnWriteValue) -> ExecutionResult<String> {
                let (value, span_range) = this.into_owned_infallible().deconstruct();
                value.concat_recursive(&ConcatBehaviour::debug(span_range))
            }

            fn to_stream(input: CopyOnWriteValue) -> ExecutionResult<OutputStream> {
                input.map_into(
                    |shared| shared.output_to_new_stream(Grouping::Flattened, shared.span_range()),
                    |owned| owned.value.into_stream(Grouping::Flattened, owned.span_range),
                )
            }

            fn to_group(input: CopyOnWriteValue) -> ExecutionResult<OutputStream> {
                input.map_into(
                    |shared| shared.output_to_new_stream(Grouping::Grouped, shared.span_range()),
                    |owned| owned.value.into_stream(Grouping::Grouped, owned.span_range),
                )
            }

            fn to_string(input: SharedValue) -> ExecutionResult<String> {
                input.concat_recursive(&ConcatBehaviour::standard(input.span_range()))
            }

            [context] fn with_span(this: CopyOnWriteValue, spans: AnyRef<StreamValue>) -> ExecutionResult<OutputStream> {
                let mut this = to_stream(context, this)?;
                let span_to_use = match spans.resolve_content_span_range() {
                    Some(span_range) => span_range.span_from_join_else_start(),
                    None => Span::call_site(),
                };
                this.replace_first_level_spans(span_to_use);
                Ok(this)
            }

            // TYPE CHECKING
            // ===============================
            fn is_none(this: SharedValue) -> bool {
                this.is_none()
            }

            // STRING-BASED CONVERSION METHODS
            // ===============================

            [context] fn to_ident(this: OwnedValue) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident(context, spanned)
            }

            [context] fn to_ident_camel(this: OwnedValue) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_camel(context, spanned)
            }

            [context] fn to_ident_snake(this: OwnedValue) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_snake(context, spanned)
            }

            [context] fn to_ident_upper_snake(this: OwnedValue) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_upper_snake(context, spanned)
            }

            // Some literals become Value::UnsupportedLiteral but can still be round-tripped back to a stream
            [context] fn to_literal(this: OwnedValue) -> ExecutionResult<Value> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_literal(context, spanned)
            }
        }
        pub(crate) mod unary_operations {
            fn cast_to_string(input: OwnedValue) -> ExecutionResult<String> {
                let (input, span_range) = input.deconstruct();
                input.concat_recursive(&ConcatBehaviour::standard(span_range))
            }

            fn cast_to_stream(input: OwnedValue) -> ExecutionResult<OutputStream> {
                input.into_stream()
            }
        }
        pub(crate) mod binary_operations {
            fn eq(lhs: AnyRef<Value>, rhs: AnyRef<Value>) -> bool {
                Value::values_equal(&lhs, &rhs)
            }

            fn ne(lhs: AnyRef<Value>, rhs: AnyRef<Value>) -> bool {
                !Value::values_equal(&lhs, &rhs)
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::String => unary_definitions::cast_to_string(),
                        CastTarget::Stream => unary_definitions::cast_to_stream(),
                        _ => return None,
                    },
                    _ => return None,
                })
            }

            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                Some(match operation {
                    BinaryOperation::Equal { .. } => binary_definitions::eq(),
                    BinaryOperation::NotEqual { .. } => binary_definitions::ne(),
                    _ => return None,
                })
            }
        }
    }
}

pub(crate) trait IntoValue: Sized {
    fn into_value(self) -> Value;
    fn into_owned(self, span_range: impl HasSpanRange) -> Owned<Self> {
        Owned::new(self, span_range.span_range())
    }
    fn into_owned_value(self, span_range: impl HasSpanRange) -> OwnedValue {
        OwnedValue::new(self.into_value(), span_range.span_range())
    }
}

impl Value {
    pub(crate) fn for_literal(literal: Literal) -> OwnedValue {
        // The unwrap should be safe because all Literal should be parsable
        // as syn::Lit; falling back to syn::Lit::Verbatim if necessary.
        Self::for_syn_lit(
            literal
                .to_token_stream()
                .interpreted_parse_with(|input| input.parse())
                .unwrap(),
        )
    }

    pub(crate) fn for_syn_lit(lit: syn::Lit) -> OwnedValue {
        // https://docs.rs/syn/latest/syn/enum.Lit.html
        let matched = match &lit {
            Lit::Int(lit) => match IntegerValue::for_litint(lit) {
                Ok(int) => Some(int.into_owned_value()),
                Err(_) => None,
            },
            Lit::Float(lit) => match FloatValue::for_litfloat(lit) {
                Ok(float) => Some(float.into_owned_value()),
                Err(_) => None,
            },
            Lit::Bool(lit) => Some(BooleanValue::for_litbool(lit).into_owned_value()),
            Lit::Str(lit) => Some(StringValue::for_litstr(lit).into_owned_value()),
            Lit::Char(lit) => Some(CharValue::for_litchar(lit).into_owned_value()),
            _ => None,
        };
        match matched {
            Some(value) => value,
            None => {
                let span = lit.span();
                Self::UnsupportedLiteral(UnsupportedLiteral { lit }).into_owned(span)
            }
        }
    }

    pub(crate) fn try_transparent_clone(
        &self,
        error_span_range: SpanRange,
    ) -> ExecutionResult<Value> {
        if !self.value_kind().supports_transparent_cloning() {
            return error_span_range.ownership_err(format!(
                "An owned value is required, but a reference was received, and {} does not support transparent cloning. You may wish to use .clone() explicitly.",
                self.articled_value_type()
            ));
        }
        Ok(self.clone())
    }

    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }

    /// Recursively compares two values for equality using `ValuesEqual` semantics.
    pub(crate) fn values_equal(lhs: &Value, rhs: &Value) -> bool {
        lhs.values_eq(rhs)
    }
}

impl ValuesEqual for Value {
    fn values_equal<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        match (self, other) {
            // Same type comparisons - delegate to type-specific implementations
            (Value::None, Value::None) => ctx.equal(),
            (Value::Boolean(l), Value::Boolean(r)) => l.values_equal(r, ctx),
            (Value::Char(l), Value::Char(r)) => l.values_equal(r, ctx),
            (Value::String(l), Value::String(r)) => l.values_equal(r, ctx),
            (Value::Integer(l), Value::Integer(r)) => l.values_equal(r, ctx),
            (Value::Float(l), Value::Float(r)) => l.values_equal(r, ctx),
            (Value::Array(l), Value::Array(r)) => l.values_equal(r, ctx),
            (Value::Object(l), Value::Object(r)) => l.values_equal(r, ctx),
            (Value::Stream(l), Value::Stream(r)) => l.values_equal(r, ctx),
            (Value::Range(l), Value::Range(r)) => l.values_equal(r, ctx),
            (Value::UnsupportedLiteral(l), Value::UnsupportedLiteral(r)) => l.values_equal(r, ctx),
            (Value::Parser(l), Value::Parser(r)) => l.values_equal(r, ctx),
            (Value::Iterator(l), Value::Iterator(r)) => l.values_equal(r, ctx),
            // Different types - use explicit cases to ensure new variants cause compile errors
            (Value::None, _) => ctx.type_mismatch(self, other),
            (Value::Boolean(_), _) => ctx.type_mismatch(self, other),
            (Value::Char(_), _) => ctx.type_mismatch(self, other),
            (Value::String(_), _) => ctx.type_mismatch(self, other),
            (Value::Integer(_), _) => ctx.type_mismatch(self, other),
            (Value::Float(_), _) => ctx.type_mismatch(self, other),
            (Value::Array(_), _) => ctx.type_mismatch(self, other),
            (Value::Object(_), _) => ctx.type_mismatch(self, other),
            (Value::Stream(_), _) => ctx.type_mismatch(self, other),
            (Value::Range(_), _) => ctx.type_mismatch(self, other),
            (Value::UnsupportedLiteral(_), _) => ctx.type_mismatch(self, other),
            (Value::Parser(_), _) => ctx.type_mismatch(self, other),
            (Value::Iterator(_), _) => ctx.type_mismatch(self, other),
        }
    }
}

impl Value {
    pub(crate) fn into_indexed(
        self,
        access: IndexAccess,
        index: Spanned<&Self>,
    ) -> ExecutionResult<Self> {
        match self {
            Value::Array(array) => array.into_indexed(index),
            Value::Object(object) => object.into_indexed(index),
            other => access.type_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn index_mut(
        &mut self,
        access: IndexAccess,
        index: Spanned<&Self>,
        auto_create: bool,
    ) -> ExecutionResult<&mut Self> {
        match self {
            Value::Array(array) => array.index_mut(index),
            Value::Object(object) => object.index_mut(index, auto_create),
            other => access.type_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn index_ref(
        &self,
        access: IndexAccess,
        index: Spanned<&Self>,
    ) -> ExecutionResult<&Self> {
        match self {
            Value::Array(array) => array.index_ref(index),
            Value::Object(object) => object.index_ref(index),
            other => access.type_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn into_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        match self {
            Value::Object(object) => object.into_property(access),
            other => access.type_err(format!(
                "Cannot access properties on a {}",
                other.value_type()
            )),
        }
    }

    pub(crate) fn property_mut(
        &mut self,
        access: &PropertyAccess,
        auto_create: bool,
    ) -> ExecutionResult<&mut Self> {
        match self {
            Value::Object(object) => object.property_mut(access, auto_create),
            other => access.type_err(format!(
                "Cannot access properties on a {}",
                other.value_type()
            )),
        }
    }

    pub(crate) fn property_ref(&self, access: &PropertyAccess) -> ExecutionResult<&Self> {
        match self {
            Value::Object(object) => object.property_ref(access),
            other => access.type_err(format!(
                "Cannot access properties on a {}",
                other.value_type()
            )),
        }
    }

    pub(crate) fn into_stream(
        self,
        grouping: Grouping,
        error_span_range: SpanRange,
    ) -> ExecutionResult<OutputStream> {
        match (self, grouping) {
            (Self::Stream(value), Grouping::Flattened) => Ok(value.value),
            (Self::Stream(value), Grouping::Grouped) => {
                let mut output = OutputStream::new();
                let span = ToStreamContext::new(&mut output, error_span_range).new_token_span();
                output.push_new_group(value.value, Delimiter::None, span);
                Ok(output)
            }
            (other, grouping) => other.output_to_new_stream(grouping, error_span_range),
        }
    }

    pub(crate) fn output_to_new_stream(
        &self,
        grouping: Grouping,
        error_span_range: SpanRange,
    ) -> ExecutionResult<OutputStream> {
        let mut output = OutputStream::new();
        self.output_to(
            grouping,
            &mut ToStreamContext::new(&mut output, error_span_range),
        )?;
        Ok(output)
    }

    pub(crate) fn output_to(
        &self,
        grouping: Grouping,
        output: &mut ToStreamContext,
    ) -> ExecutionResult<()> {
        match grouping {
            Grouping::Grouped => {
                // Grouping can be important for different values, to ensure they're read atomically
                // when the output stream is viewed as an array/iterable, e.g. in a for loop.
                // * Grouping means -1 is interpreted atomically, rather than as a punct then a number
                // * Grouping means that a stream is interpreted atomically
                output.push_grouped(|inner| self.output_flattened_to(inner), Delimiter::None)?;
            }
            Grouping::Flattened => {
                self.output_flattened_to(output)?;
            }
        }
        Ok(())
    }

    fn output_flattened_to(&self, output: &mut ToStreamContext) -> ExecutionResult<()> {
        match self {
            Self::None => {}
            Self::Integer(value) => {
                let literal = value.to_literal(output.new_token_span());
                output.push_literal(literal);
            }
            Self::Float(value) => {
                let literal = value.to_literal(output.new_token_span());
                output.push_literal(literal);
            }
            Self::Boolean(value) => {
                let ident = value.to_ident(output.new_token_span());
                output.push_ident(ident);
            }
            Self::String(value) => {
                let literal = value.to_literal(output.new_token_span());
                output.push_literal(literal);
            }
            Self::Char(value) => {
                let literal = value.to_literal(output.new_token_span());
                output.push_literal(literal);
            }
            Self::UnsupportedLiteral(literal) => {
                output.extend_raw_tokens(literal.lit.to_token_stream())
            }
            Self::Object(_) => {
                return output.type_err("Objects cannot be output to a stream");
            }
            Self::Array(array) => array.output_items_to(output, Grouping::Flattened)?,
            Self::Stream(value) => value.value.append_cloned_into(output.output_stream),
            Self::Iterator(iterator) => iterator
                .clone()
                .output_items_to(output, Grouping::Flattened)?,
            Self::Range(range) => {
                let iterator = IteratorValue::new_for_range(range.clone())?;
                iterator.output_items_to(output, Grouping::Flattened)?
            }
            Self::Parser(_) => {
                return output.type_err("Parsers cannot be output to a stream");
            }
        };
        Ok(())
    }

    pub(crate) fn concat_recursive(&self, behaviour: &ConcatBehaviour) -> ExecutionResult<String> {
        let mut output = String::new();
        self.concat_recursive_into(&mut output, behaviour)?;
        Ok(output)
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        match self {
            Value::None => {
                if behaviour.show_none_values {
                    output.push_str("None");
                }
            }
            Value::Stream(stream) => {
                stream.concat_recursive_into(output, behaviour);
            }
            Value::Array(array) => {
                array.concat_recursive_into(output, behaviour)?;
            }
            Value::Object(object) => {
                object.concat_recursive_into(output, behaviour)?;
            }
            Value::Iterator(iterator) => {
                iterator.concat_recursive_into(output, behaviour)?;
            }
            Value::Range(range) => {
                range.concat_recursive_into(output, behaviour)?;
            }
            Value::Parser(_) => {
                return behaviour
                    .error_span_range
                    .type_err("Parsers cannot be output to a string");
            }
            Value::Integer(_)
            | Value::Float(_)
            | Value::Char(_)
            | Value::Boolean(_)
            | Value::UnsupportedLiteral(_)
            | Value::String(_) => {
                // This isn't the most efficient, but it's less code and debug doesn't need to be super efficient.
                let stream = self
                    .output_to_new_stream(Grouping::Flattened, behaviour.error_span_range)
                    .expect("Non-composite values should all be able to be outputted to a stream");
                stream.concat_recursive_into(output, behaviour);
            }
        }
        Ok(())
    }
}

pub(crate) struct ToStreamContext<'a> {
    output_stream: &'a mut OutputStream,
    error_span_range: SpanRange,
}

impl<'a> ToStreamContext<'a> {
    pub(crate) fn new(output_stream: &'a mut OutputStream, error_span_range: SpanRange) -> Self {
        Self {
            output_stream,
            error_span_range,
        }
    }

    pub(crate) fn push_grouped(
        &mut self,
        f: impl FnOnce(&mut ToStreamContext) -> ExecutionResult<()>,
        delimiter: Delimiter,
    ) -> ExecutionResult<()> {
        let span = self.new_token_span();
        self.output_stream.push_grouped(
            |inner| f(&mut ToStreamContext::new(inner, self.error_span_range)),
            delimiter,
            span,
        )
    }

    pub(crate) fn new_token_span(&self) -> Span {
        // By default, we use call_site span for generated tokens
        Span::call_site()
    }
}

impl Deref for ToStreamContext<'_> {
    type Target = OutputStream;

    fn deref(&self) -> &Self::Target {
        self.output_stream
    }
}

impl DerefMut for ToStreamContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.output_stream
    }
}

impl HasSpanRange for ToStreamContext<'_> {
    fn span_range(&self) -> SpanRange {
        self.error_span_range
    }
}

impl OwnedValue {
    pub(crate) fn into_stream(self) -> ExecutionResult<OutputStream> {
        self.value.into_stream(Grouping::Flattened, self.span_range)
    }

    pub(crate) fn expect_any_iterator(
        self,
        resolution_target: &str,
    ) -> ExecutionResult<Owned<IteratorValue>> {
        IterableValue::resolve_owned(self, resolution_target)?.try_map(|v, _| v.into_iterator())
    }
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

#[derive(Copy, Clone)]
pub(crate) enum Grouping {
    Grouped,
    Flattened,
}

impl HasValueKind for Value {
    type SpecificKind = ValueKind;

    fn kind(&self) -> ValueKind {
        match self {
            Value::None => ValueKind::None,
            Value::Integer(integer) => ValueKind::Integer(integer.kind()),
            Value::Float(float) => ValueKind::Float(float.kind()),
            Value::Boolean(_) => ValueKind::Boolean,
            Value::String(_) => ValueKind::String,
            Value::Char(_) => ValueKind::Char,
            Value::Array(_) => ValueKind::Array,
            Value::Object(_) => ValueKind::Object,
            Value::Stream(_) => ValueKind::Stream,
            Value::Range(range) => ValueKind::Range(range.kind()),
            Value::Iterator(_) => ValueKind::Iterator,
            Value::Parser(_) => ValueKind::Parser,
            Value::UnsupportedLiteral(_) => ValueKind::UnsupportedLiteral,
        }
    }
}
