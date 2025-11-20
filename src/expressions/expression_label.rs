use super::*;

pub(crate) struct CatchLabel {
    pub(crate) label: syn::Lifetime,
    pub(crate) colon: Token![:],
    pub(crate) catch_location_id: CatchLocationId,
}

impl CatchLabel {
    pub(crate) fn ident_string(&self) -> String {
        self.label.ident.to_string()
    }
}

impl syn::parse::Parse for CatchLabel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lifetime = input.parse()?;
        let colon = input.parse()?;
        Ok(Self {
            label: lifetime,
            colon,
            catch_location_id: CatchLocationId::new_placeholder(),
        })
    }
}

impl ParseSourceOptional for CatchLabel {}

impl HasSpanRange for CatchLabel {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.label.apostrophe, self.colon.span)
    }
}
