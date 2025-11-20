use super::*;

pub(crate) struct ExpressionLabel {
    pub(crate) label: syn::Lifetime,
    pub(crate) colon: Token![:],
}

impl ExpressionLabel {
    pub(crate) fn ident_string(&self) -> String {
        self.label.ident.to_string()
    }
}

impl syn::parse::Parse for ExpressionLabel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lifetime = input.parse()?;
        let colon = input.parse()?;
        Ok(Self {
            label: lifetime,
            colon,
        })
    }
}

impl ParseSourceOptional for ExpressionLabel {}

impl HasSpanRange for ExpressionLabel {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.label.apostrophe, self.colon.span)
    }
}
