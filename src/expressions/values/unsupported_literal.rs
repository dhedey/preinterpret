use super::*;

#[derive(Clone)]
pub(crate) struct UnsupportedLiteral {
    pub(crate) lit: syn::Lit,
}

impl UnsupportedLiteral {
    /// Compare two unsupported literals for equality by comparing their token string representation.
    pub(super) fn literals_equal(lhs: &UnsupportedLiteral, rhs: &UnsupportedLiteral) -> bool {
        use quote::ToTokens;
        lhs.lit.to_token_stream().to_string() == rhs.lit.to_token_stream().to_string()
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
