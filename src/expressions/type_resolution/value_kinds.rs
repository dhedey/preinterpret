use super::*;

/// A trait for specific value kinds that can provide a display name.
/// This is implemented by `ValueKind`, `IntegerKind`, `FloatKind`, etc.
pub(crate) trait IsSpecificLeafKind: Copy + Into<ValueKind> {
    fn method_resolver(&self) -> &'static dyn MethodResolver;
    fn articled_display_name(&self) -> &'static str;
}

/// A trait for types that have a value kind.
pub(crate) trait HasValueKind {
    type SpecificKind: IsSpecificLeafKind;

    fn kind(&self) -> Self::SpecificKind;

    fn value_kind(&self) -> ValueKind {
        self.kind().into()
    }

    fn articled_kind(&self) -> &'static str {
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
    Range,
    Iterator,
    Parser,
}

impl ValueKind {
    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        Some(match name {
            "none" => ValueKind::None,
            "untyped_int" => ValueKind::Integer(IntegerKind::Untyped),
            "i8" => ValueKind::Integer(IntegerKind::I8),
            "i16" => ValueKind::Integer(IntegerKind::I16),
            "i32" => ValueKind::Integer(IntegerKind::I32),
            "i64" => ValueKind::Integer(IntegerKind::I64),
            "i128" => ValueKind::Integer(IntegerKind::I128),
            "isize" => ValueKind::Integer(IntegerKind::Isize),
            "u8" => ValueKind::Integer(IntegerKind::U8),
            "u16" => ValueKind::Integer(IntegerKind::U16),
            "u32" => ValueKind::Integer(IntegerKind::U32),
            "u64" => ValueKind::Integer(IntegerKind::U64),
            "u128" => ValueKind::Integer(IntegerKind::U128),
            "usize" => ValueKind::Integer(IntegerKind::Usize),
            "untyped_float" => ValueKind::Float(FloatKind::Untyped),
            "f32" => ValueKind::Float(FloatKind::F32),
            "f64" => ValueKind::Float(FloatKind::F64),
            "bool" => ValueKind::Boolean,
            "string" => ValueKind::String,
            "unsupported_literal" => ValueKind::UnsupportedLiteral,
            "char" => ValueKind::Char,
            "array" => ValueKind::Array,
            "object" => ValueKind::Object,
            "stream" => ValueKind::Stream,
            "range" => ValueKind::Range,
            "iterator" => ValueKind::Iterator,
            "parser" => ValueKind::Parser,
            _ => return None,
        })
    }

    pub(crate) fn source_name(&self) -> &'static str {
        match self {
            ValueKind::None => "none",
            ValueKind::Integer(IntegerKind::Untyped) => "untyped_int",
            ValueKind::Integer(IntegerKind::I8) => "i8",
            ValueKind::Integer(IntegerKind::I16) => "i16",
            ValueKind::Integer(IntegerKind::I32) => "i32",
            ValueKind::Integer(IntegerKind::I64) => "i64",
            ValueKind::Integer(IntegerKind::I128) => "i128",
            ValueKind::Integer(IntegerKind::Isize) => "isize",
            ValueKind::Integer(IntegerKind::U8) => "u8",
            ValueKind::Integer(IntegerKind::U16) => "u16",
            ValueKind::Integer(IntegerKind::U32) => "u32",
            ValueKind::Integer(IntegerKind::U64) => "u64",
            ValueKind::Integer(IntegerKind::U128) => "u128",
            ValueKind::Integer(IntegerKind::Usize) => "usize",
            ValueKind::Float(FloatKind::Untyped) => "untyped_float",
            ValueKind::Float(FloatKind::F32) => "f32",
            ValueKind::Float(FloatKind::F64) => "f64",
            ValueKind::Boolean => "bool",
            ValueKind::String => "string",
            ValueKind::Char => "char",
            ValueKind::UnsupportedLiteral => "unsupported_literal",
            ValueKind::Array => "array",
            ValueKind::Object => "object",
            ValueKind::Stream => "stream",
            ValueKind::Range => "range",
            ValueKind::Iterator => "iterator",
            ValueKind::Parser => "parser",
        }
    }
}

