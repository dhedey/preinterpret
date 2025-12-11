use super::*;

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

static NONE: NoneTypeData = NoneTypeData;
static BOOLEAN: BooleanTypeData = BooleanTypeData;
static STRING: StringTypeData = StringTypeData;
static CHAR: CharTypeData = CharTypeData;
static UNSUPPORTED_LITERAL: UnsupportedLiteralTypeData = UnsupportedLiteralTypeData;
static ARRAY: ArrayTypeData = ArrayTypeData;
static OBJECT: ObjectTypeData = ObjectTypeData;
static STREAM: StreamTypeData = StreamTypeData;
static RANGE: RangeTypeData = RangeTypeData;
static ITERABLE: IterableTypeData = IterableTypeData;
static ITERATOR: IteratorTypeData = IteratorTypeData;
static PARSER: ParserTypeData = ParserTypeData;

impl ValueKind {
    fn method_resolver(&self) -> &'static dyn MethodResolver {
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

impl IsSpecificValueKind for IntegerKind {
    fn display_name(&self) -> &'static str {
        match self {
            IntegerKind::Untyped => "untyped integer",
            IntegerKind::I8 => "i8",
            IntegerKind::I16 => "i16",
            IntegerKind::I32 => "i32",
            IntegerKind::I64 => "i64",
            IntegerKind::I128 => "i128",
            IntegerKind::Isize => "isize",
            IntegerKind::U8 => "u8",
            IntegerKind::U16 => "u16",
            IntegerKind::U32 => "u32",
            IntegerKind::U64 => "u64",
            IntegerKind::U128 => "u128",
            IntegerKind::Usize => "usize",
        }
    }

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
}

impl From<IntegerKind> for ValueKind {
    fn from(kind: IntegerKind) -> Self {
        ValueKind::Integer(kind)
    }
}

static INTEGER: IntegerTypeData = IntegerTypeData;
static UNTYPED_INTEGER: UntypedIntegerTypeData = UntypedIntegerTypeData;
static I8: I8TypeData = I8TypeData;
static I16: I16TypeData = I16TypeData;
static I32: I32TypeData = I32TypeData;
static I64: I64TypeData = I64TypeData;
static I128: I128TypeData = I128TypeData;
static ISIZE: IsizeTypeData = IsizeTypeData;
static U8: U8TypeData = U8TypeData;
static U16: U16TypeData = U16TypeData;
static U32: U32TypeData = U32TypeData;
static U64: U64TypeData = U64TypeData;
static U128: U128TypeData = U128TypeData;
static USIZE: UsizeTypeData = UsizeTypeData;

impl IntegerKind {
    pub(in crate::expressions) fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            IntegerKind::Untyped => &UNTYPED_INTEGER,
            IntegerKind::I8 => &I8,
            IntegerKind::I16 => &I16,
            IntegerKind::I32 => &I32,
            IntegerKind::I64 => &I64,
            IntegerKind::I128 => &I128,
            IntegerKind::Isize => &ISIZE,
            IntegerKind::U8 => &U8,
            IntegerKind::U16 => &U16,
            IntegerKind::U32 => &U32,
            IntegerKind::U64 => &U64,
            IntegerKind::U128 => &U128,
            IntegerKind::Usize => &USIZE,
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

static FLOAT: FloatTypeData = FloatTypeData;
static UNTYPED_FLOAT: UntypedFloatTypeData = UntypedFloatTypeData;
static F32: F32TypeData = F32TypeData;
static F64: F64TypeData = F64TypeData;

impl FloatKind {
    pub(in crate::expressions) fn method_resolver(&self) -> &'static dyn MethodResolver {
        match self {
            FloatKind::Untyped => &UNTYPED_FLOAT,
            FloatKind::F32 => &F32,
            FloatKind::F64 => &F64,
        }
    }
}

pub(crate) struct Type {
    span: Span,
    pub(crate) kind: TypeKind,
}

// A ValueKind represents a kind of leaf value.
// But a TypeKind represents a type in the hierarchy, which points at a type data.
pub(crate) enum TypeKind {
    Integer,
    SpecificInteger(IntegerKind),
    Float,
    SpecificFloat(FloatKind),
    Boolean,
    String,
    Char,
    Array,
    Object,
    Stream,
    Range,
    Iterable,
    Iterator,
    Parser,
}

