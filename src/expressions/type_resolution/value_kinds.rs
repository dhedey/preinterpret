use super::*;

/// A trait for specific value kinds that can provide a display name.
/// This is implemented by `ValueLeafKind`, `IntegerKind`, `FloatKind`, etc.
pub(crate) trait IsSpecificLeafKind: Copy + Into<ValueLeafKind> {
    fn source_type_name(&self) -> &'static str;
    fn articled_display_name(&self) -> &'static str;
    fn method_resolver(&self) -> &'static dyn MethodResolver;
}

impl ValueLeafKind {
    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        Some(match name {
            "none" => ValueLeafKind::None(NoneKind),
            "untyped_int" => ValueLeafKind::Integer(IntegerLeafKind::Untyped(UntypedIntegerKind)),
            "i8" => ValueLeafKind::Integer(IntegerLeafKind::I8(I8Kind)),
            "i16" => ValueLeafKind::Integer(IntegerLeafKind::I16(I16Kind)),
            "i32" => ValueLeafKind::Integer(IntegerLeafKind::I32(I32Kind)),
            "i64" => ValueLeafKind::Integer(IntegerLeafKind::I64(I64Kind)),
            "i128" => ValueLeafKind::Integer(IntegerLeafKind::I128(I128Kind)),
            "isize" => ValueLeafKind::Integer(IntegerLeafKind::Isize(IsizeKind)),
            "u8" => ValueLeafKind::Integer(IntegerLeafKind::U8(U8Kind)),
            "u16" => ValueLeafKind::Integer(IntegerLeafKind::U16(U16Kind)),
            "u32" => ValueLeafKind::Integer(IntegerLeafKind::U32(U32Kind)),
            "u64" => ValueLeafKind::Integer(IntegerLeafKind::U64(U64Kind)),
            "u128" => ValueLeafKind::Integer(IntegerLeafKind::U128(U128Kind)),
            "usize" => ValueLeafKind::Integer(IntegerLeafKind::Usize(UsizeKind)),
            "untyped_float" => ValueLeafKind::Float(FloatLeafKind::Untyped(UntypedFloatKind)),
            "f32" => ValueLeafKind::Float(FloatLeafKind::F32(F32Kind)),
            "f64" => ValueLeafKind::Float(FloatLeafKind::F64(F64Kind)),
            "bool" => ValueLeafKind::Bool(BoolKind),
            "string" => ValueLeafKind::String(StringKind),
            "unsupported_literal" => ValueLeafKind::UnsupportedLiteral(UnsupportedLiteralKind),
            "char" => ValueLeafKind::Char(CharKind),
            "array" => ValueLeafKind::Array(ArrayKind),
            "object" => ValueLeafKind::Object(ObjectKind),
            "stream" => ValueLeafKind::Stream(StreamKind),
            "range" => ValueLeafKind::Range(RangeKind),
            "iterator" => ValueLeafKind::Iterator(IteratorKind),
            "parser" => ValueLeafKind::Parser(ParserKind),
            _ => return None,
        })
    }
}

impl ValueLeafKind {
    /// This should be true for types which users expect to have value
    /// semantics, but false for types which are expensive to clone or
    /// are expected to have reference semantics.
    ///
    /// This indicates if an &x can be converted to an x via cloning
    /// when doing method resolution.
    pub(crate) fn supports_transparent_cloning(&self) -> bool {
        match self {
            ValueLeafKind::None(_) => true,
            ValueLeafKind::Integer(_) => true,
            ValueLeafKind::Float(_) => true,
            ValueLeafKind::Bool(_) => true,
            // Strings are value-like, so it makes sense to transparently clone them
            ValueLeafKind::String(_) => true,
            ValueLeafKind::Char(_) => true,
            ValueLeafKind::UnsupportedLiteral(_) => false,
            ValueLeafKind::Array(_) => false,
            ValueLeafKind::Object(_) => false,
            ValueLeafKind::Stream(_) => false,
            ValueLeafKind::Range(_) => true,
            ValueLeafKind::Iterator(_) => false,
            // A parser is a handle, so can be cloned transparently.
            // It may fail to be able to be used to parse if the underlying stream is out of scope of course.
            ValueLeafKind::Parser(_) => true,
        }
    }
}

// A ValueLeafKind represents a kind of leaf value.
// But a TypeKind represents a type in the hierarchy, which points at a type data.
pub(crate) enum TypeKind {
    Leaf(ValueLeafKind),
    Parent(ParentTypeKind),
    Dyn(DynTypeKind),
}

