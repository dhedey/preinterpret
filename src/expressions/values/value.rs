use super::*;

// TODO[concepts]: Uncomment when ready
// pub(crate) type QqqValue = Actual<'static, ValueType, BeOwned>;
// pub(crate) type QqqValueReferencable = Actual<'static, ValueType, BeReferenceable>;
// pub(crate) type QqqValueRef<'a> = Actual<'a, ValueType, BeAnyRef>;
// pub(crate) type QqqValueMut<'a> = Actual<'a, ValueType, BeAnyMut>;

define_parent_type! {
    pub(crate) ValueType,
    content: pub(crate) ValueContent,
    leaf_kind: pub(crate) ValueLeafKind,
    type_kind: ParentTypeKind::Value(pub(crate) ValueTypeKind),
    variants: {
        None => NoneType,
        Integer => IntegerType,
        Float => FloatType,
        Bool => BoolType,
        String => StringType,
        Char => CharType,
        // Unsupported literal is a type here so that we can parse such a token
        // as a value rather than a stream, and give it better error messages
        UnsupportedLiteral => UnsupportedLiteralType,
        Array => ArrayType,
        Object => ObjectType,
        Stream => StreamType,
        Range => RangeType,
        Iterator => IteratorType,
        Parser => ParserType,
    },
    type_name: "value",
    articled_display_name: "any value",
    temp_type_data: ValueTypeData,
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

define_interface! {
    struct ValueTypeData,
    parent: ValueTypeData,
    pub(crate) mod value_interface {
        pub(crate) mod methods {
            fn clone(this: CopyOnWriteValue) -> OwnedValue {
                this.into_owned_infallible()
            }

            fn as_mut(Spanned(this, span): Spanned<ArgumentValue>) -> ExecutionResult<MutableValue> {
                Ok(match this {
                    ArgumentValue::Owned(owned) => Mutable::new_from_owned(owned),
                    ArgumentValue::CopyOnWrite(copy_on_write) => ArgumentOwnership::Mutable
                        .map_from_copy_on_write(Spanned(copy_on_write, span))?
                        .expect_mutable(),
                    ArgumentValue::Mutable(mutable) => mutable,
                    ArgumentValue::Assignee(assignee) => assignee.0,
                    ArgumentValue::Shared(shared) => ArgumentOwnership::Mutable
                        .map_from_shared(Spanned(shared, span))?
                        .expect_mutable(),
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

            fn debug(Spanned(this, span_range): Spanned<CopyOnWriteValue>) -> ExecutionResult<()> {
                let message = this.concat_recursive(&ConcatBehaviour::debug(span_range))?;
                span_range.debug_err(message)
            }

            fn to_debug_string(Spanned(this, span_range): Spanned<CopyOnWriteValue>) -> ExecutionResult<String> {
                this.concat_recursive(&ConcatBehaviour::debug(span_range))
            }

            fn to_stream(Spanned(input, span_range): Spanned<CopyOnWriteValue>) -> ExecutionResult<OutputStream> {
                input.map_into(
                    |shared| shared.output_to_new_stream(Grouping::Flattened, span_range),
                    |owned| owned.0.into_stream(Grouping::Flattened, span_range),
                )
            }

            fn to_group(Spanned(input, span_range): Spanned<CopyOnWriteValue>) -> ExecutionResult<OutputStream> {
                input.map_into(
                    |shared| shared.output_to_new_stream(Grouping::Grouped, span_range),
                    |owned| owned.0.into_stream(Grouping::Grouped, span_range),
                )
            }

            fn to_string(Spanned(input, span_range): Spanned<SharedValue>) -> ExecutionResult<String> {
                input.concat_recursive(&ConcatBehaviour::standard(span_range))
            }

            [context] fn with_span(value: Spanned<CopyOnWriteValue>, spans: AnyRef<StreamValue>) -> ExecutionResult<OutputStream> {
                let mut this = to_stream(context, value)?;
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

            // EQUALITY METHODS
            // ===============================
            // Compare values with strict type checking - errors on value kind mismatch.
            [context] fn typed_eq(this: AnyRef<Value>, other: AnyRef<Value>) -> ExecutionResult<bool> {
                let this_value: &Value = &this;
                let other_value: &Value = &other;
                this_value.typed_eq(other_value, context.span_range())
            }

            // STRING-BASED CONVERSION METHODS
            // ===============================

            [context] fn to_ident(this: Spanned<OwnedValue>) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident(context, spanned)
            }

            [context] fn to_ident_camel(this: Spanned<OwnedValue>) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_camel(context, spanned)
            }

            [context] fn to_ident_snake(this: Spanned<OwnedValue>) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_snake(context, spanned)
            }

            [context] fn to_ident_upper_snake(this: Spanned<OwnedValue>) -> ExecutionResult<Ident> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_upper_snake(context, spanned)
            }

            // Some literals become Value::UnsupportedLiteral but can still be round-tripped back to a stream
            [context] fn to_literal(this: Spanned<OwnedValue>) -> ExecutionResult<Value> {
                let stream = this.into_stream()?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_literal(context, spanned)
            }
        }
        pub(crate) mod unary_operations {
            fn cast_to_string(Spanned(input, span_range): Spanned<OwnedValue>) -> ExecutionResult<String> {
                input.concat_recursive(&ConcatBehaviour::standard(span_range))
            }

            fn cast_to_stream(input: Spanned<OwnedValue>) -> ExecutionResult<OutputStream> {
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
                    UnaryOperation::Cast { target: CastTarget(kind), .. } => match kind {
                        ValueLeafKind::String(_) => unary_definitions::cast_to_string(),
                        ValueLeafKind::Stream(_) => unary_definitions::cast_to_stream(),
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
    fn into_owned(self) -> Owned<Self> {
        Owned::new(self)
    }
    fn into_owned_value(self) -> OwnedValue {
        Owned(self.into_value())
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
            None => Self::UnsupportedLiteral(UnsupportedLiteral { lit }).into_owned(),
        }
    }

    pub(crate) fn try_transparent_clone(
        &self,
        error_span_range: SpanRange,
    ) -> ExecutionResult<Value> {
        if !self.value_kind().supports_transparent_cloning() {
            return error_span_range.ownership_err(format!(
                "An owned value is required, but a reference was received, and {} does not support transparent cloning. You may wish to use .clone() explicitly.",
                self.articled_kind()
            ));
        }
        Ok(self.clone())
    }

    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }

    /// Recursively compares two values for equality using `ValuesEqual` semantics.
    pub(crate) fn values_equal(lhs: &Value, rhs: &Value) -> bool {
        lhs.lenient_eq(rhs)
    }
}

impl ValuesEqual for Value {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        // Each variant has two lines: same-type comparison, then type-mismatch fallback.
        // This ensures adding a new variant only requires adding two lines at the bottom.
        match (self, other) {
            (Value::None, Value::None) => ctx.values_equal(),
            (Value::None, _) => ctx.kind_mismatch(self, other),
            (Value::Boolean(l), Value::Boolean(r)) => l.test_equality(r, ctx),
            (Value::Boolean(_), _) => ctx.kind_mismatch(self, other),
            (Value::Char(l), Value::Char(r)) => l.test_equality(r, ctx),
            (Value::Char(_), _) => ctx.kind_mismatch(self, other),
            (Value::String(l), Value::String(r)) => l.test_equality(r, ctx),
            (Value::String(_), _) => ctx.kind_mismatch(self, other),
            (Value::Integer(l), Value::Integer(r)) => l.test_equality(r, ctx),
            (Value::Integer(_), _) => ctx.kind_mismatch(self, other),
            (Value::Float(l), Value::Float(r)) => l.test_equality(r, ctx),
            (Value::Float(_), _) => ctx.kind_mismatch(self, other),
            (Value::Array(l), Value::Array(r)) => l.test_equality(r, ctx),
            (Value::Array(_), _) => ctx.kind_mismatch(self, other),
            (Value::Object(l), Value::Object(r)) => l.test_equality(r, ctx),
            (Value::Object(_), _) => ctx.kind_mismatch(self, other),
            (Value::Stream(l), Value::Stream(r)) => l.test_equality(r, ctx),
            (Value::Stream(_), _) => ctx.kind_mismatch(self, other),
            (Value::Range(l), Value::Range(r)) => l.test_equality(r, ctx),
            (Value::Range(_), _) => ctx.kind_mismatch(self, other),
            (Value::UnsupportedLiteral(l), Value::UnsupportedLiteral(r)) => l.test_equality(r, ctx),
            (Value::UnsupportedLiteral(_), _) => ctx.kind_mismatch(self, other),
            (Value::Parser(l), Value::Parser(r)) => l.test_equality(r, ctx),
            (Value::Parser(_), _) => ctx.kind_mismatch(self, other),
            (Value::Iterator(l), Value::Iterator(r)) => l.test_equality(r, ctx),
            (Value::Iterator(_), _) => ctx.kind_mismatch(self, other),
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
            other => access.type_err(format!("Cannot index into {}", other.articled_kind())),
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
            other => access.type_err(format!("Cannot index into {}", other.articled_kind())),
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
            other => access.type_err(format!("Cannot index into {}", other.articled_kind())),
        }
    }

    pub(crate) fn into_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        match self {
            Value::Object(object) => object.into_property(access),
            other => access.type_err(format!(
                "Cannot access properties on {}",
                other.articled_kind()
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
                "Cannot access properties on {}",
                other.articled_kind()
            )),
        }
    }

    pub(crate) fn property_ref(&self, access: &PropertyAccess) -> ExecutionResult<&Self> {
        match self {
            Value::Object(object) => object.property_ref(access),
            other => access.type_err(format!(
                "Cannot access properties on {}",
                other.articled_kind()
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
                value.output_to(output);
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

impl Spanned<OwnedValue> {
    pub(crate) fn into_stream(self) -> ExecutionResult<OutputStream> {
        let Spanned(value, span_range) = self;
        value.0.into_stream(Grouping::Flattened, span_range)
    }

    pub(crate) fn resolve_any_iterator(
        self,
        resolution_target: &str,
    ) -> ExecutionResult<Owned<IteratorValue>> {
        IterableValue::resolve_owned(self, resolution_target)?.try_map(|v| v.into_iterator())
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

// TODO[concepts]: Remove when this is auto-generated after Value changes
impl HasLeafKind for Value {
    type LeafKind = ValueLeafKind;

    fn kind(&self) -> ValueLeafKind {
        match self {
            Value::None => ValueLeafKind::None(NoneKind),
            Value::Integer(integer) => ValueLeafKind::Integer(integer.kind()),
            Value::Float(float) => ValueLeafKind::Float(float.kind()),
            Value::Boolean(_) => ValueLeafKind::Bool(BoolKind),
            Value::String(_) => ValueLeafKind::String(StringKind),
            Value::Char(_) => ValueLeafKind::Char(CharKind),
            Value::Array(_) => ValueLeafKind::Array(ArrayKind),
            Value::Object(_) => ValueLeafKind::Object(ObjectKind),
            Value::Stream(_) => ValueLeafKind::Stream(StreamKind),
            Value::Range(_) => ValueLeafKind::Range(RangeKind),
            Value::Iterator(_) => ValueLeafKind::Iterator(IteratorKind),
            Value::Parser(_) => ValueLeafKind::Parser(ParserKind),
            Value::UnsupportedLiteral(_) => {
                ValueLeafKind::UnsupportedLiteral(UnsupportedLiteralKind)
            }
        }
    }
}
