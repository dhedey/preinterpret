# Preinterpet - The code generation toolkit

[<img alt="github" src="https://img.shields.io/badge/github-dhedey/preinterpret-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dhedey/preinterpret)
[<img alt="crates.io" src="https://img.shields.io/crates/v/preinterpret.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/preinterpret)
[<img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/preinterpret?style=for-the-badge&logo=rust&logoColor=green&color=green" height="20">](https://crates.io/crates/preinterpret)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-preinterpret-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/preinterpret)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dhedey/preinterpret/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/dhedey/preinterpret/actions?query=branch%3Amain)

<!--
If updating this readme, please ensure that the lib.rs rustdoc is also updated:
* Copy the whole of this document to a new text file
* Replace `\n` with `\n//! ` prefix to each line
* Also fix the first line
* Paste into `lib.rs`
* Run ./style-fix.sh
-->

Preinterpret takes the pain out of Rust code generation, providing a new paradigm to replace the clunkiness of declarative macros [[1](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html), [2](https://github.com/rust-lang/rust/issues/96184#issue-1207293401), [3](https://veykril.github.io/tlborm/decl-macros/minutiae/metavar-and-expansion.html), [4](https://veykril.github.io/tlborm/decl-macros/patterns/push-down-acc.html)]. At its heart it is a bespoke Rust-like interpreted language, built from the ground up for code generation use cases.

The [preinterpret](https://crates.io/crates/preinterpret) crate provides:
* The `stream!` macro, which starts in token stream output mode
* The `run!` macro, which starts in interpreter mode, and can `emit` or return a token stream

To install, add the following to your `Cargo.toml`:

```toml
[dependencies]
preinterpret = "0.2"
```

## Use cases

### Simple code generation

Sometimes you just need to generate lots of similar code, and preinterpet can be used directly:

```rust
preinterpret::run!{
    for N in 0..=12 {
        let type_params = %[];
        for a in ('A'..'Z').into_iter().take(N) {
            type_params += a.to_ident() + %[,]
        }
        emit %[
            impl<#type_params> TupleMeasure for (#type_params) {
                fn len(&self) -> usize {
                    #N
                }
            }
        ]
    }
}
```

### Inside procedural macros

It can be used to simplify code generation inside procedural macro definitions:
* It replaces [paste](https://crates.io/crates/paste) to allow concatenated creation of idents
* It brings various features previously reserved for procedural macros: [quote](https://crates.io/crates/quote)-like token-stream substitution and [syn](https://crates.io/crates/syn)-based functionality for operating on tokens and literals.

Notably, using variables to name token stream sections for clarity and reuse can make macro code a lot easier to read.

```rust
macro_rules! create_my_type {
    (
        $(#[$attributes:meta])*
        $vis:vis struct $type_name:ident {
            $($field_name:ident: $inner_type:ident),* $(,)?
        }
    ) => {preinterpret::stream! {
        #{
            let type_name = %[My $type_name].to_ident();
        }
        
        $(#[$attributes])*
        $vis struct #type_name {
            $($field_name: $inner_type,)*
        }

        impl #type_name {
            $(
                fn #(%[my_ $inner_type].to_ident_snake())(&self) -> &$inner_type {
                    &self.$field_name
                }
            )*
        }
    }}
}
create_my_type! {
    struct Struct {
        field0: String,
        field1: u64,
    }
}
assert_eq!(MyStruct { field0: "Hello".into(), field1: 21 }.my_string(), "Hello")
```

### As a replacement for procedural macros

Coming soon...


## User Guide

Preinterpret works with its own very simple language, with two pieces of syntax:

* **Commands**: `[!command_name! input token stream...]` take an input token stream and output a token stream. There are a number of commands which cover a toolkit of useful functions.
* **Variables**: `#(let var_name = %[token stream...];)` defines a variable, and `#var_name` substitutes the variable into another command or the output.

Commands can be nested intuitively. In general, the input of commands are first interpreted before the command itself executes.

### Declarative macro example

The following artificial example demonstrates how `preinterpret` can be integrate into declarative macros, and covers use of variables, idents and case conversion:

```rust
macro_rules! create_my_type {
    (
        $(#[$attributes:meta])*
        $vis:vis struct $type_name:ident {
            $($field_name:ident: $inner_type:ident),* $(,)?
        }
    ) => {preinterpret::stream! {
        #{
            let type_name = %[My $type_name].to_ident();
        }
        
        $(#[$attributes])*
        $vis struct #type_name {
            $($field_name: $inner_type,)*
        }

        impl #type_name {
            $(
                fn #(%[my_ $inner_type].to_ident_snake())(&self) -> &$inner_type {
                    &self.$field_name
                }
            )*
        }
    }}
}
create_my_type! {
    struct Struct {
        field0: String,
        field1: u64,
    }
}
assert_eq!(MyStruct { field0: "Hello".into(), field1: 21 }.my_string(), "Hello")
```

### Quick background on token streams and macros

To properly understand how preinterpret works, we need to take a very brief detour into the language of macros.

In Rust, the input and output to a macro is a [`TokenStream`](https://doc.rust-lang.org/proc_macro/enum.TokenStream.html). A `TokenStream` is simply an iterator of [`TokenTree`](https://doc.rust-lang.org/proc_macro/enum.TokenTree.html)s at a particular nesting level. A token tree is one of four things:

* A [`Group`](https://doc.rust-lang.org/proc_macro/struct.Group.html) - typically `(..)`, `[..]` or `{..}`. It consists of a matched pair of [`Delimiter`s](https://doc.rust-lang.org/proc_macro/enum.Delimiter.html) and an internal token stream. There is also a transparent delimiter, used to group the result of token stream substitutions (although [confusingly](https://github.com/rust-lang/rust/issues/67062) a little broken in rustc).
* An [`Ident`](https://doc.rust-lang.org/proc_macro/struct.Ident.html) - An unquoted string, used to identitied something named. Think `MyStruct`, or `do_work` or `my_module`. Note that keywords such as `struct` or `async` and the values `true` and `false` are classified as idents at this abstraction level.
* A [`Punct`](https://doc.rust-lang.org/proc_macro/struct.Punct.html) - A single piece of punctuation. Think `!` or `:`.
* A [`Literal`](https://doc.rust-lang.org/proc_macro/struct.Literal.html) - This includes string literals `"my string"`, char literals `'x'` and numeric literals `23` / `51u64`.

When you return output from a macro, you are outputting back a token stream, which the compiler will interpret.

Preinterpret commands take token streams as input, and return token streams as output.

### Migration from paste

If migrating from [paste](https://crates.io/crates/paste), the main difference is that you need to specify _what kind of concatenated thing you want to create_. Paste tried to work this out magically from context, but sometimes got it wrong.

In other words, you typically want to replace `[< ... >]` with `[!ident! ...]`, and sometimes `[!string! ...]` or `[!literal! ...]`:
* To create type and function names, use `[!ident! My #preinterpret_type_name $macro_type_name]`, `[!ident_camel! ...]`, `[!ident_snake! ...]` or `[!ident_upper_snake! ...]`
* For doc macros or concatenated strings, use `[!string! "My type is: " #type_name]`
* If you're creating literals of some kind by concatenating parts together, use `[!literal! 32 u32]`

For example:

```rust
preinterpret::stream! {
    #{
        let type_name = %[HelloWorld];
    }

    struct #type_name;

    #[doc = #(%["This type is called [`" #type_name "`]"].to_string())]
    impl #type_name {
        fn #(%[say_ #type_name].to_ident_snake())() -> &'static str {
            #(%["It's time to say: " #(type_name.to_string().to_title_case()) "!"].to_string())
        }
    }
}
assert_eq!(HelloWorld::say_hello_world(), "It's time to say: Hello World!")
```

## Command List

### Special commands

* `#(let foo = %[Hello];)` followed by `#(let foo = %[#bar(World)];)` sets the variable `#foo` to the token stream `Hello` and `#bar` to the token stream `Hello(World)`, and outputs no tokens. Using `#foo` or `#bar` later on will output the current value in the corresponding variable.
* `%raw[abc #abc %[test]]` outputs its contents as-is, without any interpretation, giving the token stream `abc #abc %[test]`.
* `let _ = %raw[$foo]` ignores all content inside `[...]` and outputs no tokens. It is useful to make a declarative macro loop over a meta-variable without outputting it into the resulting stream.

### Concatenate and convert commands

Each of these commands functions in three steps:
* Apply the interpreter to the token stream, which recursively executes preinterpret commands.
* Convert each token of the resulting stream into a string, and concatenate these together. String and char literals are unquoted, and this process recurses into groups.
* Apply some command-specific conversion.

The following commands output idents:

* `%[X Y "Z"].to_ident()` outputs the ident `XYZ`
* `%[my hello_world].to_ident_camel()` outputs `MyHelloWorld`
* `%[my_ HelloWorld].to_ident_snake()` outputs `my_hello_world`
* `%[my_ const Name].to_ident_upper_snake()` outputs `MY_CONST_NAME`

The following commands output any kind of literal, for example:

* `%[31 u 32].to_literal()` outputs the integer literal `31u32`
* `%['"' hello '"'].to_literal()` outputs the string literal `"hello"`

The following commands output strings, without dropping non-alphanumeric characters:

* `%[X Y " " Z (Hello World)].to_string()` outputs `"XY Z(HelloWorld)"`
* `"foo_bar".to_uppercase()` outputs `"FOO_BAR"`
* `"FooBar".to_lowercase()` outputs `"foobar"`
* `"fooBar".capitalize()"` outputs `"FooBar"`
* `"FooBar".decapitalize()` outputs `"fooBar"`

The following commands output strings, whilst also dropping non-alphanumeric characters:

* `"FooBar".to_lower_snake_case()` outputs `"foo_bar"`
* `"FooBar".to_upper_snake_case()"` outputs `"FOO_BAR"`
* `"foo_bar".to_upper_camel_case()"` outputs `"FooBar"`
* `"foo_bar".to_lower_camel_case()"` outputs `"fooBar"`
* `"fooBar".to_kebab_case()"` outputs `"foo-bar"`
* `"fooBar".to_title_case()"` outputs `"Foo Bar"`
* `"fooBar".insert_spaces()"` outputs `"foo Bar"`

> [!NOTE]
>
> These string conversion methods are designed to work intuitively across a wide class of input strings, by creating word boundaries when going from non-alphanumeric to alphanumeric, lowercase to uppercase, or uppercase to uppercase if the next character is lowercase.
>
> The case-conversion commands which drop non-alphanumeric characters can potentially break up grapheme clusters and can cause unintuitive behaviour
> when used with complex unicode strings.  
>
> A wide ranging set of tests covering behaviour are in [tests/string.rs](https://www.github.com/dhedey/preinterpret/blob/main/tests/string.rs).

## Motivation

Compared to writing just declarative macros, preinterpret provides:

* **Heightened [readability](#readability)** - quote-like variable definition and substitution make it easier to work with code generation code.
* **Heightened [expressivity](#expressivity)** - a toolkit of simple commands reduce boilerplate, and mitigate the need to build custom procedural macros in some cases.
* **Heightened [simplicity](#simplicity)** - helping developers avoid the confusing corners [[1](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html), [2](https://github.com/rust-lang/rust/issues/96184#issue-1207293401), [3](https://veykril.github.io/tlborm/decl-macros/minutiae/metavar-and-expansion.html), [4](https://veykril.github.io/tlborm/decl-macros/patterns/push-down-acc.html)] of declarative macro land.

### Readability

The preinterpret syntax is intended to be immediately intuitive even for people not familiar with the crate. It enables developers to make more readable macros:

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
    } => {preinterpret::stream!{
        #{
            let impl_generics = %[$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?];
            let type_generics = %[$(< $( $lt ),+ >)?];
            let my_type = %[$type_name #type_generics];
        }

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

Preinterpret provides a suite of simple, composable commands to convert token streams, literals and idents. The full list is documented in the [Command List](#command-list) section.

For example:

```rust
macro_rules! create_struct_and_getters {
    (
        $name:ident { $($field:ident),* $(,)? }
    ) => {preinterpret::stream!{
        // Define a struct with the given fields
        pub struct $name {
            $(
                $field: String,
            )*
        }

        impl $name {
            $(
                // Define get_X for each field X
                pub fn #(%[get_ $field].to_ident())(&self) -> &str {
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

### Simplicity

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
    } => {preinterpret::stream!{
        #[xyz(as_type = #(%[$my_inner_type].to_string()))]
        $vis struct $my_type($my_inner_type);
    }}
}
```

## Roadmap

A much more fully-featured rust-inspired compile-time language and interpeter is coming in v1.0, currently on the [develop](https://github.com/dhedey/preinterpret/pull/3) branch, offering:
* Values
* Expressions
* Built in functions/methods
* Parsing

### Possible extension: Eager expansion of macros

When [eager expansion of macros returning literals](https://github.com/rust-lang/rust/issues/90765) is stabilized, it would be nice to include a command to do that, which could be used to include code, for example: `[!expand_literal_macros! include!("my-poem.txt")]`.

## License

Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE)
or the [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
