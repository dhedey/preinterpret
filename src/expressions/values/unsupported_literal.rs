use super::*;

#[derive(Clone)]
pub(crate) struct UnsupportedLiteral {
    pub(crate) lit: syn::Lit,
}

impl ValuesEqual for UnsupportedLiteral {
    /// Compares two unsupported literals by their token string representation.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        use quote::ToTokens;
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

impl HasValueKind for UnsupportedLiteral {
    type SpecificKind = ValueKind;

    fn kind(&self) -> ValueKind {
        ValueKind::UnsupportedLiteral
    }
}

define_interface! {
    struct UnsupportedLiteralTypeData,
    parent: ValueTypeData,
    pub(crate) mod unsupported_literal_interface {
        pub(crate) mod methods {}
        pub(crate) mod unary_operations {}
        pub(crate) mod binary_operations {}
        interface_items {}
    }
}