impl TypeKind {
    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        if let Some(kind) = ValueLeafKind::from_source_name(name) {
            return Some(TypeKind::Leaf(kind));
        }
        if let Some(kind) = ParentTypeKind::from_source_name(name) {
            return Some(TypeKind::Parent(kind));
        }
        if let Some(kind) = DynTypeKind::from_source_name(name) {
            return Some(TypeKind::Dyn(kind));
        }
        None
    }

    pub(crate) fn source_name(&self) -> &'static str {
        match self {
            TypeKind::Leaf(leaf_kind) => leaf_kind.source_type_name(),
            TypeKind::Parent(parent_kind) => parent_kind.source_name(),
            TypeKind::Dyn(dyn_kind) => dyn_kind.source_name(),
        }
    }
}

pub(crate) enum ParentTypeKind {
    Value(ValueTypeKind),
    Integer(IntegerTypeKind),
    Float(FloatTypeKind),
}

impl ParentTypeKind {
    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        Some(match name {
            ValueTypeKind::SOURCE_TYPE_NAME => ParentTypeKind::Value(ValueTypeKind),
            IntegerTypeKind::SOURCE_TYPE_NAME => ParentTypeKind::Integer(IntegerTypeKind),
            FloatTypeKind::SOURCE_TYPE_NAME => ParentTypeKind::Float(FloatTypeKind),
            _ => return None,
        })
    }

    pub(crate) fn source_name(&self) -> &'static str {
        match self {
            ParentTypeKind::Value(ValueTypeKind) => ValueTypeKind::SOURCE_TYPE_NAME,
            ParentTypeKind::Integer(IntegerTypeKind) => IntegerTypeKind::SOURCE_TYPE_NAME,
            ParentTypeKind::Float(FloatTypeKind) => FloatTypeKind::SOURCE_TYPE_NAME,
        }
    }

    pub(crate) fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            ParentTypeKind::Value(x) => x.method_resolver(),
            ParentTypeKind::Integer(x) => x.method_resolver(),
            ParentTypeKind::Float(x) => x.method_resolver(),
        }
    }
}

pub(crate) enum DynTypeKind {
    Iterable,
}

impl DynTypeKind {
    pub(in crate::expressions) fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            DynTypeKind::Iterable => &IterableTypeData,
        }
    }

    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        match name {
            "iterable" => Some(DynTypeKind::Iterable),
            _ => None,
        }
    }

    pub(crate) fn source_name(&self) -> &'static str {
        match self {
            DynTypeKind::Iterable => "iterable",
        }
    }
}

pub(crate) struct TypeIdent {
    span: Span,
    pub(crate) kind: TypeKind,
}

impl TypeIdent {
    pub(crate) fn from_ident(ident: &Ident) -> ParseResult<Self> {
        let span = ident.span();
        let name = ident.to_string();
        let kind = match TypeKind::from_source_name(name.as_str()) {
            Some(kind) => kind,
            None => {
                let lower_case_name = name.to_lowercase();
                let error_message = match TypeKind::from_source_name(&lower_case_name) {
                    Some(_) => format!("Expected '{}'", lower_case_name),
                    None => match lower_case_name.as_str() {
                        "integer" => "Expected 'int'".to_string(),
                        "str" => "Expected 'string'".to_string(),
                        "character" => "Expected 'char'".to_string(),
                        "list" => "Expected 'array'".to_string(),
                        "obj" => "Expected 'object'".to_string(),
                        _ => format!("Unknown type '{}'", name),
                    },
                };
                return span.parse_err(error_message);
            }
        };
        Ok(Self { span, kind })
    }
}

impl ParseSource for TypeIdent {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Self::from_ident(&input.parse()?)
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl TypeKind {
    pub(in crate::expressions) fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            TypeKind::Leaf(leaf_kind) => leaf_kind.method_resolver(),
            TypeKind::Parent(parent_kind) => parent_kind.method_resolver(),
            TypeKind::Dyn(dyn_kind) => dyn_kind.method_resolver(),
        }
    }
}

pub(crate) struct TypeProperty {
    pub(crate) source_type: TypeIdent,
    _colons: Unused<Token![::]>,
    pub(crate) property: Ident,
}

impl ParseSource for TypeProperty {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(Self {
            source_type: input.parse()?,
            _colons: input.parse()?,
            property: input.parse()?,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl TypeProperty {
    pub(crate) fn resolve_spanned(
        &self,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        let resolver = self.source_type.kind.method_resolver();
        // TODO[performance] - lazily initialize properties as Shared
        let resolved_property = resolver.resolve_type_property(&self.property.to_string());
        match resolved_property {
            Some(value) => ownership.map_from_shared(Spanned(
                SharedValue::new_from_owned(value.into_owned_value()),
                self.span_range(),
            )),
            None => self.type_err(format!(
                "Type '{}' has no property named '{}'",
                self.source_type.kind.source_name(),
                self.property,
            )),
        }
    }
}

impl HasSpanRange for TypeProperty {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.source_type.span, self.property.span())
    }
}
