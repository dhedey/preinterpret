//! This macro is a powerful but simple general-purpose tool to ease building declarative macros which create
//! new types.
//!
//! # Motivation and Examples
//!
//! Effectively it functions as a more powerful version of [paste!](https://github.com/dtolnay/paste),
//! whilst bringing the power of [quote!](https://docs.rs/quote/latest/quote/)'s variable
//! substitution to declarative macros.
//!
//! This approach neatly solves the following cases:
//! 1. Wanting `paste!` to output strings or work with [attributes other than doc](https://github.com/dtolnay/paste/issues/40#issuecomment-2062953012).
//! 2. Improves readability of long procedural macros through substitution of repeated segments.
//! 3. Avoiding defining internal `macro_rules!` to handle instances where you need to do a procedural macro repeat across two conflicting expansions .
//! 4. Alternatives to [meta-variables](https://github.com/rust-lang/rust/issues/83527) such as `$count`, `$index` before
//!    they are stabilized, and alternatives to some forms of append-only recursive declarative macros.
//!
//! An example of case 1:
//! ```rust
//! # extern crate sbor_derive;
//! # use sbor_derive::*;
//! #
//! macro_rules! impl_new_type {
//!     {
//!         $vis:vis $my_type:ident($my_inner_type:ty)
//!     } => {eager_replace!{
//!         #[sbor(as_type = [!stringify! $my_inner_type])]
//!         $vis struct $my_type($my_inner_type)
//!
//!         // ...
//!     }}
//! }
//! ```
//!
//! The following is an example of case 2 and case 3, which creates a much more readable macro.
//! This example is hard to do with a normal macro, because the iteration of the generics in `#ImplGenerics` and `#MyType` wouldn't be compatible with the iteration over `$trait`.
//! Instead, you have to work around it, for example with internal `macro_rules!` definitions [as per this stack overflow post](https://stackoverflow.com/a/73543948).
//!
//! Using the `!SET!` functionality, we can define these token streams earlier and output them in each loop iteration.
//! This also makes the intention of the macro writer much clearer, similar to [quote!](https://docs.rs/quote/latest/quote/)
//! in procedural macros:
//! ```rust
//! # extern crate sbor_derive;
//! # use sbor_derive::*;
//! #
//! macro_rules! impl_marker_traits {
//!     {
//!         $vis:vis $type_name:ident
//!         // Arbitrary generics
//!         $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
//!         [
//!             $($trait:ident),*
//!             $(,)? // Optional trailing comma
//!         ]
//!     } => {eager_replace!{
//!         [!SET! #ImplGenerics = $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?]
//!         [!SET! #TypeGenerics = $(< $( $lt ),+ >)?]
//!         [!SET! #MyType = $type_name #TypeGenerics]
//!
//!         // Output for each marker trait
//!         $(
//!             impl #ImplGenerics $trait for #MyType {}
//!         )*
//!     }}
//! }
//! ```
//!
//! An example of case 4 - a simple count function, without needing recursive macros:
//! ```rust
//! # extern crate sbor_derive;
//! # use sbor_derive::*;
//! #
//! macro_rules! count_idents {
//!     {
//!         $($value: ident),*
//!     } => {eager_replace!{
//!         [!SET! #index = 0]
//!         $(
//!             [!SET! #ignored = $value]
//!             [!SET! #index = #index + 1]
//!         )*
//!         #index
//!     }}
//! }
//! ```
//! To quickly work this through, take `count_idents!(a, b, c)`. As a first pass, the declarative macro expands, giving:
//! ```text
//! eager_replace!{
//!   [!SET! #index = 0]
//!   [!SET! ignored = a]
//!   [!SET! #index = #index + 1]
//!   [!SET! ignored = b]
//!   [!SET! #index = #index + 1]
//!   [!SET! ignored = c]
//!   [!SET! #index = #index + 1]
//!   #index
//! }
//! ```
//! Which then evaluates by setting `#index` to the token stream `0 + 1 + 1 + 1`, and then outputting that sum.
//!
//! # Details
//! ## Specific functions
//!
//! * `[!concat! X Y " " Z (Hello World)]` gives `"XY Z(HelloWorld)"` by concatenating each argument without spaces, and recursing inside groups. String and char literals are first unquoted. Spaces can be added with " ".
//! * `[!ident! X Y "Z"]` gives an ident `XYZ`, using the same algorithm as `concat`.
//! * `[!literal! 31 u 32]` gives `31u32`, using the same algorithm as `concat`.
//! * `[!raw! abc #abc [!ident! test]]` outputs its contents without any nested expansion, giving `abc #abc [!ident! test]`.
//! * `[!stringify! X Y " " Z]` gives `"X Y \" \" Z"` - IMPORTANT: This uses `token_stream.into_string()` which is compiler-version dependent. Do not use if that is important. Instead, the output from `concat` should be independent of compiler version.
//!
//! Note that all functions except `raw` resolve in a nested manner as you would expected, e.g.
//! ```rust,ignore
//! [!ident! X Y [!ident! Hello World] Z] // "XYHelloWorldZ"
//! ```
//!
//! ## Variables
//!
//! You can define variables starting with `#` which can be used outside the set call.
//! All of the following calls don't return anything, but create a variable, which can be embedded later in the macro.
//!
//! * `[!SET! #MyVar = ..]` sets `#MyVar` to the given token stream.
//! * `[!SET:concat! #MyVar = ..]` sets `#MyVar` to the result of applying the `concat` function to the token stream.
//! * `[!SET:ident! #MyVar = ..]` sets `#MyVar` to the result of applying the `ident` function to the token stream.
//! * `[!SET:literal! #MyVar = ..]` sets `#MyVar` to the result of applying the `literal` function to the token stream.
//! * `[!SET:raw! #MyVar = ..]` sets `#MyVar` to the result of applying the `raw` function to the token stream.
//! * `[!SET:stringify! #MyVar = ..]` sets `#MyVar` to the result of applying the `stringify` function to the token stream.
//!
//! # Future extensions
//! ## String case conversion
//!
//! This could in future support case conversion like [paste](https://docs.rs/paste/1.0.15/paste/index.html).
//! e.g. `[!snakecase! ..]`, `[!camelcase! ..]`, `[!uppercase! ..]`, `[!lowercase! ..]`, `[!capitalize! ..]`, `[!decapitalize! ..]`.
//! Which all use the `concat` algorithm to combine inputs, and then apply a string function.
//!
//! These can be composed to achieve things like `UPPER_SNAKE_CASE` or `lowerCamelCase`,
//!
//! # Hypothetical extensions
//! None of these are likely additions, but in theory, this system could be made turing complete to decrease the amount
//! you have to reach for writing your own procedural macros.
//!
//! ## Functions returning literals
//! * Integer functions like `[!sum! a b]`, `[!mod! a b]` which work on integer literal tokens.
//! * Boolean conditionals like `[!eq! a b]`, `[!lt! a b]`, `[!lte! a b]` operating on literals `[!contains! needle (haystack)]`
//!
//! ## Eager expansion of macros
//! When eager expansion of macros returning literals from https://github.com/rust-lang/rust/issues/90765 is stabilized,
//! things like `[!expand_literal_macros! include!("my-poem.txt")]` will be possible.
//!
//! ## Conditions and if statements
//! `[!IF! cond { .. } !ELSE! { .. }]`, for example `[!IF! [!eq! [!mod! $length 2] 0] { "even length" } !ELSE! { "odd length" }]`.
//!
//! ## Labels and gotos
//! `[!LABEL:loop!]` and `[!GOBACKTO:loop!]` would bring turing completeness - although it would need a re-architecture
//! of the token streaming logic to support jumping backwards in the stream.


mod internal_prelude;
mod parsing;
mod interpreter;
mod commands;

use internal_prelude::*;

/// See the [crate-level documentation](crate) for more information.
#[proc_macro]
pub fn preinterpret(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    interpret(proc_macro2::TokenStream::from(token_stream)).unwrap_or_else(|err| err.to_compile_error()).into()
}