impl IsSpecificLeafKind for ValueKind {
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
            ValueKind::Range => "a range",
            ValueKind::Iterator => "an iterator",
            ValueKind::Parser => "a parser",
        }
    }

    fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            ValueKind::None => &NoneTypeData,
            ValueKind::Integer(kind) => kind.method_resolver(),
            ValueKind::Float(kind) => kind.method_resolver(),
            ValueKind::Boolean => &BooleanTypeData,
            ValueKind::String => &StringTypeData,
            ValueKind::Char => &CharTypeData,
            ValueKind::UnsupportedLiteral => &UnsupportedLiteralTypeData,
            ValueKind::Array => &ArrayTypeData,
            ValueKind::Object => &ObjectTypeData,
            ValueKind::Stream => &StreamTypeData,
            ValueKind::Range => &RangeTypeData,
            ValueKind::Iterator => &IteratorTypeData,
            ValueKind::Parser => &ParserTypeData,
        }
    }
}

impl ValueKind {
    /// This should be true for types which users expect to have value
    /// semantics, but false for types which are expensive to clone or
    /// are expected to have reference semantics.
    ///
    /// This indicates if an &x can be converted to an x via cloning
    /// when doing method resolution.
    pub(crate) fn supports_transparent_cloning(&self) -> bool {
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

    fn resolve_type_property(&self, property_name: &str) -> Option<Value> {
        self.method_resolver().resolve_type_property(property_name)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum IntegerKind {
    Untyped,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

impl IsSpecificLeafKind for IntegerKind {
    fn articled_display_name(&self) -> &'static str {
        match self {
            IntegerKind::Untyped => "an untyped integer",
            IntegerKind::I8 => "an i8",
            IntegerKind::I16 => "an i16",
            IntegerKind::I32 => "an i32",
            IntegerKind::I64 => "an i64",
            IntegerKind::I128 => "an i128",
            IntegerKind::Isize => "an isize",
            IntegerKind::U8 => "a u8",
            IntegerKind::U16 => "a u16",
            IntegerKind::U32 => "a u32",
            IntegerKind::U64 => "a u64",
            IntegerKind::U128 => "a u128",
            IntegerKind::Usize => "a usize",
        }
    }

    fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            IntegerKind::Untyped => &UntypedIntegerTypeData,
            IntegerKind::I8 => &I8TypeData,
            IntegerKind::I16 => &I16TypeData,
            IntegerKind::I32 => &I32TypeData,
            IntegerKind::I64 => &I64TypeData,
            IntegerKind::I128 => &I128TypeData,
            IntegerKind::Isize => &IsizeTypeData,
            IntegerKind::U8 => &U8TypeData,
            IntegerKind::U16 => &U16TypeData,
            IntegerKind::U32 => &U32TypeData,
            IntegerKind::U64 => &U64TypeData,
            IntegerKind::U128 => &U128TypeData,
            IntegerKind::Usize => &UsizeTypeData,
        }
    }
}

impl From<IntegerKind> for ValueKind {
    fn from(kind: IntegerKind) -> Self {
        ValueKind::Integer(kind)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FloatKind {
    Untyped,
    F32,
    F64,
}

impl IsSpecificLeafKind for FloatKind {
    fn articled_display_name(&self) -> &'static str {
        match self {
            FloatKind::Untyped => "an untyped float",
            FloatKind::F32 => "an f32",
            FloatKind::F64 => "an f64",
        }
    }

    fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            FloatKind::Untyped => &UntypedFloatTypeData,
            FloatKind::F32 => &F32TypeData,
            FloatKind::F64 => &F64TypeData,
        }
    }
}

impl From<FloatKind> for ValueKind {
    fn from(kind: FloatKind) -> Self {
        ValueKind::Float(kind)
    }
}

// A ValueKind represents a kind of leaf value.
// But a TypeKind represents a type in the hierarchy, which points at a type data.
pub(crate) enum TypeKind {
    Leaf(ValueKind),
    Parent(ParentTypeKind),
    Dyn(DynTypeKind),
}

impl TypeKind {
    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        if let Some(kind) = ValueKind::from_source_name(name) {
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
            TypeKind::Leaf(leaf_kind) => leaf_kind.source_name(),
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
