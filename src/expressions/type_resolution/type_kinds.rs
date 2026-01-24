use super::*;

/// A trait for specific value kinds that can provide a display name.
/// This is implemented by `ValueLeafKind`, `IntegerKind`, `FloatKind`, etc.
pub(crate) trait IsLeafKind: Copy + Into<AnyValueLeafKind> {
    fn source_type_name(&self) -> &'static str;
    fn articled_display_name(&self) -> &'static str;
    fn feature_resolver(&self) -> &'static dyn TypeFeatureResolver;
}

impl AnyValueLeafKind {
    /// This should be true for types which users expect to have value
    /// semantics, but false for types which are expensive to clone or
    /// are expected to have reference semantics.
    ///
    /// This indicates if an &x can be converted to an x via cloning
    /// when doing method resolution.
    pub(crate) fn supports_transparent_cloning(&self) -> bool {
        match self {
            AnyValueLeafKind::None(_) => true,
            AnyValueLeafKind::Integer(_) => true,
            AnyValueLeafKind::Float(_) => true,
            AnyValueLeafKind::Bool(_) => true,
            // Strings are value-like, so it makes sense to transparently clone them
            AnyValueLeafKind::String(_) => true,
            AnyValueLeafKind::Char(_) => true,
            AnyValueLeafKind::UnsupportedLiteral(_) => false,
            AnyValueLeafKind::Array(_) => false,
            AnyValueLeafKind::Object(_) => false,
            AnyValueLeafKind::Stream(_) => false,
            AnyValueLeafKind::Range(_) => true,
            AnyValueLeafKind::Iterator(_) => false,
            // A parser is a handle, so can be cloned transparently.
            // It may fail to be able to be used to parse if the underlying stream is out of scope of course.
            AnyValueLeafKind::Parser(_) => true,
            AnyValueLeafKind::Function(_) => true,
            AnyValueLeafKind::PreinterpretApi(_) => true,
        }
    }
}

// A AnyValueLeafKind represents a kind of leaf value.
// But a TypeKind represents a type in the hierarchy, which points at a type data.
pub(crate) enum TypeKind {
    Leaf(AnyValueLeafKind),
    Parent(ParentTypeKind),
    Dyn(DynTypeKind),
}

impl TypeKind {
    /// This should be true for types which users expect to have value
    /// semantics, but false for types which are expensive to clone or
    /// are expected to have reference semantics.
    ///
    /// This indicates if an &x can be converted to an x via cloning
    /// when doing method resolution.
    pub(crate) fn supports_transparent_cloning(&self) -> bool {
        match self {
            TypeKind::Leaf(leaf_kind) => leaf_kind.supports_transparent_cloning(),
            TypeKind::Parent(_) => false,
            TypeKind::Dyn(_) => false,
        }
    }

    pub(crate) fn articled_display_name(&self) -> &'static str {
        match self {
            TypeKind::Leaf(leaf_kind) => leaf_kind.articled_display_name(),
            TypeKind::Parent(parent_kind) => parent_kind.articled_display_name(),
            TypeKind::Dyn(dyn_kind) => dyn_kind.articled_display_name(),
        }
    }

    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        // Parse Leaf and Parent TypeKinds
        if let Some(tk) = <AnyType as IsType>::type_kind_from_source_name(name) {
            return Some(tk);
        }
        // Parse Dyn TypeKinds
        if let Some(dyn_type_kind) = DynTypeKind::from_source_name(name) {
            return Some(TypeKind::Dyn(dyn_type_kind));
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
    Value(AnyValueTypeKind),
    Integer(IntegerTypeKind),
    Float(FloatTypeKind),
}

impl ParentTypeKind {
    pub(crate) fn articled_display_name(&self) -> &'static str {
        match self {
            ParentTypeKind::Value(x) => x.articled_display_name(),
            ParentTypeKind::Integer(x) => x.articled_display_name(),
            ParentTypeKind::Float(x) => x.articled_display_name(),
        }
    }

    pub(crate) fn source_name(&self) -> &'static str {
        match self {
            ParentTypeKind::Value(x) => x.source_type_name(),
            ParentTypeKind::Integer(x) => x.source_type_name(),
            ParentTypeKind::Float(x) => x.source_type_name(),
        }
    }

    pub(crate) fn feature_resolver(&self) -> &'static dyn TypeFeatureResolver {
        match self {
            ParentTypeKind::Value(x) => x.feature_resolver(),
            ParentTypeKind::Integer(x) => x.feature_resolver(),
            ParentTypeKind::Float(x) => x.feature_resolver(),
        }
    }
}

pub(crate) enum DynTypeKind {
    Iterable,
}

impl DynTypeKind {
    pub(crate) fn articled_display_name(&self) -> &'static str {
        match self {
            DynTypeKind::Iterable => IterableType::ARTICLED_DISPLAY_NAME,
        }
    }

    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        match name {
            IterableType::SOURCE_TYPE_NAME => Some(DynTypeKind::Iterable),
            _ => None,
        }
    }

    pub(crate) fn source_name(&self) -> &'static str {
        match self {
            DynTypeKind::Iterable => IterableType::SOURCE_TYPE_NAME,
        }
    }

    pub(crate) fn feature_resolver(&self) -> &'static dyn TypeFeatureResolver {
        match self {
            DynTypeKind::Iterable => &IterableType,
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
                        "integer" => format!("Expected '{}'", IntegerType::SOURCE_TYPE_NAME),
                        "str" => format!("Expected '{}'", StringType::SOURCE_TYPE_NAME),
                        "character" => format!("Expected '{}'", CharType::SOURCE_TYPE_NAME),
                        "list" | "vec" | "tuple" => {
                            format!("Expected '{}'", ArrayType::SOURCE_TYPE_NAME)
                        }
                        "obj" => format!("Expected '{}'", ObjectType::SOURCE_TYPE_NAME),
                        _ => format!("Unknown type '{}'", name),
                    },
                };
                return span.parse_err(error_message);
            }
        };
        Ok(Self { span, kind })
    }
}

impl HasSpan for TypeIdent {
    fn span(&self) -> Span {
        self.span
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
    pub(in crate::expressions) fn feature_resolver(&self) -> &'static dyn TypeFeatureResolver {
        match self {
            TypeKind::Leaf(leaf_kind) => leaf_kind.feature_resolver(),
            TypeKind::Parent(parent_kind) => parent_kind.feature_resolver(),
            TypeKind::Dyn(dyn_kind) => dyn_kind.feature_resolver(),
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
        let resolver = self.source_type.kind.feature_resolver();
        // TODO[performance] - lazily initialize properties as Shared
        let property_name = &self.property.to_string();
        if let Some(value) = resolver.resolve_type_property(property_name) {
            return ownership.map_from_shared(Spanned(
                SharedValue::new_from_owned(value.into_any_value()),
                self.span_range(),
            ));
        }
        if let Some(method) = resolver.resolve_method(property_name) {
            return ownership.map_from_shared(Spanned(
                SharedValue::new_from_owned(method.into_any_value()),
                self.span_range(),
            ));
        }
        if let Some(function) = resolver.resolve_type_function(property_name) {
            return ownership.map_from_shared(Spanned(
                SharedValue::new_from_owned(function.into_any_value()),
                self.span_range(),
            ));
        }
        return self.type_err(format!(
            "Type '{}' has no property, function or method named '{}'",
            self.source_type.kind.source_name(),
            self.property,
        ));
    }
}

impl HasSpanRange for TypeProperty {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.source_type.span, self.property.span())
    }
}
