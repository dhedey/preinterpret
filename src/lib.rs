//! # Preinterpet - The code generation toolkit
//!
//! [<img alt="github" src="https://img.shields.io/badge/github-dhedey/preinterpret-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dhedey/preinterpret)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/preinterpret.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/preinterpret)
//! [<img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/preinterpret?style=for-the-badge&logo=rust&logoColor=green&color=green" height="20">](https://crates.io/crates/preinterpret)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-preinterpret-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/preinterpret)
//! [<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dhedey/preinterpret/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/dhedey/preinterpret/actions?query=branch%3Amain)
//!
//! <!--
//! If updating this readme, please ensure that the lib.rs rustdoc is also updated:
//! * Copy the whole of this document to a new text file
//! * Replace `\n` with `\n//! ` prefix to each line
//! * Also fix the first line
//! * Paste into `lib.rs`
//! * Run ./style-fix.sh
//! -->
//!
//! This crate provides the `preinterpret!` macro, which works as a simple pre-processor to the token stream. It takes inspiration from and effectively combines the [quote](https://crates.io/crates/quote), [paste](https://crates.io/crates/paste) and [syn](https://crates.io/crates/syn) crates, to empower code generation authors and declarative macro writers, bringing:
//!
//! * **Heightened [readability](#readability)** - quote-like variable definition and substitution make it easier to work with code generation code.
//! * **Heightened [expressivity](#expressivity)** - a toolkit of simple commands reduce boilerplate, and mitigate the need to build custom procedural macros in some cases.
//! * **Heightened [simplicity](#simplicity)** - helping developers avoid the confusing corners [[1](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html), [2](https://github.com/rust-lang/rust/issues/96184#issue-1207293401), [3](https://veykril.github.io/tlborm/decl-macros/minutiae/metavar-and-expansion.html), [4](https://veykril.github.io/tlborm/decl-macros/patterns/push-down-acc.html)] of declarative macro land.
//!
//! The `preinterpret!` macro can be used inside the output of a declarative macro, or by itself, functioning as a mini code generation tool all of its own.
//!
//! ```toml
//! [dependencies]
//! preinterpret = "0.2"
//! ```
//!
//! ## User Guide
//!
//! Preinterpret works with its own very simple language, with two pieces of syntax:
//!
//! * **Commands**: `[!command_name! input token stream...]` take an input token stream and output a token stream. There are a number of commands which cover a toolkit of useful functions.
//! * **Variables**: `[!set! #var_name = token stream...]` defines a variable, and `#var_name` substitutes the variable into another command or the output.
//!
//! Commands can be nested intuitively. The input of all commands (except `[!raw! ...]`) are first interpreted before the command itself executes.
//!
//! ### Declarative macro example
//!
//! The following artificial example demonstrates how `preinterpret` can be integrate into declarative macros, and covers use of variables, idents and case conversion:
//!
//! ```rust
//! macro_rules! create_my_type {
//!     (
//!         $(#[$attributes:meta])*
//!         $vis:vis struct $type_name:ident {
//!             $($field_name:ident: $inner_type:ident),* $(,)?
//!         }
//!     ) => {preinterpret::preinterpret! {
//!         [!set! #type_name = [!ident! My $type_name]]
//!         
//!         $(#[$attributes])*
//!         $vis struct #type_name {
//!             $($field_name: $inner_type,)*
//!         }
//!
//!         impl #type_name {
//!             $(
//!                 fn [!ident_snake! my_ $inner_type](&self) -> &$inner_type {
//!                     &self.$field_name
//!                 }
//!             )*
//!         }
//!     }}
//! }
//! create_my_type! {
//!     struct Struct {
//!         field0: String,
//!         field1: u64,
//!     }
//! }
//! assert_eq!(MyStruct { field0: "Hello".into(), field1: 21 }.my_string(), "Hello")
//! ```
//!
//! ### Quick background on token streams and macros
//!
//! To properly understand how preinterpret works, we need to take a very brief detour into the language of macros.
//!
//! In Rust, the input and output to a macro is a [`TokenStream`](https://doc.rust-lang.org/proc_macro/enum.TokenStream.html). A `TokenStream` is simply an iterator of [`TokenTree`](https://doc.rust-lang.org/proc_macro/enum.TokenTree.html)s at a particular nesting level. A token tree is one of four things:
//!
//! * A [`Group`](https://doc.rust-lang.org/proc_macro/struct.Group.html) - typically `(..)`, `[..]` or `{..}`. It consists of a matched pair of [`Delimiter`s](https://doc.rust-lang.org/proc_macro/enum.Delimiter.html) and an internal token stream. There is also a transparent delimiter, used to group the result of token stream substitutions (although [confusingly](https://github.com/rust-lang/rust/issues/67062) a little broken in rustc).
//! * An [`Ident`](https://doc.rust-lang.org/proc_macro/struct.Ident.html) - An unquoted string, used to identitied something named. Think `MyStruct`, or `do_work` or `my_module`. Note that keywords such as `struct` or `async` and the values `true` and `false` are classified as idents at this abstraction level.
//! * A [`Punct`](https://doc.rust-lang.org/proc_macro/struct.Punct.html) - A single piece of punctuation. Think `!` or `:`.
//! * A [`Literal`](https://doc.rust-lang.org/proc_macro/struct.Literal.html) - This includes string literals `"my string"`, char literals `'x'` and numeric literals `23` / `51u64`.
//!
//! When you return output from a macro, you are outputting back a token stream, which the compiler will interpret.
//!
//! Preinterpret commands take token streams as input, and return token streams as output.
//!
//! ### Migration from paste
//!
//! If migrating from [paste](https://crates.io/crates/paste), the main difference is that you need to specify _what kind of concatenated thing you want to create_. Paste tried to work this out magically from context, but sometimes got it wrong.
//!
//! In other words, you typically want to replace `[< ... >]` with `[!ident! ...]`, and sometimes `[!string! ...]` or `[!literal! ...]`:
//! * To create type and function names, use `[!ident! My #preinterpret_type_name $macro_type_name]`, `[!ident_camel! ...]`, `[!ident_snake! ...]` or `[!ident_upper_snake! ...]`
//! * For doc macros or concatenated strings, use `[!string! "My type is: " #type_name]`
//! * If you're creating literals of some kind by concatenating parts together, use `[!literal! 32 u32]`
//!
//! For example:
//!
//! ```rust
//! preinterpret::preinterpret! {
//!     [!set! #type_name = [!ident! HelloWorld]]
//!
//!     struct #type_name;
//!
//!     #[doc = [!string! "This type is called [`" #type_name "`]"]]
//!     impl #type_name {
//!         fn [!ident_snake! say_ #type_name]() -> &'static str {
//!             [!string! "It's time to say: " [!title! #type_name] "!"]
//!         }
//!     }
//! }
//! assert_eq!(HelloWorld::say_hello_world(), "It's time to say: Hello World!")
//! ```
//!
//! ## Command List
//!
//! ### Special commands
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
//! The following commands output idents:
//!
//! * `[!ident! X Y "Z"]` outputs the ident `XYZ`
//! * `[!ident_camel! my hello_world]` outputs `MyHelloWorld`
//! * `[!ident_snake! my_ HelloWorld]` outputs `my_hello_world`
//! * `[!ident_upper_snake! my_ const Name]` outputs `MY_CONST_NAME`
//!
//! The `!literal!` command outputs any kind of literal, for example:
//!
//! * `[!literal! 31 u 32]` outputs the integer literal `31u32`
//! * `[!literal! '"' hello '"']` outputs the string literal `"hello"`
//!
//! The following commands output strings, without dropping non-alphanumeric characters:
//!
//! * `[!string! X Y " " Z (Hello World)]` outputs `"XY Z(HelloWorld)"`
//! * `[!upper! foo_bar]` outputs `"FOO_BAR"`
//! * `[!lower! FooBar]` outputs `"foobar"`
//! * `[!capitalize! fooBar]` outputs `"FooBar"`
//! * `[!decapitalize! FooBar]` outputs `"fooBar"`
//!
//! The following commands output strings, whilst also dropping non-alphanumeric characters:
//!
//! * `[!snake! FooBar]` and `[!lower_snake! FooBar]` are equivalent and output `"foo_bar"`
//! * `[!upper_snake! FooBar]` outputs `"FOO_BAR"`
//! * `[!camel! foo_bar]` and `[!upper_camel! foo_bar]` are equivalent and output `"FooBar"`. This filters out non-alphanumeric characters.
//! * `[!lower_camel! foo_bar]` outputs `"fooBar"`
//! * `[!kebab! fooBar]` outputs `"foo-bar"`
//! * `[!title! fooBar]` outputs `"Foo Bar"`
//! * `[!insert_spaces! fooBar]` outputs `"foo Bar"`
//!
//! > [!NOTE]
//! >
//! > These string conversion methods are designed to work intuitively across a wide class of input strings, by creating word boundaries when going from non-alphanumeric to alphanumeric, lowercase to uppercase, or uppercase to uppercase if the next character is lowercase.
//! >
//! > The case-conversion commands which drop non-alphanumeric characters can potentially break up grapheme clusters and can cause unintuitive behaviour
//! > when used with complex unicode strings.  
//! >
//! > A wide ranging set of tests covering behaviour are in [tests/string.rs](https://www.github.com/dhedey/preinterpret/blob/main/tests/string.rs).
//!
//! ## Motivation
//!
//! ### Readability
//!
//! The preinterpret syntax is intended to be immediately intuitive even for people not familiar with the crate. It enables developers to make more readable macros:
//!
//! * Developers can name clear concepts in their macro output, and re-use them by name, decreasing code duplication.
//! * Developers can use variables to subdivide logic inside the macro, without having to resort to creating lots of small, functional helper macros.
//!
//! These ideas are demonstrated with the following simple example:
//!
//! ```rust
//! macro_rules! impl_marker_traits {
//!     {
//!         impl [
//!             // The marker traits to implement
//!             $($trait:ident),* $(,)?
//!         ] for $type_name:ident
//!         $(
//!             // Arbitrary (non-const) type generics
//!             < $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >
//!         )?
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
//! trait MarkerTrait1 {}
//! trait MarkerTrait2 {}
//! struct MyType<T: Clone>(T);
//! impl_marker_traits! {
//!     impl [MarkerTrait1, MarkerTrait2] for MyType<T: Clone>
//! };
//! ```
//!
//! ### Expressivity
//!
//! Preinterpret provides a suite of simple, composable commands to convert token streams, literals and idents. The full list is documented in the [Command List](#command-list) section.
//!
//! For example:
//!
//! ```rust
//! macro_rules! create_struct_and_getters {
//!     (
//!         $name:ident { $($field:ident),* $(,)? }
//!     ) => {preinterpret::preinterpret!{
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
//! create_struct_and_getters! {
//!   MyStruct { hello, world }
//! }
//! ```
//!
//! Variable assignment works intuitively with the `* + ?` expansion operators, allowing basic procedural logic, such as creation of loop counts and indices before [meta-variables](https://github.com/rust-lang/rust/issues/83527) are stabilized.
//!
//! For example:
//! ```rust
//! macro_rules! count_idents {
//!     {
//!         $($item: ident),*
//!     } => {preinterpret::preinterpret!{
//!         [!set! #current_index = 0usize]
//!         $(
//!             [!ignore! $item] // Loop over the items, but don't output them
//!             [!set! #current_index = #current_index + 1]
//!         )*
//!         [!set! #count = #current_index]
//!         #count
//!     }}
//! }
//! ```
//!
//! To quickly explain how this works, imagine we evaluate `count_idents!(a, b, c)`. As `count_idents!` is the most outer macro, it runs first, and expands into the following token stream:
//!
//! ```rust
//! let count = preinterpret::preinterpret!{
//!   [!set! #current_index = 0usize]
//!   [!ignore! a]
//!   [!set! #current_index = #current_index + 1]
//!   [!ignore! = b]
//!   [!set! #current_index = #current_index + 1]
//!   [!ignore! = c]
//!   [!set! #current_index = #current_index + 1]
//!   [!set! #count = #current_index]
//!   #count
//! };
//! ```
//!
//! Now the `preinterpret!` macro runs, resulting in `#count` equal to the token stream `0usize + 1 + 1 + 1`.
//! This will be improved in future releases by adding support for mathematical operations on integer literals.
//!
//! ### Simplicity
//!
//! Using preinterpret partially mitigates some common areas of confusion when writing declarative macros.
//!
//! #### Cartesian metavariable expansion errors
//!
//! Sometimes you wish to output some loop over one meta-variable, whilst inside the loop of a non-parent meta-variable - in other words, you expect to create a cartesian product across these variables. But the macro evaluator only supports zipping of meta-variables of the same length, and [gives an unhelpful error message](https://github.com/rust-lang/rust/issues/96184#issue-1207293401).
//!
//! The classical wisdom is to output an internal `macro_rules!` definition to handle the inner output of the cartesian product [as per this stack overflow post](https://stackoverflow.com/a/73543948), but this isn't very intuitive.
//!
//! Standard use of preinterpret avoids this problem entirely, as demonstrated by the first readability example. If written out natively without preinterpret, the iteration of the generics in `#impl_generics` and `#my_type` wouldn't be compatible with the iteration over `$trait`.
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
//! #### The recursive macro paradigm shift
//!
//! To do anything particularly advanced with declarative macros, you end up needing to conjure up various functional macro helpers to partially apply or re-order grammars. This is quite a paradigm-shift from most rust code.
//!
//! In quite a few cases, preinterpret can allow developers to avoid writing these recursive helper macros entirely.
//!
//! #### Limitations with paste support
//!
//! The widely used [paste](https://crates.io/crates/paste) crate takes the approach of magically hiding the token types from the developer, by attempting to work out whether a pasted value should be an ident, string or literal.
//!
//! This works 95% of the time, but in other cases such as [in attributes](https://github.com/dtolnay/paste/issues/99#issue-1909928493), it can cause developer friction. This proved to be one of the motivating use cases for developing preinterpret.
//!
//! Preinterpret is more explicit about types, and doesn't have these issues:
//!
//! ```rust
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
//! ## Future Extension Possibilities
//!
//! ### Add github docs page / rust book
//!
//! Add a github docs page / rust book at this repository, to allow us to build out a suite of examples, like `serde` or the little book of macros.
//!
//! ### Destructuring / Parsing Syntax, and Declarative Macros 2.0
//!
//! I have a vision for having preinterpret effectively replace the use of declarative macros in the Rust ecosystem, by:
//!
//! * Enabling writing intuitive, procedural code, which feels a lot like normal rust
//! * Exposing the power of [syn](https://crates.io/crates/syn) into this language, and preparing people to write procedural macros
//!
//! This would avoid pretty much all of the main the complexities of declarative macros:
//!
//! * The entirely lacking compile errors and auto-complete when it can't match tokens.
//! * What the metavariable types mean as `:ty`, `:tt`, `:vis` and how to use them all... And the fact that, once matched, all but `tt` are opaque for future matching.
//! * Having to learn a new paradigm of inverted thinking, which is pretty alien to rust.
//! * The `macro_rules!` declaration itself - I can never remember which brackets to use...
//!
//! The idea is that we create two new tools:
//!
//! * The `parse` command which can be built up of compasable parse-helpers (mostly wrapping `syn` calls), intuitively handling lots of common patterns seen in code
//! * Add control flow (`for`, `match` and the like) which runs lazily and declaratively, over the token stream primitive
//!
//! In more detail:
//!
//! * `[!parse! (DESTRUCTURING) = (INPUT)]` is a more general `[!set!]` which acts like a `let <XX> = <YY> else { panic!() }`. It takes a `()`-wrapped parse destructuring on the left and a token stream as input on the right. Any `#x` in the parse definition acts as a binding rather than as a substitution. Parsing will handled commas intelligently, and accept intelligent parse operations to do heavy-lifting for the user. Parse operations look like `[!OPERATION! DESTRUCTURING]` with the operation name in `UPPER_SNAKE_CASE`. Some examples might be:
//!     * `[!FIELDS! { hello: #a, world?: #b }]` - which can be parsed in any order, cope with trailing commas, and forbid fields in the source stream which aren't in the destructuring.
//!     * `[!SUBFIELDS! { hello: #a, world?: #b }]` - which can parse fields in any order, cope with trailing commas, and allow fields in the source stream which aren't in the destructuring.
//!     * `[!ITEM! { #ident, #impl_generics, ... }]` - which calls syn's parse item on the token
//!     * `[!IDENT! #x]`, `[!LITERAL! #x]`, `[!TYPE! { tokens: #x, path: #y }]` and the like to parse idents / literals etc directly from the token stream (rather than token streams). These will either take just a variable to capture the full token stream, or support an optional-argument style binding, where the developer can request certain sub-patterns or mapped token streams.
//!     * More tailored examples, such as `[!GENERICS! { impl: #x, type: #y, where: #z }]` which uses syn to parse the generics, and then uses subfields on the result.
//!     * Possibly `[!GROUPED! #x]` to parse a group with no brackets, to avoid parser ambiguity in some cases
//!     * `[!OPTIONAL! ...]` might be supported, but other complex logic (loops, matching) is delayed lazily until interpretation time - which feels more intuitive.
//! * `[!for! (DESTRUCTURING) in (INPUT) { ... }]` which operates like the rust `for` loop, and uses a parse destructuring on the left, and has support for optional commas between values
//! * `[!match! (INPUT) => { (DESTRUCTURING_1) => { ... }, (DESTRUCTURING_2) => { ... }, (#fallback) => { ... } }]` which operates like a rust `match` expression, and can replace the function of the branches of declarative macro inputs.
//! * `[!macro_rules! name!(DESTRUCTURING) = { ... }]` which can define a declarative macro, but just parses its inputs as a token stream, and uses preinterpret for its heavy lifting.
//!
//! And then we can end up with syntax like the following:
//!
//! ```rust,ignore
//! // =================================================
//! // Hypothetical future syntax - not yet implemented!
//! // =================================================
//!
//! // A simple macro can just take a token stream as input
//! preinterpret::preinterpret! {
//!     [!macro_rules! my_macro!(#input) {
//!         [!for! (#trait for #type) in (#input) {
//!             impl #trait for #type
//!         }]
//!     }]
//! }
//! my_macro!(
//!     MyTrait for MyType,
//!     MyTrait for MyType2,
//! );
//!
//! // It can also parse its input in the declaration.
//! // Repeated sections have to be captured as a stream, and delegated to explicit lazy [!for! ...] binding.
//! // This enforces a more procedural code style, and gives clearer compiler errors.
//! preinterpret::preinterpret! {
//!     [!macro_rules! multi_impl_super_duper!(
//!         #type_list,
//!         ImplOptions [!FIELDS! {
//!             greeting: #hello,
//!             location: #world,
//!             punctuation?: #punct = ("!") // Default
//!         }]
//!     ) = {
//!         [!for! (
//!             #type [!GENERICS! { impl: #impl_generics, type: #type_generics }]
//!         ) in (#type_list) {
//!             impl<#impl_generics> SuperDuper for #type #type_generics {
//!                 const Hello: &'static str = [!string! #hello " " #world #punct];
//!             }
//!         }]
//!     }]
//! }
//! ```
//!
//! ### Possible extension: Integer commands
//!
//! Each of these commands functions in three steps:
//! * Apply the interpreter to the token stream, which recursively executes preinterpret commands.
//! * Iterate over each token (recursing into groups), expecting each to be an integer literal.
//! * Apply some command-specific mapping to this stream of integer literals, and output a single integer literal without its type suffix. The suffix can be added back manually if required with a wrapper such as `[!literal! [!add! 1 2] u64]`.
//!
//! Integer commands under consideration are:
//!
//! * `[!add! 5u64 9 32]` outputs `46`. It takes any number of integers and outputs their sum. The calculation operates in `u128` space.
//! * `[!sub! 64u32 1u32]` outputs `63`. It takes two integers and outputs their difference. The calculation operates in `i128` space.
//! * `[!mod! $length 2]` outputs `0` if `$length` is even, else `1`. It takes two integers `a` and `b`, and outputs `a mod b`.
//!
//! We also support the following assignment commands:
//!
//! * `[!increment! #i]` is shorthand for `[!set! #i = [!add! #i 1]]` and outputs no tokens.
//!
//! Even better - we could even support calculator-style expression interpretation:
//!
//! * `[!usize! (5 + 10) / mod(4, 2)]` outputs `7usize`
//!
//! ### Possible extension: User-defined commands
//!
//! * `[!define! [!my_command! <PARSE_DESTRUCTURING>] { <OUTPUT> }]`
//!
//! ### Possible extension: Boolean commands
//!
//! Each of these commands functions in three steps:
//! * Apply the interpreter to the token stream, which recursively executes preinterpret commands.
//! * Expects to read exactly two token trees (unless otherwise specified)
//! * Apply some command-specific comparison, and outputs the boolean literal `true` or `false`.
//!
//! Comparison commands under consideration are:
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
//! ### Possible extension: Token stream commands
//!
//! * `[!skip! 4 from [#stream]]` reads and drops the first 4 token trees from the stream, and outputs the rest
//! * `[!ungroup! (#stream)]` outputs `#stream`. It expects to receive a single group (i.e. wrapped in brackets), and unwraps it.
//!
//! ### Possible extension: Control flow commands
//!
//! #### If statement
//!
//! `[!if! #cond then { #a } else { #b }]` outputs `#a` if `#cond` is `true`, else `#b` if `#cond` is false.
//!
//! The `if` command works as follows:
//! * It starts by only interpreting its first token tree, and expects to see a single `true` or `false` literal.
//! * It then expects to reads an unintepreted `then` ident, following by a single `{ .. }` group, whose contents get interpreted and output only if the condition was `true`.
//! * It optionally also reads an `else` ident and a by a single `{ .. }` group, whose contents get interpreted and output only if the condition was `false`.
//!
//! #### For loop
//!
//! * `[!for! #token_tree in [#stream] { ... }]`
//!
//! #### Goto and label
//!
//! * `[!label! loop_start]` - defines a label which can be returned to. Effectively, it takes a clones of the remaining token stream after the label in the interpreter.
//! * `[!goto! loop_start]` - jumps to the last execution of `[!label! loop_start]`. It unrolls the preinterpret stack (dropping all unwritten token streams) until it finds a stackframe in which the interpreter has the defined label, and continues the token stream from there.
//!
//! ```rust,ignore
//! // Hypothetical future syntax - not yet implemented!
//! preinterpret::preinterpret!{
//!     [!set! #i = 0]
//!     [!label! loop]
//!     const [!ident! AB #i]: u8 = 0;
//!     [!increment! #i]
//!     [!if! [!lte! #i 100] then { [!goto! loop] }]
//! }
//! ```
//!
//! ### Possible extension: Eager expansion of macros
//!
//! When [eager expansion of macros returning literals](https://github.com/rust-lang/rust/issues/90765) is stabilized, it would be nice to include a command to do that, which could be used to include code, for example: `[!expand_literal_macros! include!("my-poem.txt")]`.
//!
//! ### Possible extension: Explicit parsing feature to enable syn
//!
//! The heavy `syn` library is (in basic preinterpret) only needed for literal parsing, and error conversion into compile errors.
//!
//! We could add a parsing feature to speed up compile times a lot for stacks which don't need the parsing functionality.
//!
//! ## License
//!
//! Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE)
//! or the [MIT license](LICENSE-MIT) at your option.
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
//!
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
/// * `#foo` outputs the variable's saved token stream
/// * `[!ident! ...]` outputs an ident from parsing the concatenated token stream
/// * `[!ident_camel! ...]` outputs an UpperCamelCased ident from parsing the concatenated token stream
/// * `[!ident_snake! ...]` outputs a lower_snake_cased ident from parsing the concatenated token stream
/// * `[!ident_upper_snake! ...]` outputs an UPPER_SNAKE_CASED ident from parsing the concatenated token stream
/// * `[!string! ...]` outputs the concatenated token stream
/// * `[!literal! ..]` outputs a literal from parsing the concatenated token stream
/// * `#[doc = [!string! "My documentation is for " #my_type "."]]` can be used to create documentation strings
///
/// See the [crate-level documentation](crate) for full details.
#[proc_macro]
pub fn preinterpret(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    interpret(proc_macro2::TokenStream::from(token_stream))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

// This is the recommended way to run the doc tests in the readme
#[doc = include_str!("../README.md")]
#[cfg(doctest)] // Don't actually export this!
#[proc_macro]
pub fn readme_doctests(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unimplemented!()
}
