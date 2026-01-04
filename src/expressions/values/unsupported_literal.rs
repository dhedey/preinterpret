use super::*;

define_leaf_type! {
    pub(crate) UnsupportedLiteralType => ValueType(ValueContent::UnsupportedLiteral),
    content: syn::Lit,
    kind: pub(crate) UnsupportedLiteralKind,
    type_name: "unsupported_literal",
    articled_display_name: "an unsupported literal",
    dyn_impls: {},
}

#[derive(Clone)]
pub(crate) struct UnsupportedLiteral {
    pub(crate) lit: syn::Lit,
}

impl ValuesEqual for UnsupportedLiteral {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.lit.to_token_stream().to_string() == other.lit.to_token_stream().to_string() {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
        }
    }
}

impl Debug for UnsupportedLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lit.to_token_stream())
    }
}

impl HasLeafKind for UnsupportedLiteral {
    type LeafKind = UnsupportedLiteralKind;

    fn kind(&self) -> Self::LeafKind {
        UnsupportedLiteralKind
    }
}

define_type_features! {
    impl UnsupportedLiteralType,
    parent: ValueType,
    pub(crate) mod unsupported_literal_interface {
        pub(crate) mod methods {}
        pub(crate) mod unary_operations {}
        pub(crate) mod binary_operations {}
        interface_items {}
    }
}
