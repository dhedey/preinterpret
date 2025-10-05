use super::*;

#[derive(Clone)]
pub(crate) enum ExpressionValue {
    None,
    Integer(ExpressionInteger),
    Float(ExpressionFloat),
    Boolean(ExpressionBoolean),
    String(ExpressionString),
    Char(ExpressionChar),
    // Unsupported literal is a type here so that we can parse such a token
    // as a value rather than a stream, and give it better error messages
    UnsupportedLiteral(UnsupportedLiteral),
    Array(ExpressionArray),
    Object(ExpressionObject),
    Stream(ExpressionStream),
    Range(ExpressionRange),
    Iterator(ExpressionIterator),
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
    Range,
    Iterator,
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
            ValueKind::Range => &RANGE,
            ValueKind::Iterator => &ITERATOR,
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
            // It's super common to want to embed a stream in another stream
            // Having to embed it as #(type_name.clone()) instead of
            // #type_name would be awkward
            ValueKind::Stream => true,
            ValueKind::Range => true,
            ValueKind::Iterator => false,
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

    fn resolve_binary_operation(&self, operation: &BinaryOperation) -> Option<MethodInterface> {
        self.method_resolver().resolve_binary_operation(operation)
    }
}

define_interface! {
    struct NoneTypeData,
    parent: ValueTypeData,
    pub(crate) mod none_interface {
        pub(crate) mod methods {}
        pub(crate) mod unary_operations {}
        interface_items {}
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

            fn take_owned(mut this: MutableValue) -> ExpressionValue {
                core::mem::replace(this.deref_mut(), ExpressionValue::None)
            }

            fn as_mut(this: OwnedValue) -> MutableValue {
                MutableValue::new_from_owned(this)
            }

            // NOTE:
            // All value types can be coerced into SharedValue as an input, so this method does actually do something
            fn as_ref(this: SharedValue) -> SharedValue {
                this
            }

            fn swap(mut a: MutableValue, mut b: MutableValue) -> () {
                core::mem::swap(a.deref_mut(), b.deref_mut());
            }

            fn debug(this: CopyOnWriteValue) -> ExecutionResult<()> {
                let (value, span_range) = this.into_owned_infallible().deconstruct();
                let message = value.concat_recursive(&ConcatBehaviour::debug(span_range))?;
                span_range.execution_err(message)
            }

            fn to_debug_string(this: CopyOnWriteValue) -> ExecutionResult<String> {
                let (value, span_range) = this.into_owned_infallible().deconstruct();
                value.concat_recursive(&ConcatBehaviour::debug(span_range))
            }

            fn to_stream(input: ExpressionValue) -> ExecutionResult<OutputStream> {
                input.into_new_output_stream(Grouping::Flattened)
            }

            fn to_group(input: ExpressionValue) -> ExecutionResult<OutputStream> {
                input.into_new_output_stream(Grouping::Grouped)
            }

            fn to_string(input: SharedValue) -> ExecutionResult<String> {
                input.concat_recursive(&ConcatBehaviour::standard(input.span_range()))
            }

            // TYPE CHECKING
            // ===============================
            fn is_none(this: SharedValue) -> bool {
                this.is_none()
            }

            // STRING-BASED CONVERSION METHODS
            // ===============================

            [context] fn to_ident(this: ExpressionValue) -> ExecutionResult<Ident> {
                let stream = to_stream(context, this)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident(context, spanned)
            }

            [context] fn to_ident_camel(this: ExpressionValue) -> ExecutionResult<Ident> {
                let stream = to_stream(context, this)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_camel(context, spanned)
            }

            [context] fn to_ident_snake(this: ExpressionValue) -> ExecutionResult<Ident> {
                let stream = to_stream(context, this)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_snake(context, spanned)
            }

            [context] fn to_ident_upper_snake(this: ExpressionValue) -> ExecutionResult<Ident> {
                let stream = to_stream(context, this)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_upper_snake(context, spanned)
            }

            [context] fn to_literal(this: ExpressionValue) -> ExecutionResult<Literal> {
                let stream = to_stream(context, this)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_literal(context, spanned)
            }
        }
        pub(crate) mod unary_operations {
            fn cast_to_string(input: OwnedValue) -> ExecutionResult<String> {
                let (input, span_range) = input.deconstruct();
                input.concat_recursive(&ConcatBehaviour::standard(span_range))
            }

            fn cast_to_stream(input: ExpressionValue) -> ExecutionResult<OutputStream> {
                input.into_new_output_stream(Grouping::Flattened)
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
        }
    }
}

