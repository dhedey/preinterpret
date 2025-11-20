use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionLabel {
    pub(crate) lifetime: syn::Lifetime,
    pub(crate) colon: Token![:],
}

impl std::fmt::Debug for ExpressionLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpressionLabel")
            .field("label", &self.ident_string())
            .finish()
    }
}

impl ExpressionLabel {
    pub(crate) fn ident_string(&self) -> String {
        self.lifetime.ident.to_string()
    }
}

impl syn::parse::Parse for ExpressionLabel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lifetime = input.parse()?;
        let colon = input.parse()?;
        Ok(Self { lifetime, colon })
    }
}

impl HasSpanRange for ExpressionLabel {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.lifetime.apostrophe, self.colon.span)
    }
}
