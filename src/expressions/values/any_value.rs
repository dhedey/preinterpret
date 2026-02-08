use super::*;

pub(crate) type AnyValue = AnyValueContent<'static, BeOwned>;
/// For symmetry
pub(crate) type AnyValueOwned = AnyValue;
pub(crate) type AnyValueRef<'a> = AnyValueContent<'a, BeRef>;
pub(crate) type AnyValueAnyRef<'a> = AnyValueContent<'a, BeAnyRef>;
pub(crate) type AnyValueShared = Shared<AnyValue>;
pub(crate) type AnyValueMutable = Mutable<AnyValue>;
pub(crate) type AnyValueAssignee = Assignee<AnyValue>;
// pub(crate) type AnyValueShared = AnyValueContent<'static, BeShared>;
// pub(crate) type AnyValueMutable = AnyValueContent<'static, BeMutable>;
// pub(crate) type AnyValueAssignee = AnyValueContent<'static, BeAssignee>;

define_parent_type! {
    pub(crate) AnyType,
    content: pub(crate) AnyValueContent,
    leaf_kind: pub(crate) AnyValueLeafKind,
    type_kind: ParentTypeKind::Value(pub(crate) AnyValueTypeKind),
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
        Function => FunctionType,
        PreinterpretApi => PreinterpretApiType,
    },
    type_name: "any",
    articled_value_name: "any",
}

