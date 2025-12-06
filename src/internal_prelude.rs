pub(crate) use core::iter;
pub(crate) use core::marker::PhantomData;
pub(crate) use core::ops::{Deref, DerefMut};
pub(crate) use proc_macro2::extra::*;
pub(crate) use proc_macro2::*;
pub(crate) use quote::ToTokens;
pub(crate) use std::{
    borrow::Borrow,
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
    str::FromStr,
};
pub(crate) use syn::buffer::Cursor;
pub(crate) use syn::ext::IdentExt as SynIdentExt;
pub(crate) use syn::parse::{
    discouraged::Speculative, Parse as SynParse, ParseBuffer as SynParseBuffer,
    ParseStream as SynParseStream, Parser as SynParser,
};
pub(crate) use syn::punctuated::Punctuated;
pub(crate) use syn::{parse_str, Lit, LitFloat, LitInt, Token};
pub(crate) use syn::{Error as SynError, Result as SynResult};

pub(crate) use crate::expressions::*;
pub(crate) use crate::extensions::*;
pub(crate) use crate::interpretation::*;
pub(crate) use crate::misc::*;
