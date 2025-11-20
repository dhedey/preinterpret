use super::*;

macro_rules! ExactIdent {
    [$ident:ident as $name:ident] => {
        pub(crate) struct $name(Ident);

        impl ParseSource for $name {
            fn parse(input: SourceParser) -> ParseResult<Self> {
                let ident = input.parse_ident_matching(stringify!($ident))?;
                Ok(Self(ident))
            }

            fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
                Ok(())
            }
        }

        impl HasSpan for $name {
            fn span(&self) -> Span {
                self.0.span()
            }
        }
    }
}

// FULL KEYWORDS
// These are keywords which have special meaning in the language
// and cannot be used as identifiers.

pub(crate) mod keyword {
    pub(crate) const REVERT: &str = "revert";
    pub(crate) const EMIT: &str = "emit";
    pub(crate) const ATTEMPT: &str = "attempt";
}

pub(crate) fn is_keyword(ident: &str) -> bool {
    matches!(ident, keyword::REVERT | keyword::EMIT | keyword::ATTEMPT)
}

ExactIdent![emit as EmitKeyword];
ExactIdent![revert as RevertKeyword];
ExactIdent![attempt as AttemptKeyword];

// CONTEXTUAL KEYWORDS
// These can still be used as identifiers in other contexts.

ExactIdent![group as GroupKeyword];
ExactIdent![raw as RawKeyword];
