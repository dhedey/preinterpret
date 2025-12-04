use super::*;

#[derive(Clone)]
pub(crate) struct UnsupportedLiteral {
    pub(crate) lit: syn::Lit,
}

impl ValuesEqual for UnsupportedLiteral {
    /// Compares two unsupported literals by their token string representation.
    fn values_equal<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        use quote::ToTokens;
        if self.lit.to_token_stream().to_string() == other.lit.to_token_stream().to_string() {
            ctx.equal()
        } else {
            ctx.not_equal(self, other)
        }
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
