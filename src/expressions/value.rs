use super::*;

#[derive(Clone)]
pub(crate) enum ExpressionValue {
    None,
    Integer(IntegerExpression),
    Float(FloatExpression),
    Boolean(BooleanExpression),
    String(StringExpression),
    Char(CharExpression),
    // Unsupported literal is a type here so that we can parse such a token
    // as a value rather than a stream, and give it better error messages
    UnsupportedLiteral(UnsupportedLiteral),
    Array(ArrayExpression),
    Object(ObjectExpression),
    Stream(StreamExpression),
    Range(RangeExpression),
    Iterator(IteratorExpression),
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
            ValueKind::Stream => false,
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

define_optional_object! {
    pub(crate) struct SettingsInputs {
        iteration_limit: usize => (DEFAULT_ITERATION_LIMIT_STR, "The new iteration limit"),
    }
}

define_interface! {
    struct NoneTypeData,
    parent: ValueTypeData,
    pub(crate) mod none_interface {
        pub(crate) mod methods {
            [context] fn configure_preinterpret(_none: (), inputs: SettingsInputs) {
                if let Some(limit) = inputs.iteration_limit {
                    context.interpreter.set_iteration_limit(Some(limit));
                }
            }
        }
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

            fn as_mut(this: ResolvedValue) -> ExecutionResult<MutableValue> {
                Ok(match this {
                    ResolvedValue::Owned(owned) => Mutable::new_from_owned(owned),
                    ResolvedValue::CopyOnWrite(copy_on_write) => ResolvedValueOwnership::Mutable.map_from_copy_on_write(copy_on_write)?.expect_mutable(),
                    ResolvedValue::Mutable(mutable) => mutable,
                    ResolvedValue::Shared(shared) => ResolvedValueOwnership::Mutable.map_from_shared(shared)?.expect_mutable(),
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

            fn replace(mut a: AssigneeValue, b: ExpressionValue) -> ExpressionValue {
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

            [context] fn with_span(this: CopyOnWriteValue, spans: AnyRef<StreamExpression>) -> ExecutionResult<OutputStream> {
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

            [context] fn to_literal(this: OwnedValue) -> ExecutionResult<Literal> {
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
            Lit::Int(lit) => match IntegerExpression::for_litint(lit) {
                Ok(int) => Some(int.into_owned_value()),
                Err(_) => None,
            },
            Lit::Float(lit) => match FloatExpression::for_litfloat(lit) {
                Ok(float) => Some(float.into_owned_value()),
                Err(_) => None,
            },
            Lit::Bool(lit) => Some(BooleanExpression::for_litbool(lit).into_owned_value()),
            Lit::Str(lit) => Some(StringExpression::for_litstr(lit).into_owned_value()),
            Lit::Char(lit) => Some(CharExpression::for_litchar(lit).into_owned_value()),
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
    ) -> ExecutionResult<ExpressionValue> {
        if !self.kind().supports_transparent_cloning() {
            return error_span_range.ownership_err(format!(
                "An owned value is required, but a reference was received, and {} does not support transparent cloning. You may wish to use .clone() explicitly.",
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
                    (IntegerExpressionValue::Untyped(untyped_lhs), rhs) => match rhs {
                        IntegerExpressionValue::Untyped(untyped_rhs) => {
                            IntegerExpressionValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        IntegerExpressionValue::U8(rhs) => {
                            IntegerExpressionValuePair::U8(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::U16(rhs) => {
                            IntegerExpressionValuePair::U16(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::U32(rhs) => {
                            IntegerExpressionValuePair::U32(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::U64(rhs) => {
                            IntegerExpressionValuePair::U64(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::U128(rhs) => {
                            IntegerExpressionValuePair::U128(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::Usize(rhs) => {
                            IntegerExpressionValuePair::Usize(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::I8(rhs) => {
                            IntegerExpressionValuePair::I8(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::I16(rhs) => {
                            IntegerExpressionValuePair::I16(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::I32(rhs) => {
                            IntegerExpressionValuePair::I32(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::I64(rhs) => {
                            IntegerExpressionValuePair::I64(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::I128(rhs) => {
                            IntegerExpressionValuePair::I128(untyped_lhs.parse_as()?, rhs)
                        }
                        IntegerExpressionValue::Isize(rhs) => {
                            IntegerExpressionValuePair::Isize(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, IntegerExpressionValue::Untyped(untyped_rhs)) => match lhs {
                        IntegerExpressionValue::Untyped(untyped_lhs) => {
                            IntegerExpressionValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        IntegerExpressionValue::U8(lhs) => {
                            IntegerExpressionValuePair::U8(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::U16(lhs) => {
                            IntegerExpressionValuePair::U16(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::U32(lhs) => {
                            IntegerExpressionValuePair::U32(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::U64(lhs) => {
                            IntegerExpressionValuePair::U64(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::U128(lhs) => {
                            IntegerExpressionValuePair::U128(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::Usize(lhs) => {
                            IntegerExpressionValuePair::Usize(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::I8(lhs) => {
                            IntegerExpressionValuePair::I8(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::I16(lhs) => {
                            IntegerExpressionValuePair::I16(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::I32(lhs) => {
                            IntegerExpressionValuePair::I32(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::I64(lhs) => {
                            IntegerExpressionValuePair::I64(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::I128(lhs) => {
                            IntegerExpressionValuePair::I128(lhs, untyped_rhs.parse_as()?)
                        }
                        IntegerExpressionValue::Isize(lhs) => {
                            IntegerExpressionValuePair::Isize(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (IntegerExpressionValue::U8(lhs), IntegerExpressionValue::U8(rhs)) => {
                        IntegerExpressionValuePair::U8(lhs, rhs)
                    }
                    (IntegerExpressionValue::U16(lhs), IntegerExpressionValue::U16(rhs)) => {
                        IntegerExpressionValuePair::U16(lhs, rhs)
                    }
                    (IntegerExpressionValue::U32(lhs), IntegerExpressionValue::U32(rhs)) => {
                        IntegerExpressionValuePair::U32(lhs, rhs)
                    }
                    (IntegerExpressionValue::U64(lhs), IntegerExpressionValue::U64(rhs)) => {
                        IntegerExpressionValuePair::U64(lhs, rhs)
                    }
                    (IntegerExpressionValue::U128(lhs), IntegerExpressionValue::U128(rhs)) => {
                        IntegerExpressionValuePair::U128(lhs, rhs)
                    }
                    (IntegerExpressionValue::Usize(lhs), IntegerExpressionValue::Usize(rhs)) => {
                        IntegerExpressionValuePair::Usize(lhs, rhs)
                    }
                    (IntegerExpressionValue::I8(lhs), IntegerExpressionValue::I8(rhs)) => {
                        IntegerExpressionValuePair::I8(lhs, rhs)
                    }
                    (IntegerExpressionValue::I16(lhs), IntegerExpressionValue::I16(rhs)) => {
                        IntegerExpressionValuePair::I16(lhs, rhs)
                    }
                    (IntegerExpressionValue::I32(lhs), IntegerExpressionValue::I32(rhs)) => {
                        IntegerExpressionValuePair::I32(lhs, rhs)
                    }
                    (IntegerExpressionValue::I64(lhs), IntegerExpressionValue::I64(rhs)) => {
                        IntegerExpressionValuePair::I64(lhs, rhs)
                    }
                    (IntegerExpressionValue::I128(lhs), IntegerExpressionValue::I128(rhs)) => {
                        IntegerExpressionValuePair::I128(lhs, rhs)
                    }
                    (IntegerExpressionValue::Isize(lhs), IntegerExpressionValue::Isize(rhs)) => {
                        IntegerExpressionValuePair::Isize(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.type_err(format!("The {} operator cannot infer a common integer operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbolic_description(), left_value.value_type(), right_value.value_type()));
                    }
                };
                ExpressionValuePair::Integer(integer_pair)
            }
            (ExpressionValue::Boolean(left), ExpressionValue::Boolean(right)) => {
                ExpressionValuePair::BooleanPair(left, right)
            }
            (ExpressionValue::Float(left), ExpressionValue::Float(right)) => {
                let float_pair = match (left.value, right.value) {
                    (FloatExpressionValue::Untyped(untyped_lhs), rhs) => match rhs {
                        FloatExpressionValue::Untyped(untyped_rhs) => {
                            FloatExpressionValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        FloatExpressionValue::F32(rhs) => {
                            FloatExpressionValuePair::F32(untyped_lhs.parse_as()?, rhs)
                        }
                        FloatExpressionValue::F64(rhs) => {
                            FloatExpressionValuePair::F64(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, FloatExpressionValue::Untyped(untyped_rhs)) => match lhs {
                        FloatExpressionValue::Untyped(untyped_lhs) => {
                            FloatExpressionValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        FloatExpressionValue::F32(lhs) => {
                            FloatExpressionValuePair::F32(lhs, untyped_rhs.parse_as()?)
                        }
                        FloatExpressionValue::F64(lhs) => {
                            FloatExpressionValuePair::F64(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (FloatExpressionValue::F32(lhs), FloatExpressionValue::F32(rhs)) => {
                        FloatExpressionValuePair::F32(lhs, rhs)
                    }
                    (FloatExpressionValue::F64(lhs), FloatExpressionValue::F64(rhs)) => {
                        FloatExpressionValuePair::F64(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operation.type_err(format!("The {} operator cannot infer a common float operand type from {} and {}. Consider using `as` to cast to matching types.", operation.symbolic_description(), left_value.value_type(), right_value.value_type()));
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
                return operation.type_err(format!("Cannot infer common type from {} {} {}. Consider using `as` to cast the operands to matching types.", left.value_type(), operation.symbolic_description(), right.value_type()));
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

    pub(crate) fn into_integer(self) -> Option<IntegerExpression> {
        match self {
            ExpressionValue::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        right: IntegerExpression,
        operation: WrappedOp<IntegerBinaryOperation>,
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
            ExpressionValue::Array(array) => array.into_indexed(index),
            ExpressionValue::Object(object) => object.into_indexed(index),
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
            ExpressionValue::Array(array) => array.index_mut(index),
            ExpressionValue::Object(object) => object.index_mut(index, auto_create),
            other => access.type_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn index_ref(
        &self,
        access: IndexAccess,
        index: Spanned<&Self>,
    ) -> ExecutionResult<&Self> {
        match self {
            ExpressionValue::Array(array) => array.index_ref(index),
            ExpressionValue::Object(object) => object.index_ref(index),
            other => access.type_err(format!("Cannot index into a {}", other.value_type())),
        }
    }

    pub(crate) fn into_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        match self {
            ExpressionValue::Object(object) => object.into_property(access),
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
            ExpressionValue::Object(object) => object.property_mut(access, auto_create),
            other => access.type_err(format!(
                "Cannot access properties on a {}",
                other.value_type()
            )),
        }
    }

    pub(crate) fn property_ref(&self, access: &PropertyAccess) -> ExecutionResult<&Self> {
        match self {
            ExpressionValue::Object(object) => object.property_ref(access),
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
                let iterator = IteratorExpression::new_for_range(range.clone())?;
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
    ) -> ExecutionResult<Owned<IteratorExpression>> {
        IterableValue::resolve_owned(self, resolution_target)?.try_map(|v, _| v.into_iterator())
    }
}

impl SpannedAnyRefMut<'_, ExpressionValue> {
    pub(super) fn handle_compound_assignment(
        self,
        operation: &CompoundAssignmentOperation,
        right: OwnedValue,
    ) -> ExecutionResult<()> {
        let (mut left, left_span_range) = self.deconstruct();
        match (&mut *left, operation) {
            (ExpressionValue::Stream(left_mut), CompoundAssignmentOperation::Add(_)) => {
                let right: StreamExpression = right.resolve_as("The target of += on a stream")?;
                right.value.append_into(&mut left_mut.value);
            }
            (ExpressionValue::Array(left_mut), CompoundAssignmentOperation::Add(_)) => {
                let mut right: ArrayExpression =
                    right.resolve_as("The target of += on an array")?;
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
        value_type.lower_indefinite_articled()
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
    Integer(IntegerExpressionValuePair),
    Float(FloatExpressionValuePair),
    BooleanPair(BooleanExpression, BooleanExpression),
    StringPair(StringExpression, StringExpression),
    CharPair(CharExpression, CharExpression),
    ArrayPair(ArrayExpression, ArrayExpression),
    ObjectPair(ObjectExpression, ObjectExpression),
    StreamPair(StreamExpression, StreamExpression),
}

impl ExpressionValuePair {
    pub(super) fn handle_paired_binary_operation(
        self,
        operation: WrappedOp<PairedBinaryOperation>,
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
