mod arena;
mod dynamic_references;
mod errors;
mod field_inputs;
mod iterators;
mod keywords;
mod mut_rc_ref_cell;
mod parse_traits;
pub(crate) mod string_conversion;

pub(crate) use arena::*;
pub(crate) use dynamic_references::*;
pub(crate) use errors::*;
pub(crate) use field_inputs::*;
pub(crate) use iterators::*;
pub(crate) use keywords::*;
// Old RefCell-based abstractions - no longer glob-exported; replaced by dynamic_references.
// pub(crate) use mut_rc_ref_cell::*;
pub(crate) use parse_traits::*;

use crate::internal_prelude::*;

#[allow(unused)]
pub(crate) fn print_if_slow<T>(
    inner: impl FnOnce() -> T,
    slow_threshold: std::time::Duration,
    print_message: impl FnOnce(&T, std::time::Duration) -> String,
) -> T {
    use std::time::*;
    let before = SystemTime::now();
    let output = inner();
    let after = SystemTime::now();

    let elapsed = after.duration_since(before).unwrap();
    if elapsed >= slow_threshold {
        println!("{}", print_message(&output, elapsed));
    }
    output
}

// Equivalent to `!` but stable in our MSRV
pub(crate) enum Never {}

impl IsValueContent for Never {
    type Type = NoneType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for Never {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        match self {}
    }
}
