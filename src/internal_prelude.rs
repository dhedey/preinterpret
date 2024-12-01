pub(crate) use core::iter;
pub(crate) use proc_macro2::*;
pub(crate) use std::{collections::HashMap, str::FromStr};
pub(crate) use syn::{parse_str, Error, Lit, Result};

pub(crate) type PeekableTokenIter = iter::Peekable<<TokenStream as IntoIterator>::IntoIter>;
pub(crate) use crate::interpreter::*;
pub(crate) use crate::parsing::*;
pub(crate) use crate::commands::*;

pub(crate) trait SpanErrorExt {
    fn error(self, message: impl std::fmt::Display) -> Error;
}

impl SpanErrorExt for Span {
    fn error(self, message: impl std::fmt::Display) -> Error {
        Error::new(self, message)
    }
}