define_type_features! {
    impl AnyType,
    pub(crate) mod value_interface {
        methods {
            fn clone(this: CopyOnWriteValue) -> AnyValue {
                this.clone_to_owned_infallible()
            }

            fn as_mut(Spanned(this, span): Spanned<ArgumentValue>) -> FunctionResult<AnyValueMutable> {
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

            fn replace(mut a: AssigneeValue, b: AnyValue) -> AnyValue {
                core::mem::replace(a.0.deref_mut(), b)
            }

            [context] fn debug(Spanned(this, span_range): Spanned<CopyOnWriteValue>) -> FunctionResult<()> {
                let message = this.as_ref_value().concat_recursive(&ConcatBehaviour::debug(span_range), context.interpreter)?;
                span_range.debug_err(message)
            }

            [context] fn to_debug_string(Spanned(this, span_range): Spanned<CopyOnWriteValue>) -> FunctionResult<String> {
                this.as_ref_value().concat_recursive(&ConcatBehaviour::debug(span_range), context.interpreter)
            }

            [context] fn to_stream(Spanned(input, span_range): Spanned<CopyOnWriteValue>) -> FunctionResult<OutputStream> {
                let interpreter_ptr = context.interpreter as *mut Interpreter;
                input.map_into(
                    // SAFETY: map_into only calls one of these two closures,
                    // so only one mutable reference is active at a time.
                    |shared| shared.as_ref_value().output_to_new_stream(Grouping::Flattened, span_range, unsafe { &mut *interpreter_ptr }),
                    |owned| owned.into_stream(Grouping::Flattened, span_range, unsafe { &mut *interpreter_ptr }),
                )
            }

            [context] fn to_group(Spanned(input, span_range): Spanned<CopyOnWriteValue>) -> FunctionResult<OutputStream> {
                let interpreter_ptr = context.interpreter as *mut Interpreter;
                input.map_into(
                    // SAFETY: map_into only calls one of these two closures,
                    // so only one mutable reference is active at a time.
                    |shared| shared.as_ref_value().output_to_new_stream(Grouping::Grouped, span_range, unsafe { &mut *interpreter_ptr }),
                    |owned| owned.into_stream(Grouping::Grouped, span_range, unsafe { &mut *interpreter_ptr }),
                )
            }

            [context] fn to_string(Spanned(input, span_range): Spanned<SharedValue>) -> FunctionResult<String> {
                input.as_ref_value().concat_recursive(&ConcatBehaviour::standard(span_range), context.interpreter)
            }

            [context] fn with_span(value: Spanned<CopyOnWriteValue>, spans: AnyRef<OutputStream>) -> FunctionResult<OutputStream> {
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
            [context] fn typed_eq(this: AnyValueAnyRef, other: AnyValueAnyRef) -> FunctionResult<bool> {
                this.as_ref_value().typed_eq(&other.as_ref_value(), context.span_range())
            }

            // STRING-BASED CONVERSION METHODS
            // ===============================

            [context] fn to_ident(this: Spanned<AnyValue>) -> FunctionResult<Ident> {
                let stream = this.into_stream(context.interpreter)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident(context, spanned)
            }

            [context] fn to_ident_camel(this: Spanned<AnyValue>) -> FunctionResult<Ident> {
                let stream = this.into_stream(context.interpreter)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_camel(context, spanned)
            }

            [context] fn to_ident_snake(this: Spanned<AnyValue>) -> FunctionResult<Ident> {
                let stream = this.into_stream(context.interpreter)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_snake(context, spanned)
            }

            [context] fn to_ident_upper_snake(this: Spanned<AnyValue>) -> FunctionResult<Ident> {
                let stream = this.into_stream(context.interpreter)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_ident_upper_snake(context, spanned)
            }

            // Some literals become Value::UnsupportedLiteral but can still be round-tripped back to a stream
            [context] fn to_literal(this: Spanned<AnyValue>) -> FunctionResult<AnyValue> {
                let stream = this.into_stream(context.interpreter)?;
                let spanned = stream.into_spanned_ref(context.output_span_range);
                stream_interface::methods::to_literal(context, spanned)
            }
        }
        unary_operations {
            [context] fn cast_to_string(Spanned(input, span_range): Spanned<AnyValue>) -> FunctionResult<String> {
                input.as_ref_value().concat_recursive(&ConcatBehaviour::standard(span_range), context.interpreter)
            }

            [context] fn cast_to_stream(Spanned(input, span_range): Spanned<AnyValue>) -> FunctionResult<OutputStream> {
                input.into_stream(Grouping::Flattened, span_range, context.interpreter)
            }
        }
        binary_operations {
            fn eq(lhs: AnyValueAnyRef, rhs: AnyValueAnyRef) -> bool {
                AnyValue::values_equal(lhs.as_ref_value(), rhs.as_ref_value())
            }

            fn ne(lhs: AnyValueAnyRef, rhs: AnyValueAnyRef) -> bool {
                !AnyValue::values_equal(lhs.as_ref_value(), rhs.as_ref_value())
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Cast { target: CastTarget(kind), .. } => match kind {
                        AnyValueLeafKind::String(_) => unary_definitions::cast_to_string(),
                        AnyValueLeafKind::Stream(_) => unary_definitions::cast_to_stream(),
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

pub(crate) trait IntoAnyValue: Sized {
    fn into_any_value(self) -> AnyValue;
}

impl<'a, F: IsHierarchicalForm> AnyValueContent<'a, F> {
    pub(crate) fn is_none(&self) -> bool {
        matches!(&self, AnyValueContent::None(_))
    }
}

impl AnyValue {
    pub(crate) fn for_literal(literal: Literal) -> AnyValue {
        // The unwrap should be safe because all Literal should be parsable
        // as syn::Lit; falling back to syn::Lit::Verbatim if necessary.
        Self::for_syn_lit(
            literal
                .to_token_stream()
                .interpreted_parse_with(|input| input.parse())
                .unwrap(),
        )
    }

    pub(crate) fn for_syn_lit(lit: syn::Lit) -> AnyValue {
        // https://docs.rs/syn/latest/syn/enum.Lit.html
        let matched = match &lit {
            Lit::Int(lit) => match IntegerValue::for_litint(lit) {
                Ok(int) => Some(int.into_any_value()),
                Err(_) => None,
            },
            Lit::Float(lit) => match FloatValue::for_litfloat(lit) {
                Ok(float) => Some(float.into_any_value()),
                Err(_) => None,
            },
            Lit::Bool(lit) => Some(lit.value.into_any_value()),
            Lit::Str(lit) => Some(lit.value().into_any_value()),
            Lit::Char(lit) => Some(lit.value().into_any_value()),
            _ => None,
        };
        match matched {
            Some(value) => value,
            None => UnsupportedLiteral(lit).into_any_value(),
        }
    }

    // TODO[concepts]: Remove from here
    pub(crate) fn try_transparent_clone(
        &self,
        error_span_range: SpanRange,
    ) -> FunctionResult<AnyValue> {
        if !self.value_kind().supports_transparent_cloning() {
            return error_span_range.ownership_err(format!(
                "An owned value is required, but a reference was received, and {} does not support transparent cloning. You may wish to use .clone() explicitly.",
                self.kind().articled_value_name(),
            ));
        }
        Ok(self.clone())
    }

    pub(crate) fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        self.as_ref_value()
            .test_equality(&other.as_ref_value(), ctx)
    }

    /// Recursively compares two values for equality using `ValuesEqual` semantics.
    pub(crate) fn values_equal<'a>(lhs: AnyValueRef<'a>, rhs: AnyValueRef<'a>) -> bool {
        lhs.lenient_eq(&rhs)
    }
}

impl<'a> ValuesEqual for AnyValueRef<'a> {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        // Each variant has two lines: same-type comparison, then type-mismatch fallback.
        // This ensures adding a new variant only requires adding two lines at the bottom.
        match (self, other) {
            (AnyValueContent::None(_), AnyValueContent::None(_)) => ctx.values_equal(),
            (AnyValueContent::None(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Bool(l), AnyValueContent::Bool(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Bool(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Char(l), AnyValueContent::Char(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Char(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::String(l), AnyValueContent::String(r)) => l.test_equality(r, ctx),
            (AnyValueContent::String(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Integer(l), AnyValueContent::Integer(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Integer(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Float(l), AnyValueContent::Float(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Float(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Array(l), AnyValueContent::Array(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Array(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Object(l), AnyValueContent::Object(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Object(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Stream(l), AnyValueContent::Stream(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Stream(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Range(l), AnyValueContent::Range(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Range(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::UnsupportedLiteral(l), AnyValueContent::UnsupportedLiteral(r)) => {
                l.test_equality(r, ctx)
            }
            (AnyValueContent::UnsupportedLiteral(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Parser(l), AnyValueContent::Parser(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Parser(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Iterator(l), AnyValueContent::Iterator(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Iterator(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::Function(l), AnyValueContent::Function(r)) => l.test_equality(r, ctx),
            (AnyValueContent::Function(_), _) => ctx.kind_mismatch(self, other),
            (AnyValueContent::PreinterpretApi(not_a_value), _) => match **not_a_value {},
        }
    }
}

impl AnyValue {
    pub(crate) fn into_stream(
        self,
        grouping: Grouping,
        error_span_range: SpanRange,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<OutputStream> {
        match (self, grouping) {
            (AnyValueContent::Stream(value), Grouping::Flattened) => Ok(value),
            (AnyValueContent::Stream(value), Grouping::Grouped) => {
                let mut output: OutputStream = OutputStream::new();
                let span = Span::call_site();
                output.push_new_group(value, Delimiter::None, span);
                Ok(output)
            }
            (other, grouping) => {
                other
                    .as_ref_value()
                    .output_to_new_stream(grouping, error_span_range, interpreter)
            }
        }
    }
}

impl<'a> AnyValueRef<'a> {
    pub(crate) fn output_to_new_stream(
        self,
        grouping: Grouping,
        error_span_range: SpanRange,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<OutputStream> {
        interpreter.capture_output(|output| {
            self.output_to(
                grouping,
                &mut ToStreamContext::new(output, error_span_range),
            )
        })
    }

    pub(crate) fn output_to(
        self,
        grouping: Grouping,
        output: &mut ToStreamContext,
    ) -> FunctionResult<()> {
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

    fn output_flattened_to(self, output: &mut ToStreamContext) -> FunctionResult<()> {
        match self {
            AnyValueContent::None(_) => {}
            AnyValueContent::Integer(value) => {
                let literal = value
                    .clone_to_owned_infallible()
                    .to_literal(output.new_token_span());
                output.push_literal(literal);
            }
            AnyValueContent::Float(value) => {
                value.clone_to_owned_infallible().output_to(output);
            }
            AnyValueContent::Bool(value) => {
                let ident = Ident::new_bool(*value, output.new_token_span());
                output.push_ident(ident);
            }
            AnyValueContent::String(value) => {
                let literal = Literal::string(value).with_span(output.new_token_span());
                output.push_literal(literal);
            }
            AnyValueContent::Char(value) => {
                let literal = Literal::character(*value).with_span(output.new_token_span());
                output.push_literal(literal);
            }
            AnyValueContent::UnsupportedLiteral(literal) => {
                output.extend_raw_tokens(literal.0.to_token_stream())
            }
            AnyValueContent::Object(_) => {
                return output.type_err("Objects cannot be output to a stream");
            }
            AnyValueContent::Array(array) => array.output_items_to(output, Grouping::Flattened)?,
            AnyValueContent::Stream(value) => value.append_cloned_into(output),
            AnyValueContent::Iterator(iterator) => iterator
                .clone()
                .output_items_to(output, Grouping::Flattened)?,
            AnyValueContent::Range(value) => {
                let iterator = IteratorValue::new_for_range(value.clone())?;
                iterator.output_items_to(output, Grouping::Flattened)?
            }
            AnyValueContent::Parser(_) => {
                return output.type_err("Parsers cannot be output to a stream");
            }
            AnyValueContent::Function(_) => {
                return output.type_err("Functions cannot be output to a stream");
            }
            AnyValueContent::PreinterpretApi(not_a_value) => match *not_a_value {},
        };
        Ok(())
    }

    pub(crate) fn concat_recursive(
        self,
        behaviour: &ConcatBehaviour,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<String> {
        let mut output = String::new();
        self.concat_recursive_into(&mut output, behaviour, interpreter)?;
        Ok(output)
    }

    pub(crate) fn concat_recursive_into(
        self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
        interpreter: &mut Interpreter,
    ) -> FunctionResult<()> {
        match self {
            AnyValueContent::None(_) => {
                if behaviour.show_none_values {
                    output.push_str("None");
                }
            }
            AnyValueContent::Stream(stream) => {
                stream.concat_as_literal_into(output, behaviour);
            }
            AnyValueContent::Array(array) => {
                array.concat_recursive_into(output, behaviour, interpreter)?;
            }
            AnyValueContent::Object(object) => {
                object.concat_recursive_into(output, behaviour, interpreter)?;
            }
            AnyValueContent::Iterator(iterator) => {
                iterator.concat_recursive_into(output, behaviour, interpreter)?;
            }
            AnyValueContent::Range(range) => {
                range.concat_recursive_into(output, behaviour, interpreter)?;
            }
            AnyValueContent::Parser(parser) => {
                if behaviour.use_debug_literal_syntax {
                    write!(output, "parser[{:?}]", parser).unwrap();
                } else {
                    return behaviour
                        .error_span_range
                        .type_err("Parsers cannot be output to a string");
                }
            }
            AnyValueContent::Function(_) => {
                if behaviour.use_debug_literal_syntax {
                    output.push_str("function[?]")
                } else {
                    return behaviour
                        .error_span_range
                        .type_err("Functions cannot be output to a string");
                }
            }
            AnyValueContent::Integer(_)
            | AnyValueContent::Float(_)
            | AnyValueContent::Char(_)
            | AnyValueContent::Bool(_)
            | AnyValueContent::UnsupportedLiteral(_)
            | AnyValueContent::String(_) => {
                // This isn't the most efficient, but it's less code and debug doesn't need to be super efficient.
                let stream = self
                    .output_to_new_stream(
                        Grouping::Flattened,
                        behaviour.error_span_range,
                        interpreter,
                    )
                    .expect("Non-composite values should all be able to be outputted to a stream");
                stream.concat_content_into(output, behaviour);
            }
            AnyValueContent::PreinterpretApi(not_a_value) => match *not_a_value {},
        }
        Ok(())
    }
}

pub(crate) struct ToStreamContext<'a> {
    inner: OutputInterpreter<'a>,
    error_span_range: SpanRange,
}

impl<'a> ToStreamContext<'a> {
    pub(crate) fn new(output: &'a mut OutputInterpreter, error_span_range: SpanRange) -> Self {
        Self {
            inner: output.reborrow(),
            error_span_range,
        }
    }

    pub(crate) fn push_grouped<E>(
        &mut self,
        f: impl FnOnce(&mut ToStreamContext) -> Result<(), E>,
        delimiter: Delimiter,
    ) -> Result<(), E> {
        let span = self.new_token_span();
        let error_span_range = self.error_span_range;
        self.inner.in_output_group(delimiter, span, |inner| {
            let mut ctx = ToStreamContext {
                inner: inner.reborrow(),
                error_span_range,
            };
            f(&mut ctx)
        })
    }

    pub(crate) fn new_token_span(&self) -> Span {
        Span::call_site()
    }
}

impl<'a> Deref for ToStreamContext<'a> {
    type Target = OutputInterpreter<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ToStreamContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl HasSpanRange for ToStreamContext<'_> {
    fn span_range(&self) -> SpanRange {
        self.error_span_range
    }
}

impl Spanned<AnyValue> {
    pub(crate) fn into_statement_result(self) -> ExecutionResult<()> {
        let Spanned(value, span_range) = self;
        match value {
            AnyValueContent::None(_) => Ok(()),
            _ => span_range.control_flow_err("A non-returning statement must not return a value. If you wish to explicitly discard the expression's result, use `let _ = ...;`. Alternatively, If you wish to output the value into the parent token stream, use `emit ...;`"),
        }
    }

    pub(crate) fn into_stream(self, interpreter: &mut Interpreter) -> FunctionResult<OutputStream> {
        let Spanned(value, span_range) = self;
        value.into_stream(Grouping::Flattened, span_range, interpreter)
    }

    pub(crate) fn resolve_any_iterator(
        self,
        resolution_target: &str,
    ) -> FunctionResult<IteratorValue> {
        self.dyn_resolve::<dyn IsIterable>(resolution_target)?
            .into_iterator()
    }
}

#[derive(Copy, Clone)]
pub(crate) enum Grouping {
    Grouped,
    Flattened,
}
