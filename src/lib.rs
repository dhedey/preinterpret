//! TODO - copy from README.md and adjust ```rust``` to be runnable as doctests

mod internal_prelude;
mod parsing;
mod interpreter;
mod command;
mod commands;
mod string_conversion;

use internal_prelude::*;

/// See the [crate-level documentation](crate) for more information.
#[proc_macro]
pub fn preinterpret(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    interpret(proc_macro2::TokenStream::from(token_stream)).unwrap_or_else(|err| err.to_compile_error()).into()
}