impl Type {
    pub(crate) fn from_source_name(name: &str) -> Option<TypeKind> {
        Some(match name {
            "int" => TypeKind::Integer,
            "i8" => TypeKind::SpecificInteger(IntegerKind::I8),
            "i16" => TypeKind::SpecificInteger(IntegerKind::I16),
            "i32" => TypeKind::SpecificInteger(IntegerKind::I32),
            "i64" => TypeKind::SpecificInteger(IntegerKind::I64),
            "i128" => TypeKind::SpecificInteger(IntegerKind::I128),
            "isize" => TypeKind::SpecificInteger(IntegerKind::Isize),
            "u8" => TypeKind::SpecificInteger(IntegerKind::U8),
            "u16" => TypeKind::SpecificInteger(IntegerKind::U16),
            "u32" => TypeKind::SpecificInteger(IntegerKind::U32),
            "u64" => TypeKind::SpecificInteger(IntegerKind::U64),
            "u128" => TypeKind::SpecificInteger(IntegerKind::U128),
            "usize" => TypeKind::SpecificInteger(IntegerKind::Usize),
            "float" => TypeKind::Float,
            "f32" => TypeKind::SpecificFloat(FloatKind::F32),
            "f64" => TypeKind::SpecificFloat(FloatKind::F64),
            "bool" => TypeKind::Boolean,
            "string" => TypeKind::String,
            "char" => TypeKind::Char,
            "array" => TypeKind::Array,
            "object" => TypeKind::Object,
            "stream" => TypeKind::Stream,
            "range" => TypeKind::Range,
            "iterable" => TypeKind::Iterable,
            "iterator" => TypeKind::Iterator,
            "parser" => TypeKind::Parser,
            _ => return None,
        })
    }

    pub(crate) fn source_name(&self) -> &'static str {
        // This should be inverse of parse below
        match &self.kind {
            TypeKind::Integer => "int",
            TypeKind::SpecificInteger(kind) => kind.display_name(),
            TypeKind::Float => "float",
            TypeKind::SpecificFloat(kind) => kind.display_name(),
            TypeKind::Boolean => "bool",
            TypeKind::String => "string",
            TypeKind::Char => "char",
            TypeKind::Array => "array",
            TypeKind::Object => "object",
            TypeKind::Stream => "stream",
            TypeKind::Range => "range",
            TypeKind::Iterable => "iterable",
            TypeKind::Iterator => "iterator",
            TypeKind::Parser => "parser",
        }
    }

    pub(crate) fn from_ident(ident: &Ident) -> ParseResult<Self> {
        let span = ident.span();
        let name = ident.to_string();
        let kind = match Self::from_source_name(name.as_str()) {
            Some(kind) => kind,
            None => {
                let lower_case_name = name.to_lowercase();
                let error_message = match Self::from_source_name(&lower_case_name) {
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

impl ParseSource for Type {
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
            TypeKind::Integer => &INTEGER,
            TypeKind::SpecificInteger(integer_kind) => integer_kind.method_resolver(),
            TypeKind::Float => &FLOAT,
            TypeKind::SpecificFloat(float_kind) => float_kind.method_resolver(),
            TypeKind::Boolean => &BOOLEAN,
            TypeKind::String => &STRING,
            TypeKind::Char => &CHAR,
            TypeKind::Array => &ARRAY,
            TypeKind::Object => &OBJECT,
            TypeKind::Stream => &STREAM,
            TypeKind::Range => &RANGE,
            TypeKind::Iterable => &ITERABLE,
            TypeKind::Iterator => &ITERATOR,
            TypeKind::Parser => &PARSER,
        }
    }
}

pub(crate) struct TypeProperty {
    pub(crate) source_type: Type,
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
    pub(crate) fn resolve(&self, ownership: RequestedOwnership) -> ExecutionResult<RequestedValue> {
        let resolver = self.source_type.kind.method_resolver();
        // TODO[performance] - lazily initialize properties as Shared
        let resolved_property = resolver.resolve_type_property(&self.property.to_string());
        match resolved_property {
            Some(value) => ownership
                .map_from_shared(Spanned(
                    SharedValue::new_from_owned(value),
                    self.span_range(),
                ))
                .map(|spanned| spanned.0),
            None => self.type_err(format!(
                "Type '{}' has no property named '{}'",
                self.source_type.source_name(),
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
