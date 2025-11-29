use super::*;

pub(crate) struct CatchLabel {
    label: syn::Lifetime,
    _colon: Unused<Token![:]>,
}

impl CatchLabel {
    pub(crate) fn ident_string(&self) -> String {
        self.label.ident.to_string()
    }
}

impl ParseSource for CatchLabel {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let lifetime = input.parse()?;
        let _colon = input.parse()?;
        Ok(Self {
            label: lifetime,
            _colon,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

impl ParseSourceOptional for CatchLabel {}
