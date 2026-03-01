pub(crate) use proc_macro2::{extra::*, *};
pub(crate) use quote::ToTokens;
pub(crate) use std::{
    borrow::{Borrow, Cow},
    cell::{RefCell, RefMut},
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
    fmt::Write,
    iter,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
    str::FromStr,
};
pub(crate) use syn::{
    buffer::Cursor,
    ext::IdentExt as SynIdentExt,
    parse::{
        discouraged::Speculative, Parse as SynParse, ParseBuffer as SynParseBuffer,
        ParseStream as SynParseStream, Parser as SynParser,
    },
    parse_str,
    punctuated::Punctuated,
    Error as SynError, Lit, LitFloat, LitInt, Result as SynResult, Token,
};

pub(crate) use crate::expressions::*;
pub(crate) use crate::extensions::*;
pub(crate) use crate::interpretation::*;
pub(crate) use crate::misc::*;
pub(crate) use crate::static_analysis::*;
