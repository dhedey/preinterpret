# Preinterpet - The code generation toolkit

[<img alt="github" src="https://img.shields.io/badge/github-dhedey/preinterpret-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dhedey/preinterpret)
[<img alt="crates.io" src="https://img.shields.io/crates/v/preinterpret.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/preinterpret)
[<img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/preinterpret?style=for-the-badge&logo=rust&logoColor=green&color=green" height="20">](https://crates.io/crates/preinterpret)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-preinterpret-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/preinterpret)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dhedey/preinterpret/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/dhedey/preinterpret/actions?query=branch%3Amain)

<!--
If updating this readme, please ensure that the rustdoc is also updated.
-->

This crate provides the `preinterpret!` macro, which works as a simple pre-processor to the token stream. It is inspired by the [quote](https://crates.io/crates/quote) and [paste](https://crates.io/crates/paste) crates, and built to empower code generation authors and declarative macro writers, bringing:

* **Heightened readability** - making it easier to work with code generation code.
* **Heightened expressivity** - reducing boilerplate, and mitigating the need to build custom procedural macros in some cases.
* **Heightened simplicity** - helping developers avoid various declarative macro surprises.

It provides two composable features:

* Variable definition with `[!set! #variable = ... ]` and variable substition with `#variable` (think [quote](https://crates.io/crates/quote) for declarative macros)
* A toolkit of simple functions operating on token streams, literals and idents, such as `[!ident! Hello #world]` (think [paste](https://crates.io/crates/paste) but more comprehesive, and still maintained)

The `preinterpret!` macro can be used inside the output of a declarative macro, or by itself, functioning as a mini code generation tool all of its own.

```toml
[dependencies]
preinterpret = "0.1"
```

## Motivation

### Readability

The preinterpret syntax is intended to be immediately intuitive even for people not familiar with the crate. And it enables developers to make more readable macros:

* Developers can name clear concepts in their macro output, and re-use them by name, decreasing code duplication.
* Developers can use variables to subdivide logic inside the macro, without having to resort to creating lots of small, functional helper macros.

These ideas are demonstrated with the following simple example:

```rust
macro_rules! impl_marker_traits {
    {
        impl [
            // The marker traits to implement
            $($trait:ident),* $(,)?
        ] for $type_name:ident
        $(
            // Arbitrary (non-const) type generics
            < $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >
        )?
    } => {preinterpret::preinterpret!{
        [!set! #impl_generics = $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?]
        [!set! #type_generics = $(< $( $lt ),+ >)?]
        [!set! #my_type = $type_name #type_generics]

        $(
            // Output each marker trait for the type
            impl #impl_generics $trait for #my_type {}
        )*
    }}
}
trait MarkerTrait1 {}
trait MarkerTrait2 {}
struct MyType<T: Clone>(T);
impl_marker_traits! {
    impl [MarkerTrait1, MarkerTrait2] for MyType<T: Clone>
};
```

### Expressivity

Preinterpret provides a suite of simple, composable commands to convert token streams, literals and idents. The full list is documented in the [Details](#details) section.

For example:

```rust
macro_rules! create_struct_and_getters {
    (
        $name:ident { $($field:ident),* $(,)? }
    ) => {preinterpret::preinterpret!{
        // Define a struct with the given fields
        pub struct $name {
            $(
                $field: String,
            )*
        }

        impl $name {
            $(
                // Define get_X for each field X
                pub fn [!ident! get_ $field](&self) -> &str {
                    &self.$field
                }
            )*
        }
    }}
}
create_struct_and_getters! {
  MyStruct { hello, world }
}
```

Variable assignment works intuitively with the `* + ?` expansion operators, allowing basic procedural logic, such as creation of loop counts and indices before [meta-variables](https://github.com/rust-lang/rust/issues/83527) are stabilized.

For example:
```rust
macro_rules! count_idents {
    {
        $($item: ident),*
    } => {preinterpret::preinterpret!{
        [!set! #current_index = 0usize]
        $(
            [!ignore! $item] // Loop over the items, but don't output them
            [!set! #current_index = #current_index + 1]
        )*
        [!set! #count = #current_index]
        #count
    }}
}
```

To quickly explain how this works, imagine we evaluate `count_idents!(a, b, c)`. As `count_idents!` is the most outer macro, it runs first, and expands into the following token stream:

```rust
let count = preinterpret::preinterpret!{
  [!set! #current_index = 0usize]
  [!ignore! a]
  [!set! #current_index = #current_index + 1]
  [!ignore! = b]
  [!set! #current_index = #current_index + 1]
  [!ignore! = c]
  [!set! #current_index = #current_index + 1]
  [!set! #count = #current_index]
  #count
};
```

Now the `preinterpret!` macro runs, resulting in `#count` equal to the token stream `0usize + 1 + 1 + 1`.
This will be improved in future releases by adding support for mathematical operations on integer literals.

### Heightened simplicity

Using preinterpret partially mitigates some common areas of confusion when writing declarative macros.

#### Cartesian metavariable expansion errors

Sometimes you wish to output some loop over one meta-variable, whilst inside the loop of a non-parent meta-variable - in other words, you expect to create a cartesian product across these variables. But the macro evaluator only supports zipping of meta-variables of the same length, and [gives an unhelpful error message](https://github.com/rust-lang/rust/issues/96184#issue-1207293401).

The classical wisdom is to output an internal `macro_rules!` definition to handle the inner output of the cartesian product [as per this stack overflow post](https://stackoverflow.com/a/73543948), but this isn't very intuitive.

Standard use of preinterpret avoids this problem entirely, as demonstrated by the first readability example. If written out natively without preinterpret, the iteration of the generics in `#impl_generics` and `#my_type` wouldn't be compatible with the iteration over `$trait`.

#### Eager macro confusion

User-defined macros are not eager - they take a token stream in, and return a token stream; and further macros can then execute in this token stream.

But confusingly, some compiler built-in macros in the standard library (such as `format_args!`, `concat!`, `concat_idents!` and `include!`) don't work like this - they actually inspect their arguments, evaluate any macros inside eagerly, before then operating on the outputted tokens.

Don't get me wrong - it's useful that you can nest `concat!` calls and `include!` calls - but the fact that these macros use the same syntax as "normal" macros but use different resolution behaviour can cause confusion to developers first learning about macros.

Preinterpet commands also typically interpret their arguments eagerly and recursively, but it tries to be less confusing by:
* Having a clear name (Preinterpet) which suggests eager pre-processing.
* Using a different syntax `[!command! ...]` to macros to avoid confusion.
* Taking on the functionality of the `concat!` and `concat_idents!` macros so they don't have to be used alongside other macros.

#### The recursive macro paradigm shift

To do anything particularly advanced with declarative macros, you end up needing to conjure up various functional macro helpers to partially apply or re-order grammars. This is quite a paradigm-shift from most rust code.

In quite a few cases, preinterpret can allow developers to avoid writing these recursive helper macros entirely.

#### Limitations with paste support

The widely used [paste](https://crates.io/crates/paste) crate takes the approach of magically hiding the token types from the developer, by attempting to work out whether a pasted value should be an ident, string or literal.

This works 95% of the time, but in other cases such as [in attributes](https://github.com/dtolnay/paste/issues/99#issue-1909928493), it can cause developer friction. This proved to be one of the motivating use cases for developing preinterpret.

Preinterpret is more explicit about types, and doesn't have these issues:

```rust
macro_rules! impl_new_type {
    {
        $vis:vis $my_type:ident($my_inner_type:ty)
    } => {preinterpret::preinterpret!{
        #[xyz(as_type = [!string! $my_inner_type])]
        $vis struct $my_type($my_inner_type);
    }}
}
```

## Details

Each command except `raw` resolves in a nested manner as you would expect:
```rust,ignore
[!set! #foo = fn [!ident! get_ [!snake_case! Hello World]]()]
#foo // "fn get_hello_world()"
```

### Core commands

* `[!set! #foo = Hello]` followed by `[!set! #foo = #bar(World)]` sets the variable `#foo` to the token stream `Hello` and `#bar` to the token stream `Hello(World)`, and outputs no tokens. Using `#foo` or `#bar` later on will output the current value in the corresponding variable.
* `[!raw! abc #abc [!ident! test]]` outputs its contents as-is, without any interpretation, giving the token stream `abc #abc [!ident! test]`.
* `[!ignore! $foo]` ignores all of its content and outputs no tokens. It is useful to make a declarative macro loop over a meta-variable without outputting it into the resulting stream.

### Concatenate and convert commands

Each of these commands functions in three steps:
* Apply the interpreter to the token stream, which recursively executes preinterpret commands.
* Convert each token of the resulting stream into a string, and concatenate these together. String and char literals are unquoted, and this process recurses into groups.
* Apply some command-specific conversion.

The grammar value conversion commands are:

* `[!string! X Y " " Z (Hello World)]` outputs `"XY Z(HelloWorld)"`
* `[!ident! X Y "Z"]` outputs the ident `XYZ`
* `[!literal! 31 u 32]` outputs the integer literal `31u32`
* `[!literal! '"' hello '"']` outputs the string literal `"hello"`

The supported string conversion commands are:

* `[!upper_case! foo_bar]` outputs `"FOO_BAR"`
* `[!lower_case! FooBar]` outputs `"foobar"`
* `[!snake_case! FooBar]` and `[!lower_snake_case! FooBar]` are equivalent and output `"foo_bar"`
* `[!upper_snake_case! FooBar]` outputs `"FOO_BAR"`
* `[!camel_case! foo_bar]` and `[!upper_camel_case! foo_bar]` are equivalent and output `"FooBar"`
* `[!lower_camel_case! foo_bar]` outputs `"fooBar"`
* `[!capitalize! fooBar]` outputs `"FooBar"`
* `[!decapitalize! FooBar]` outputs `"fooBar"`

To create idents from these methods, simply nest them, like so:
```rust,ignore
[!ident! get_ [!snake_case! $field_name]]
```

> [!NOTE]
>
> These string conversion methods are designed to work intuitively across a relatively wide class of input strings, but treat all characters which are not lowercase or uppercase as word boundaries.
>
> Such characters get dropped in camel case conversions. This could break up grapheme clusters and cause other non-intuitive behaviour. See the [tests in string_conversion.rs](https://www.github.com/dhedey/preinterpret/blob/main/src/string_conversion.rs) for more details.

## Future Extension Possibilities

### Add github docs page / rust book

Add a github docs page / rust book at this repository, to allow us to build out a suite of examples, like `serde` or the little book of macros.

### Possible extension: Integer commands

Each of these commands functions in three steps:
* Apply the interpreter to the token stream, which recursively executes preinterpret commands.
* Iterate over each token (recursing into groups), expecting each to be an integer literal.
* Apply some command-specific mapping to this stream of integer literals, and output a single integer literal without its type suffix. The suffix can be added back manually if required with a wrapper such as `[!literal! [!add! 1 2] u64]`.

Integer commands under consideration are:

* `[!add! 5u64 9 32]` outputs `46`. It takes any number of integers and outputs their sum. The calculation operates in `u128` space.
* `[!sub! 64u32 1u32]` outputs `63`. It takes two integers and outputs their difference. The calculation operates in `i128` space.
* `[!mod! $length 2]` outputs `0` if `$length` is even, else `1`. It takes two integers `a` and `b`, and outputs `a mod b`.

We also support the following assignment commands:

* `[!increment! #i]` is shorthand for `[!set! #i [!add! #i 1]]` and outputs no tokens.

We could even support:

* `[!usize! (5 + 10) / mod(4, 2)]` outputs `7usize`

### Possible extension: Boolean commands

Each of these commands functions in three steps:
* Apply the interpreter to the token stream, which recursively executes preinterpret commands.
* Expects to read exactly two token trees (unless otherwise specified)
* Apply some command-specific comparison, and outputs the boolean literal `true` or `false`.

Comparison commands under consideration are:
* `[!eq! #foo #bar]` outputs `true` if `#foo` and `#bar` are exactly the same token tree, via structural equality. For example:
  * `[!eq! (3 4) (3   4)]` outputs `true` because the token stream ignores spacing.
  * `[!eq! 1u64 1]` outputs `false` because these are different literals.
* `[!lt! #foo #bar]` outputs `true` if `#foo` is an integer literal and less than `#bar`
* `[!gt! #foo #bar]` outputs `true` if `#foo` is an integer literal and greater than `#bar`
* `[!lte! #foo #bar]` outputs `true` if `#foo` is an integer literal and less than or equal to `#bar`
* `[!gte! #foo #bar]` outputs `true` if `#foo` is an integer literal and greater than or equal to `#bar`
* `[!not! #foo]` expects a single boolean literal, and outputs the negation of `#foo`
* `[!str_contains! "needle" [!string! haystack]]` expects two string literals, and outputs `true` if the first string is a substring of the second string.

### Possible extension: Token stream commands

* `[!skip! 4 from [#stream]]` reads and drops the first 4 token trees from the stream, and outputs the rest
* `[!ungroup! (#stream)]` outputs `#stream`. It expects to receive a single group (i.e. wrapped in brackets), and unwraps it.

### Possible extension: Control flow commands

#### If statement

`[!if! #cond then { #a } else { #b }]` outputs `#a` if `#cond` is `true`, else `#b` if `#cond` is false.

The `if` command works as follows:
* It starts by only interpreting its first token tree, and expects to see a single `true` or `false` literal.
* It then expects to reads an unintepreted `then` ident, following by a single `{ .. }` group, whose contents get interpreted and output only if the condition was `true`.
* It optionally also reads an `else` ident and a by a single `{ .. }` group, whose contents get interpreted and output only if the condition was `false`.

#### For loop

* `[!for! #token_tree in [#stream] { ... }]`

#### Goto and label

* `[!label! loop_start]` - defines a label which can be returned to. Effectively, it takes a clones of the remaining token stream after the label in the interpreter.
* `[!goto! loop_start]` - jumps to the last execution of `[!label! loop_start]`. It unrolls the preinterpret stack (dropping all unwritten token streams) until it finds a stackframe in which the interpreter has the defined label, and continues the token stream from there.

```rust,ignore
// Hypothetical future syntax - not yet implemented!
preinterpret::preinterpret!{
    [!set! #i = 0]
    [!label! loop]
    const [!ident! AB #i]: u8 = 0;
    [!increment! #i]
    [!if! [!lte! #i 100] then { [!goto! loop] }]
}
```

### Possible extension: Eager expansion of macros

When [eager expansion of macros returning literals](https://github.com/rust-lang/rust/issues/90765) is stabilized, it would be nice to include a command to do that, which could be used to include code, for example: `[!expand_literal_macros! include!("my-poem.txt")]`.

### Possible extension: Destructuring / Parsing Syntax, and Declarative Macros 2.0

Instead of just a single variable, allow destructuring, for example:
* `[!set! (#x, #y) = ]` or `[!set! Hello(#x, #y) = ]`
* `[!for! (#x, #y) in ...]` or `[!for! Hello(#x, #y) in ...]`

This puts us in the camp of being a simple replacement for a single-use declarative macro:
```rust,ignore
// Hypothetical future syntax - not yet implemented!
preinterpret::preinterpret! {
    [!set! #input =
        (MyTrait for MyType)
        (MyTrait for MyType2)
    ]

    [!for! (#trait for #type) in #input {
        impl #trait for #type
    }]
}

// Could even define simple macros which take a token stream:
preinterpret::preinterpret! {
    [!define! my_macro!(#inner) {
        [!for! (#trait for #type) in #input {
            impl #trait for #type
        }]
    }]
}
```

This is getting awfully close to looking like a declarative macro definition which takes a token stream,
and parses it at destructing time inside the macro. Which actually might be super useful, because:
* It can give much clearer compiler errors compared to declarative macros which fail silently
* It can re-interpret the same tokens in different ways in different parts of the macro

However, a few things stand in our way. Naively, it can only operate on streams of token trees, so it might need lots of brackets for parsing groups.

But instead, we can work around this by implementing simple composable parsers, which can break it up step-by-step:
* `[!parse! <GROUP> = <GROUP>]` is a more general `[!set!]` which takes a `()` wrapped group on the left and a value on the right, and interprets any `#x` on the left as a binding (i.e. place/lvalue) rather than as a value. This will handled commas intelligently, and accept functions as:
    * Maybe `[!group!]` to create a group with no brackets, to avoid parser amibuity in some cases
    * `[!fields! { hello: #a, world?: #b }]` - which can parse `#x` in any order, cope with trailing commas, and permit fields on the RHS not on the LHS
    * `[!subfields! { hello: #a, world?: #b }]` - which can parse fields in any order, cope with trailing commas, and permit fields on the RHS not on the LHS
    * `[!item!]` - which calls syn's parse item on the token
    * More tailored examples, such as `[!generics! { impl: #x, type: #y, where: #z }]` which uses syn to parse the generics, and then uses subfields on the result.
    * Any complex logic (loops, matching), is delayed lazily until execution logic time - making it much more intuitive.
* `[!for! (#a) in (#b) { ... }]` gets the power of parse in the left group (including allowing optional commas between values and copes with trailing commas)
* `[!match! (<INPUT>) => { (<CASE1>) => {<OUTPUT1>}, (<CASE2>) => {<OUTPUT1>}, }]` which captures semantics like the declarative macro inputs, and each case can optionally bind its own variables.

```rust,ignore
// Hypothetical future syntax - not yet implemented!
preinterpret::preinterpret! {
    [!define! multi_impl_super_duper!(
        #type_list,
        ImplOptions [!fields!
            hello: #hello,
            world?: #world (default "Default")
        ]
    ) = {
        [!for! (#type [!generics! { impl: #impl_generics, type: #type_generics }]) in (#type_list) {
            impl<#impl_generics> SuperDuper for #type #type_generics {
                type Hello = #hello;
                type World = #world;
            }
        }]
    }]
}
```

### Possible extension: Explicit parsing feature to enable syn

The heavy `syn` library is (in basic preinterpret) only needed for literal parsing, and error conversion into compile errors.

We could add a parsing feature to speed up compile times a lot for stacks which don't need the parsing functionality.

## License

Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE)
or the [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