pub(crate) trait ToExpressionValue: Sized {
    fn into_value(self) -> ExpressionValue;
    fn into_owned(self, span_range: impl HasSpanRange) -> Owned<Self> {
        Owned::new(self, span_range.span_range())
    }
    fn into_owned_value(self, span_range: impl HasSpanRange) -> OwnedValue {
        OwnedValue::new(self.into_value(), span_range.span_range())
    }
}

impl ToExpressionValue for () {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::None
    }
}

impl ExpressionValue {
    pub(crate) fn for_literal(literal: Literal) -> OwnedValue {
        // The unwrap should be safe because all Literal should be parsable
        // as syn::Lit; falling back to syn::Lit::Verbatim if necessary.
        Self::for_syn_lit(literal.to_token_stream().source_parse_as().unwrap())
    }

    pub(crate) fn for_syn_lit(lit: syn::Lit) -> OwnedValue {
        // https://docs.rs/syn/latest/syn/enum.Lit.html
        let matched = match &lit {
            Lit::Int(lit) => match ExpressionInteger::for_litint(lit) {
                Ok(int) => Some(int.into_owned_value()),
                Err(_) => None,
            },
            Lit::Float(lit) => match ExpressionFloat::for_litfloat(lit) {
                Ok(float) => Some(float.into_owned_value()),
                Err(_) => None,
            },
            Lit::Bool(lit) => Some(ExpressionBoolean::for_litbool(lit).into_owned_value()),
            Lit::Str(lit) => Some(ExpressionString::for_litstr(lit).into_owned_value()),
            Lit::Char(lit) => Some(ExpressionChar::for_litchar(lit).into_owned_value()),
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
        new_span_range: SpanRange,
    ) -> ExecutionResult<ExpressionValue> {
        let _ = SpanRange::dummy(/* marker to revisit new_span_range*/);
        if !self.kind().supports_transparent_cloning() {
            return new_span_range.execution_err(format!(
                "An owned value is required, but a reference was received, and {} does not support transparent cloning. You may wish to use .take_owned() or .clone() explicitly.",
                self.articled_value_type()
            ));
        }
        Ok(self.clone())
    }

    pub(super) fn expect_value_pair(
        self,
        operation: &impl Operation,
        right: Self,
    ) -> ExecutionResult<ExpressionValuePair> {
        Ok(match (self, right) {
            (ExpressionValue::Integer(left), ExpressionValue::Integer(right)) => {
                let integer_pair = match (left.value, right.value) {
                    (ExpressionIntegerValue::Untyped(untyped_lhs), rhs) => match rhs {
                        ExpressionIntegerValue::Untyped(untyped_rhs) => {
                            ExpressionIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionIntegerValue::U8(rhs) => {
                            ExpressionIntegerValuePair::U8(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U16(rhs) => {
                            ExpressionIntegerValuePair::U16(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U32(rhs) => {
                            ExpressionIntegerValuePair::U32(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U64(rhs) => {
                            ExpressionIntegerValuePair::U64(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::U128(rhs) => {
                            ExpressionIntegerValuePair::U128(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::Usize(rhs) => {
                            ExpressionIntegerValuePair::Usize(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I8(rhs) => {
                            ExpressionIntegerValuePair::I8(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I16(rhs) => {
                            ExpressionIntegerValuePair::I16(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I32(rhs) => {
                            ExpressionIntegerValuePair::I32(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I64(rhs) => {
                            ExpressionIntegerValuePair::I64(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::I128(rhs) => {
                            ExpressionIntegerValuePair::I128(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionIntegerValue::Isize(rhs) => {
                            ExpressionIntegerValuePair::Isize(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, ExpressionIntegerValue::Untyped(untyped_rhs)) => match lhs {
                        ExpressionIntegerValue::Untyped(untyped_lhs) => {
                            ExpressionIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionIntegerValue::U8(lhs) => {
                            ExpressionIntegerValuePair::U8(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U16(lhs) => {
                            ExpressionIntegerValuePair::U16(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U32(lhs) => {
                            ExpressionIntegerValuePair::U32(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U64(lhs) => {
                            ExpressionIntegerValuePair::U64(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::U128(lhs) => {
                            ExpressionIntegerValuePair::U128(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::Usize(lhs) => {
                            ExpressionIntegerValuePair::Usize(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I8(lhs) => {
                            ExpressionIntegerValuePair::I8(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I16(lhs) => {
                            ExpressionIntegerValuePair::I16(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I32(lhs) => {
                            ExpressionIntegerValuePair::I32(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I64(lhs) => {
                            ExpressionIntegerValuePair::I64(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::I128(lhs) => {
                            ExpressionIntegerValuePair::I128(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionIntegerValue::Isize(lhs) => {
                            ExpressionIntegerValuePair::Isize(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (ExpressionIntegerValue::U8(lhs), ExpressionIntegerValue::U8(rhs)) => {
                        ExpressionIntegerValuePair::U8(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U16(lhs), ExpressionIntegerValue::U16(rhs)) => {
                        ExpressionIntegerValuePair::U16(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U32(lhs), ExpressionIntegerValue::U32(rhs)) => {
                        ExpressionIntegerValuePair::U32(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U64(lhs), ExpressionIntegerValue::U64(rhs)) => {
                        ExpressionIntegerValuePair::U64(lhs, rhs)
                    }
                    (ExpressionIntegerValue::U128(lhs), ExpressionIntegerValue::U128(rhs)) => {
                        ExpressionIntegerValuePair::U128(lhs, rhs)
                    }
                    (ExpressionIntegerValue::Usize(lhs), ExpressionIntegerValue::Usize(rhs)) => {
                        ExpressionIntegerValuePair::Usize(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I8(lhs), ExpressionIntegerValue::I8(rhs)) => {
                        ExpressionIntegerValuePair::I8(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I16(lhs), ExpressionIntegerValue::I16(rhs)) => {
                        ExpressionIntegerValuePair::I16(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I32(lhs), ExpressionIntegerValue::I32(rhs)) => {
                        ExpressionIntegerValuePair::I32(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I64(lhs), ExpressionIntegerValue::I64(rhs)) => {
                        ExpressionIntegerValuePair::I64(lhs, rhs)
                    }
                    (ExpressionIntegerValue::I128(lhs), ExpressionIntegerValue::I128(rhs)) => {
                        ExpressionIntegerValuePair::I128(lhs, rhs)
                    }
                    (ExpressionIntegerValue::Isize(lhs), ExpressionIntegerValue::Isize(rhs)) => {
                        ExpressionIntegerValuePair::Isize(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.execution_err(format!("The {} operator cannot infer a common integer operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbolic_description(), left_value.value_type(), right_value.value_type()));
                    }
                };
                ExpressionValuePair::Integer(integer_pair)
            }
            (ExpressionValue::Boolean(left), ExpressionValue::Boolean(right)) => {
                ExpressionValuePair::BooleanPair(left, right)
            }
            (ExpressionValue::Float(left), ExpressionValue::Float(right)) => {
                let float_pair = match (left.value, right.value) {
                    (ExpressionFloatValue::Untyped(untyped_lhs), rhs) => match rhs {
                        ExpressionFloatValue::Untyped(untyped_rhs) => {
                            ExpressionFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionFloatValue::F32(rhs) => {
                            ExpressionFloatValuePair::F32(untyped_lhs.parse_as()?, rhs)
                        }
                        ExpressionFloatValue::F64(rhs) => {
                            ExpressionFloatValuePair::F64(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, ExpressionFloatValue::Untyped(untyped_rhs)) => match lhs {
                        ExpressionFloatValue::Untyped(untyped_lhs) => {
                            ExpressionFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        ExpressionFloatValue::F32(lhs) => {
                            ExpressionFloatValuePair::F32(lhs, untyped_rhs.parse_as()?)
                        }
                        ExpressionFloatValue::F64(lhs) => {
                            ExpressionFloatValuePair::F64(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (ExpressionFloatValue::F32(lhs), ExpressionFloatValue::F32(rhs)) => {
                        ExpressionFloatValuePair::F32(lhs, rhs)
                    }
                    (ExpressionFloatValue::F64(lhs), ExpressionFloatValue::F64(rhs)) => {
                        ExpressionFloatValuePair::F64(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.execution_err(format!("The {} operator cannot infer a common float operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbolic_description(), left_value.value_type(), right_value.value_type()));
                    }
                };
                ExpressionValuePair::Float(float_pair)
            }
            (ExpressionValue::String(left), ExpressionValue::String(right)) => {
                ExpressionValuePair::StringPair(left, right)
            }
            (ExpressionValue::Char(left), ExpressionValue::Char(right)) => {
                ExpressionValuePair::CharPair(left, right)
            }
            (ExpressionValue::Array(left), ExpressionValue::Array(right)) => {
                ExpressionValuePair::ArrayPair(left, right)
            }
            (ExpressionValue::Object(left), ExpressionValue::Object(right)) => {
                ExpressionValuePair::ObjectPair(left, right)
            }
            (ExpressionValue::Stream(left), ExpressionValue::Stream(right)) => {
                ExpressionValuePair::StreamPair(left, right)
            }
            (left, right) => {
                return operation.execution_err(format!("Cannot infer common type from {} {} {}. Consider using `as` to cast the operands to matching types.", left.value_type(), operation.symbolic_description(), right.value_type()));
            }
        })
    }

    pub(crate) fn kind(&self) -> ValueKind {
        match self {
            ExpressionValue::None => ValueKind::None,
            ExpressionValue::Integer(integer) => ValueKind::Integer(integer.value.kind()),
            ExpressionValue::Float(float) => ValueKind::Float(float.value.kind()),
            ExpressionValue::Boolean(_) => ValueKind::Boolean,
            ExpressionValue::String(_) => ValueKind::String,
            ExpressionValue::Char(_) => ValueKind::Char,
            ExpressionValue::Array(_) => ValueKind::Array,
            ExpressionValue::Object(_) => ValueKind::Object,
            ExpressionValue::Stream(_) => ValueKind::Stream,
            ExpressionValue::Range(_) => ValueKind::Range,
            ExpressionValue::Iterator(_) => ValueKind::Iterator,
            ExpressionValue::UnsupportedLiteral(_) => ValueKind::UnsupportedLiteral,
        }
    }

    pub(crate) fn is_none(&self) -> bool {
        matches!(self, ExpressionValue::None)
    }

    pub(crate) fn into_integer(self) -> Option<ExpressionInteger> {
        match self {
            ExpressionValue::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn expect_bool(self, place_descriptor: &str) -> ExecutionResult<ExpressionBoolean> {
        match self {
            ExpressionValue::Boolean(value) => Ok(value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be a boolean, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(crate) fn expect_integer(
        self,
        place_descriptor: &str,
    ) -> ExecutionResult<ExpressionInteger> {
        match self {
            ExpressionValue::Integer(value) => Ok(value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be an integer, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(crate) fn expect_str(&self, place_descriptor: &str) -> ExecutionResult<&str> {
        match self {
            ExpressionValue::String(value) => Ok(&value.value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be a string, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(crate) fn expect_string(self, place_descriptor: &str) -> ExecutionResult<ExpressionString> {
        match self {
            ExpressionValue::String(value) => Ok(value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be a string, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(crate) fn ref_expect_string(
        &self,
        place_descriptor: &str,
    ) -> ExecutionResult<&ExpressionString> {
        match self {
            ExpressionValue::String(value) => Ok(value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be a string, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(crate) fn expect_array(self, place_descriptor: &str) -> ExecutionResult<ExpressionArray> {
        match self {
            ExpressionValue::Array(value) => Ok(value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be an array, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(crate) fn expect_object(self, place_descriptor: &str) -> ExecutionResult<ExpressionObject> {
        match self {
            ExpressionValue::Object(value) => Ok(value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be an object, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(crate) fn expect_stream(self, place_descriptor: &str) -> ExecutionResult<ExpressionStream> {
        match self {
            ExpressionValue::Stream(value) => Ok(value),
            other => SpanRange::dummy(/*other*/).execution_err(format!(
                "{} must be a stream, but it is {}",
                place_descriptor,
                other.articled_value_type(),
            )),
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            ExpressionValue::None => operation.unsupported(self),
            ExpressionValue::Integer(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Float(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Boolean(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::String(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Char(value) => value.handle_integer_binary_operation(right, operation),
            ExpressionValue::UnsupportedLiteral(value) => operation.unsupported(value),
            ExpressionValue::Array(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Object(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Stream(value) => {
                value.handle_integer_binary_operation(right, operation)
            }
            ExpressionValue::Iterator(value) => operation.unsupported(value),
            ExpressionValue::Range(value) => operation.unsupported(value),
        }
    }

    pub(crate) fn into_indexed(
        self,
        access: IndexAccess,
        index: Spanned<&Self>,
    ) -> ExecutionResult<Self> {
        match self {
            ExpressionValue::Array(array) => array.into_indexed(access, index),
            ExpressionValue::Object(object) => object.into_indexed(access, index),
            other => access.execution_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn index_mut(
        &mut self,
        access: IndexAccess,
        index: Spanned<&Self>,
        auto_create: bool,
    ) -> ExecutionResult<&mut Self> {
        match self {
            ExpressionValue::Array(array) => array.index_mut(access, index),
            ExpressionValue::Object(object) => object.index_mut(access, index, auto_create),
            other => access.execution_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn index_ref(
        &self,
        access: IndexAccess,
        index: Spanned<&Self>,
    ) -> ExecutionResult<&Self> {
        match self {
            ExpressionValue::Array(array) => array.index_ref(access, index),
            ExpressionValue::Object(object) => object.index_ref(access, index),
            other => access.execution_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn into_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        match self {
            ExpressionValue::Object(object) => object.into_property(access),
            other => access.execution_err(format!(
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
            ExpressionValue::Object(object) => object.property_mut(access, auto_create),
            other => access.execution_err(format!(
                "Cannot access properties on a {}",
                other.value_type()
            )),
        }
    }

    pub(crate) fn property_ref(&self, access: &PropertyAccess) -> ExecutionResult<&Self> {
        match self {
            ExpressionValue::Object(object) => object.property_ref(access),
            other => access.execution_err(format!(
                "Cannot access properties on a {}",
                other.value_type()
            )),
        }
    }

    pub(crate) fn into_new_output_stream(
        self,
        grouping: Grouping,
    ) -> ExecutionResult<OutputStream> {
        Ok(match (self, grouping) {
            (Self::Stream(value), Grouping::Flattened) => value.value,
            (other, grouping) => {
                let mut output = OutputStream::new();
                other.output_to(grouping, &mut output)?;
                output
            }
        })
    }

    pub(crate) fn output_to(
        &self,
        grouping: Grouping,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match grouping {
            Grouping::Grouped => {
                // Grouping can be important for different values, to ensure they're read atomically
                // when the output stream is viewed as an array/iterable, e.g. in a for loop.
                // * Grouping means -1 is interpreted atomically, rather than as a punct then a number
                // * Grouping means that a stream is interpreted atomically
                output.push_grouped(
                    |inner| self.output_flattened_to(inner),
                    Delimiter::None,
                    Span::dummy(/*self*/),
                )?;
            }
            Grouping::Flattened => {
                self.output_flattened_to(output)?;
            }
        }
        Ok(())
    }

    fn output_flattened_to(&self, output: &mut OutputStream) -> ExecutionResult<()> {
        match self {
            Self::None => {}
            Self::Integer(value) => output.push_literal(value.to_literal()),
            Self::Float(value) => output.push_literal(value.to_literal()),
            Self::Boolean(value) => output.push_ident(value.to_ident()),
            Self::String(value) => output.push_literal(value.to_literal()),
            Self::Char(value) => output.push_literal(value.to_literal()),
            Self::UnsupportedLiteral(literal) => {
                output.extend_raw_tokens(literal.lit.to_token_stream())
            }
            Self::Object(_) => {
                return SpanRange::dummy(/*self*/)
                    .execution_err("Objects cannot be output to a stream");
            }
            Self::Array(array) => array.output_items_to(output, Grouping::Flattened)?,
            Self::Stream(value) => value.value.append_cloned_into(output),
            Self::Iterator(iterator) => iterator
                .clone()
                .output_items_to(output, Grouping::Flattened)?,
            Self::Range(range) => {
                let iterator = ExpressionIterator::new_for_range(range.clone())?;
                iterator.output_items_to(output, Grouping::Flattened)?
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
            ExpressionValue::None => {
                if behaviour.show_none_values {
                    output.push_str("None");
                }
            }
            ExpressionValue::Stream(stream) => {
                stream.concat_recursive_into(output, behaviour);
            }
            ExpressionValue::Array(array) => {
                array.concat_recursive_into(output, behaviour)?;
            }
            ExpressionValue::Object(object) => {
                object.concat_recursive_into(output, behaviour)?;
            }
            ExpressionValue::Iterator(iterator) => {
                iterator.concat_recursive_into(output, behaviour)?;
            }
            ExpressionValue::Range(range) => {
                range.concat_recursive_into(output, behaviour)?;
            }
            ExpressionValue::Integer(_)
            | ExpressionValue::Float(_)
            | ExpressionValue::Char(_)
            | ExpressionValue::Boolean(_)
            | ExpressionValue::UnsupportedLiteral(_)
            | ExpressionValue::String(_) => {
                // This isn't the most efficient, but it's less code and debug doesn't need to be super efficient.
                let mut stream = OutputStream::new();
                self.output_flattened_to(&mut stream)
                    .expect("Non-composite values should all be able to be outputted to a stream");
                stream.concat_recursive_into(output, behaviour);
            }
        }
        Ok(())
    }
}

impl OwnedValue {
    pub(crate) fn expect_any_iterator(
        self,
        resolution_target: &str,
    ) -> ExecutionResult<Owned<ExpressionIterator>> {
        IterableValue::resolve_owned(self, resolution_target)?.try_map(|v, _| v.into_iterator())
    }
}

impl SpannedRefMut<'_, ExpressionValue> {
    pub(super) fn handle_compound_assignment(
        self,
        operation: &CompoundAssignmentOperation,
        right: OwnedValue,
    ) -> ExecutionResult<()> {
        let (mut left, left_span_range) = self.deconstruct();
        match (&mut *left, operation) {
            (ExpressionValue::Stream(left_mut), CompoundAssignmentOperation::Add(_)) => {
                let right = right
                    .into_inner()
                    .expect_stream("The target of += on a stream")?;
                right.value.append_into(&mut left_mut.value);
            }
            (ExpressionValue::Array(left_mut), CompoundAssignmentOperation::Add(_)) => {
                let mut right = right
                    .into_inner()
                    .expect_array("The target of += on an array")?;
                left_mut.items.append(&mut right.items);
            }
            (left_mut, operation) => {
                // Fallback to just clone and use the normal operator
                let left = left_mut.clone();
                *left_mut = operation
                    .to_binary()
                    .evaluate(left.into_owned(left_span_range), right)?
                    .into_value();
            }
        }
        Ok(())
    }
}

impl ToExpressionValue for ExpressionValue {
    fn into_value(self) -> ExpressionValue {
        self
    }
}

#[derive(Copy, Clone)]
pub(crate) enum Grouping {
    Grouped,
    Flattened,
}

impl HasValueType for ExpressionValue {
    fn value_type(&self) -> &'static str {
        match self {
            Self::None => "none value",
            Self::Integer(value) => value.value_type(),
            Self::Float(value) => value.value_type(),
            Self::Boolean(value) => value.value_type(),
            Self::String(value) => value.value_type(),
            Self::Char(value) => value.value_type(),
            Self::UnsupportedLiteral(value) => value.value_type(),
            Self::Array(value) => value.value_type(),
            Self::Object(value) => value.value_type(),
            Self::Stream(value) => value.value_type(),
            Self::Iterator(value) => value.value_type(),
            Self::Range(value) => value.value_type(),
        }
    }
}

pub(super) trait HasValueType {
    fn value_type(&self) -> &'static str;

    fn articled_value_type(&self) -> String {
        let value_type = self.value_type();
        if value_type.is_empty() {
            return value_type.to_string();
        }
        let first_char = value_type.chars().next().unwrap();
        match first_char {
            'a' | 'e' | 'i' | 'o' | 'u' => format!("an {}", value_type),
            _ => format!("a {}", value_type),
        }
    }
}

#[derive(Clone)]
pub(crate) struct UnsupportedLiteral {
    lit: syn::Lit,
}

impl HasValueType for UnsupportedLiteral {
    fn value_type(&self) -> &'static str {
        "unsupported literal"
    }
}

define_interface! {
    struct UnsupportedLiteralTypeData,
    parent: ValueTypeData,
    pub(crate) mod unsupported_literal_interface {
        pub(crate) mod methods {}
        pub(crate) mod unary_operations {}
        interface_items {}
    }
}

pub(super) enum ExpressionValuePair {
    Integer(ExpressionIntegerValuePair),
    Float(ExpressionFloatValuePair),
    BooleanPair(ExpressionBoolean, ExpressionBoolean),
    StringPair(ExpressionString, ExpressionString),
    CharPair(ExpressionChar, ExpressionChar),
    ArrayPair(ExpressionArray, ExpressionArray),
    ObjectPair(ExpressionObject, ExpressionObject),
    StreamPair(ExpressionStream, ExpressionStream),
}

impl ExpressionValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        match self {
            Self::Integer(pair) => pair.handle_paired_binary_operation(operation),
            Self::Float(pair) => pair.handle_paired_binary_operation(operation),
            Self::BooleanPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::StringPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::CharPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::ArrayPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::ObjectPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
            Self::StreamPair(lhs, rhs) => lhs.handle_paired_binary_operation(rhs, operation),
        }
    }
}
