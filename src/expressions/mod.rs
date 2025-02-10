mod boolean;
mod character;
mod evaluation;
mod expression;
mod expression_parsing;
mod float;
mod integer;
mod operations;
mod stream;
mod string;
mod value;

// Marked as use for expression sub-modules to use with a `use super::*` statement
use crate::internal_prelude::*;
use boolean::*;
use character::*;
use evaluation::*;
use expression_parsing::*;
use float::*;
use integer::*;
use operations::*;
use stream::*;
use string::*;
use value::*;

pub(crate) use expression::*;
pub(crate) use integer::*;
pub(crate) use operations::*;
pub(crate) use value::*;
// For some reason Rust-analyzer didn't see it without this explicit export
pub(crate) use operations::BinaryOperation;
pub(crate) use value::ExpressionValue;
