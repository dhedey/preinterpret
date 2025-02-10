use crate::internal_prelude::*;

pub(crate) trait TokenStreamExt: Sized {
    #[allow(unused)]
    fn push(&mut self, token: TokenTree);
    fn flatten_transparent_groups(self) -> Self;
}

impl TokenStreamExt for TokenStream {
    fn push(&mut self, token: TokenTree) {
        self.extend(iter::once(token));
    }

    fn flatten_transparent_groups(self) -> Self {
        let mut output = TokenStream::new();
        for token in self {
            match token {
                TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                    output.extend(group.stream().flatten_transparent_groups());
                }
                other => output.extend(iter::once(other)),
            }
        }
        output
    }
}

pub(crate) trait IdentExt: Sized {
    fn new_bool(value: bool, span: Span) -> Self;
    fn with_span(self, span: Span) -> Self;
}

impl IdentExt for Ident {
    fn new_bool(value: bool, span: Span) -> Self {
        Ident::new(&value.to_string(), span)
    }

    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}

pub(crate) trait LiteralExt: Sized {
    #[allow(unused)]
    fn content_if_string(&self) -> Option<String>;
    fn content_if_string_like(&self) -> Option<String>;
}

impl LiteralExt for Literal {
    fn content_if_string(&self) -> Option<String> {
        match parse_str::<Lit>(&self.to_string()).unwrap() {
            Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        }
    }

    fn content_if_string_like(&self) -> Option<String> {
        match parse_str::<Lit>(&self.to_string()).unwrap() {
            Lit::Str(lit_str) => Some(lit_str.value()),
            Lit::Char(lit_char) => Some(lit_char.value().to_string()),
            Lit::CStr(lit_cstr) => Some(lit_cstr.value().to_string_lossy().to_string()),
            _ => None,
        }
    }
}

pub(crate) trait WithSpanExt {
    fn with_span(self, span: Span) -> Self;
}

impl WithSpanExt for Literal {
    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}

impl WithSpanExt for Punct {
    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}

impl WithSpanExt for Group {
    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}
