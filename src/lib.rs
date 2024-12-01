//! This crate provides the `preinterpret!` macro, which works as a simple pre-processor to the token stream, and is designed for declarative macro builders.
//!
//! It provides two composable features:
//! * Variable definition with `[!set! #variable = ... ]` and variable substition with `#variable` (think [quote](https://crates.io/crates/quote) for declarative macros).
//! * A toolkit of simple functions operating on token streams, literals and idents, such as `[!ident! Hello #world]` (think [paste](https://crates.io/crates/paste) but more comprehesive, and still maintained).
//!
//! It is inspired by the [quote](https://crates.io/crates/quote) and [paste](https://crates.io/crates/paste) crates, and built for declarative macro authors to provide:
//!
//! * **Heightened readability** - allowing developers to build more maintainable macros.
//! * **Heightened expressivity** - mitigating the need to build custom procedural macros.
//! * **Heightened sensibility** - helping developers avoid various declarative macro surprises.
//!
//! ## Motivation
//!
//! ### Heightened readability
//!
//! The preinterpret syntax is intended to be immediately intuitive even for people not familiar with the crate. And it enables developers to make more readable macros:
//! * Developers can name clear concepts in their macro output, and re-use them by name, decreasing code duplication.
//! * Developers can use variables to subdivide logic inside the macro, without having to resort to creating lots of small, functional helper macros.
//!
//! A simple example follows, but variable substitution becomes even more useful in larger macros with more boilerplate:
//!
//! ```rust,ignore
//! macro_rules! impl_marker_traits {
//!     {
//!         $vis:vis $type_name:ident
//!         $(
//!             // Arbitrary (non-const) type generics
//!             < $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >
//!         )?
//!         [
//!             // The marker traits to implement
//!             $($trait:ident),* $(,)?
//!         ]
//!     } => {preinterpret::preinterpret!{
//!         [!set! #impl_generics = $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?]
//!         [!set! #type_generics = $(< $( $lt ),+ >)?]
//!         [!set! #my_type = $type_name #type_generics]
//!
//!         $(
//!             // Output each marker trait for the type
//!             impl #impl_generics $trait for #my_type {}
//!         )*
//!     }}
//! }
//! ```
//!
//! ### Heightened Expressivity
//!
//! Preinterpret provides a suite of composable functions to convert token streams, literals and idents. The full list is documented in the [Details](#details) section.
//!
//! For example:
//!
//! ```rust,ignore
//! macro_rules! make_a_struct_and_getters {
//!     (
//!         $name:ident { $($field:ident),* (,)?
//!     }) => {preinterpret::preinterpret!{
//!         // Define a struct with the given fields
//!         pub struct $name {
//!             $(
//!                 $field: String,
//!             )*
//!         }
//!
//!         impl $name {
//!             $(
//!                 // Define get_X for each field X
//!                 pub fn [!ident! get_ $field](&self) -> &str {
//!                     &self.$field
//!                 }
//!             )*
//!         }
//!     }}
//! }
//! ```
//!
//! Variable assignment works intuitively with the `* + ?` expansion operators, allowing basic procedural logic, such as creation of loop counts and indices before [meta-variables](https://github.com/rust-lang/rust/issues/83527) are stabilized.
//!
//! For example:
//! ```rust,ignore
//! macro_rules! count_idents {
//!     {
//!         $($item: ident),*
//!     } => {preinterpret::preinterpret!{
//!         [!set! #current_index = 0]
//!         $(
//!             [!ignore! $item] // Loop over the items, but don't output them
//!             [!increment! #current_index]
//!         )*
//!         [!set! #count = #current_index]
//!         #count
//!     }}
//! }
//! ```
//!
//! To quickly explain how this works, imagine we evaluate `count_idents!(a, b, c)`. As `count_idents!` is the most outer macro, it runs first, and expands into the following token stream:
//!
//! ```rust,ignore
//! preinterpret::preinterpret!{
//!   [!set! #current_index = 0]
//!   [!ignore! a]
//!   [!increment! #current_index]
//!   [!ignore! = b]
//!   [!increment! #current_index]
//!   [!ignore! = c]
//!   [!increment! #current_index]
//!   [!set! #count = #current_index]
//!   #count
//! }
//! ```
//! Now the `preinterpret!` macro runs, resulting in `#count` equal to the literal `3`.
//!
//! ### Heightened sensibility
//!
//! Using preinterpret partially mitigates some common issues when writing declarative macros.
//!
//! #### Cartesian zip confusion
//!
//! The declarative macro evaluator zips metavariables together, but sometimes you wish to loop over to separate meta-variables at once - but this [gives an unhelpful error message](https://github.com/rust-lang/rust/issues/96184#issue-1207293401).
//!
//! The classical wisdom is to output an internal `macro_rules!` definition to handle the inner output of the cartesian product [as per this stack overflow post](https://stackoverflow.com/a/73543948), but this isn't very intuitive.
//!
//! Standard use of preinterpret avoids this problem entirely. The example under the readability demonstrates this. If written without preinterpret, the iteration of the generics in `#impl_generics` and `#my_type` wouldn't be compatible with the iteration over `$trait`.
//!
//! #### Eager macro confusion
//!
//! User-defined macros are not eager - they take a token stream in, and return a token stream; and further macros can then execute in this token stream.
//!
//! But confusingly, some compiler built-in macros in the standard library (such as `format_args!`, `concat!`, `concat_idents!` and `include!`) don't work like this - they actually inspect their arguments, evaluate any macros inside eagerly, before then operating on the outputted tokens.
//!
//! Don't get me wrong - it's useful that you can nest `concat!` calls and `include!` calls - but the fact that these macros use the same syntax as "normal" macros but use different resolution behaviour can cause confusion to developers first learning about macros.
//!
//! Preinterpet commands also typically interpret their arguments eagerly and recursively, but it tries to be less confusing by:
//! * Having a clear name (Preinterpet) which suggests eager pre-processing.
//! * Using a different syntax `[!command! ...]` to macros to avoid confusion.
//! * Taking on the functionality of the `concat!` and `concat_idents!` macros so they don't have to be used alongside other macros.
//!
//! #### Recursive function paradigm shift
//!
//! To do anything particularly advanced with declarative macros, you end up needing to conjure up various functional macro helpers to partially apply or re-order grammars. This is quite a paradigm-shift from most rust code.
//!
//! In quite a few cases, preinterpret can allow developers to avoid writing these recursive helper macros entirely.
//!
//! #### Paste limitations problem
//!
//! The widely used [paste](https://crates.io/crates/paste) crate which inspired this crate has a few awkward issues.
//!
//! The issue which originally inspired `printerpret` was that [paste doesn't work well inside attributes](https://github.com/dtolnay/paste/issues/99#issue-1909928493).
//!
//! Preinterpret doesn't have any such restrictions. For example:
//!
//! ```rust,ignore
//! macro_rules! impl_new_type {
//!     {
//!         $vis:vis $my_type:ident($my_inner_type:ty)
//!     } => {preinterpret::preinterpret!{
//!         #[xyz(as_type = [!string! $my_inner_type])]
//!         $vis struct $my_type($my_inner_type);
//!     }}
//! }
//! ```
//!
//! ## Details
//!
//! Each command except `raw` resolves in a nested manner as you would expect:
//! ```rust,ignore
//! [!set! #foo = fn [!ident! get_ [!snake_case! Hello World]]()]
//! #foo // "fn get_hello_world()"
//! ```
//!
//! ### Core commands
//!
//! * `[!set! #foo = Hello]` followed by `[!set! #foo = #bar(World)]` sets the variable `#foo` to the token stream `Hello` and `#bar` to the token stream `Hello(World)`, and outputs no tokens. Using `#foo` or `#bar` later on will output the current value in the corresponding variable.
//! * `[!raw! abc #abc [!ident! test]]` outputs its contents as-is, without any interpretation, giving the token stream `abc #abc [!ident! test]`.
//! * `[!ignore! $foo]` ignores all of its content and outputs no tokens. It is useful to make a declarative macro loop over a meta-variable without outputting it into the resulting stream.
//!
//! ### Concatenate and convert commands
//!
//! Each of these commands functions in three steps:
//! * Apply the interpreter to the token stream, which recursively executes preinterpret commands.
//! * Convert each token of the resulting stream into a string, and concatenate these together. String and char literals are unquoted, and this process recurses into groups.
//! * Apply some command-specific conversion.
//!
//! The grammar value conversion commands are:
//!
//! * `[!string! X Y " " Z (Hello World)]` outputs `"XY Z(HelloWorld)"`
//! * `[!ident! X Y "Z"]` outputs the ident `XYZ`
//! * `[!literal! 31 u 32]` outputs the integer literal `31u32`
//! * `[!literal! '"' hello '"']` outputs the string literal `"hello"`
//!
//! The supported string conversion commands are:
//!
//! * `[!upper_case! foo_bar]` outputs `"FOO_BAR"`
//! * `[!lower_case! FooBar]` outputs `"foobar"`
//! * `[!snake_case! FooBar]` and `[!lower_snake_case! FooBar]` are equivalent and output `"foo_bar"`
//! * `[!upper_snake_case! FooBar]` outputs `"FOO_BAR"`
//! * `[!camel_case! foo_bar]` and `[!upper_camel_case! foo_bar]` are equivalent and output `"FooBar"`
//! * `[!lower_camel_case! foo_bar]` outputs `"fooBar"`
//! * `[!capitalize! fooBar]` outputs `"FooBar"`
//! * `[!decapitalize! FooBar]` outputs `"fooBar"`
//!
//! To create idents from these methods, simply nest them, like so:
//! ```rust,ignore
//! [!ident! get_ [!snake_case! $field_name]]
//! ```
//!
//! > [!NOTE]
//! >
//! > These string conversion methods are designed to work intuitively across a relatively wide class of input strings, but treat all characters which are not lowercase or uppercase as word boundaries.
//! >
//! > Such characters get dropped in camel case conversions. This could break up grapheme clusters and cause other non-intuitive behaviour. See the [tests in string_conversion.rs](https://www.github.com/dhedey/preinterpret/blob/main/src/string_conversion.rs) for more details.
//!
//! ### Integer commands (COMING SOON!)
//!
//! Each of these commands functions in three steps:
//! * Apply the interpreter to the token stream, which recursively executes preinterpret commands.
//! * Iterate over each token (recursing into groups), expecting each to be an integer literal.
//! * Apply some command-specific mapping to this stream of integer literals, and output a single integer literal without its type suffix. The suffix can be added back manually if required with a wrapper such as `[!literal! [!add! 1 2] u64]`.
//!
//! The supported integer commands are:
//!
//! * `[!add! 5u64 9 32]` outputs `46`. It takes any number of integers and outputs their sum. The calculation operates in `u128` space.
//! * `[!sub! 64u32 1u32]` outputs `63`. It takes two integers and outputs their difference. The calculation operates in `i128` space.
//! * `[!mod! $length 2]` outputs `0` if `$length` is even, else `1`. It takes two integers `a` and `b`, and outputs `a mod b`.
//!
//! We also support the following assignment commands:
//! * `[!increment! #i]` is shorthand for `[!set! #i [!add! #i 1]]` and outputs no  tokens.
//!
//! ### Boolean commands (COMING SOON!)
//!
//! Each of these commands functions in three steps:
//! * Apply the interpreter to the token stream, which recursively executes preinterpret commands.
//! * Expects to read exactly two token trees (unless otherwise specified)
//! * Apply some command-specific comparison, and outputs the boolean literal `true` or `false`.
//!
//! The supported comparison commands are:
//! * `[!eq! #foo #bar]` outputs `true` if `#foo` and `#bar` are exactly the same token tree, via structural equality. For example:
//!   * `[!eq! (3 4) (3   4)]` outputs `true` because the token stream ignores spacing.
//!   * `[!eq! 1u64 1]` outputs `false` because these are different literals.
//! * `[!lt! #foo #bar]` outputs `true` if `#foo` is an integer literal and less than `#bar`
//! * `[!gt! #foo #bar]` outputs `true` if `#foo` is an integer literal and greater than `#bar`
//! * `[!lte! #foo #bar]` outputs `true` if `#foo` is an integer literal and less than or equal to `#bar`
//! * `[!gte! #foo #bar]` outputs `true` if `#foo` is an integer literal and greater than or equal to `#bar`
//! * `[!not! #foo]` expects a single boolean literal, and outputs the negation of `#foo`
//! * `[!str_contains! "needle" [!string! haystack]]` expects two string literals, and outputs `true` if the first string is a substring of the second string.
//!
//! ### Control flow commands (COMING SOON!)
//!
//! Currently, only `if` is supported:
//! * `[!if! #cond then { #a } else { #b }]` outputs `#a` if `#cond` is `true`, else `#b` if `#cond` is false.
//!
//! The `if` command works as follows:
//! * It starts by only interpreting its first token tree, and expects to see a single `true` or `false` literal.
//! * It then expects to reads an unintepreted `then` ident, following by a single `{ .. }` group, whose contents get interpreted and output only if the condition was `true`.
//! * It optionally also reads an `else` ident and a by a single `{ .. }` group, whose contents get interpreted and output only if the condition was `false`.

mod command;
mod commands;
mod internal_prelude;
mod interpreter;
mod parsing;
mod string_conversion;

use internal_prelude::*;

/// Runs a simple interpeter over the token stream, allowing for variable assignment and substitution,
/// and a toolkit of commands to simplify code generation.
///
/// Commands look like `[!command! arguments as token stream here]` and can be nested.
///
/// ## Command cheat sheet
/// * `[!set! #foo = ...]` set a variable to the provided token stream
/// * `#foo` outputs the variable's token stream
/// * `[!ident! ...]` outputs an ident from parsing the concatenated token stream
/// * `[!ident! [!snake_case! ...]]` outputs a lower-snake-cased ident from parsing the concatenated token stream
/// * `[!string! ...]` outputs the concatenated token stream
/// * `[!literal! ..]` outputs a literal from parsing the concatenated token stream
/// * `#[doc = [!string! "My documentation is for " #foo "."]]` can be used to create documentation strings
///
/// See the [crate-level documentation](crate) for full details.
#[proc_macro]
pub fn preinterpret(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    interpret(proc_macro2::TokenStream::from(token_stream))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
