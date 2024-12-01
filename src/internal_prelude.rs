pub(crate) use core::iter;
pub(crate) use proc_macro2::*;
pub(crate) use std::{collections::HashMap, str::FromStr};
pub(crate) use syn::{parse_str, Error, Lit, Result};

pub(crate) use crate::command::*;
pub(crate) use crate::commands::*;
pub(crate) use crate::interpreter::*;
pub(crate) use crate::parsing::*;
pub(crate) use crate::string_conversion::*;

pub(crate) struct Tokens(iter::Peekable<<TokenStream as IntoIterator>::IntoIter>);

impl Tokens {
    pub(crate) fn new(tokens: TokenStream) -> Self {
        Self(tokens.into_iter().peekable())
    }

    pub(crate) fn peek(&mut self) -> Option<&TokenTree> {
        self.0.peek()
    }

    pub(crate) fn next(&mut self) -> Option<TokenTree> {
        self.0.next()
    }

    pub(crate) fn next_as_ident(&mut self) -> Option<Ident> {
        match self.next() {
            Some(TokenTree::Ident(ident)) => Some(ident),
            _ => None,
        }
    }

    pub(crate) fn next_as_punct_matching(&mut self, char: char) -> Option<Punct> {
        match self.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == char => Some(punct),
            _ => None,
        }
    }

    pub(crate) fn into_token_stream(self) -> TokenStream {
        self.0.collect()
    }
}

pub(crate) trait SpanErrorExt {
    fn error(self, message: impl std::fmt::Display) -> Error;
}

impl SpanErrorExt for Span {
    fn error(self, message: impl std::fmt::Display) -> Error {
        Error::new(self, message)
    }
}
