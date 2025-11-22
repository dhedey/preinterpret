# Introduction

**Preinterpret** takes the pain out of Rust code generation.

The [preinterpret](https://crates.io/crates/preinterpret) crate provides the `stream!` and `run!` macros which execute a simple but powerful **Rust-inspired interpreted language** designed specifically for code generation.

## What is Preinterpret?

Preinterpret is a compile-time code generation toolkit that combines the best features of existing Rust macro tools:

- **Stream-first design** like [quote](https://crates.io/crates/quote) - work naturally with token streams
- **Powerful concatenation** like [paste](https://crates.io/crates/paste) - but more explicit and predictable
- **Parsing capabilities** like [syn](https://crates.io/crates/syn) - parse and transform Rust syntax

But unlike these tools, Preinterpret provides a **unified programming model**: a Rust-like expression-based language with JavaScript-style flexibility for objects and arrays.

## Why Preinterpret?

Writing Rust macros is hard. Declarative macros have confusing corners, procedural macros require heavy dependencies, and both make debugging difficult.

Preinterpret brings three key improvements:

### 🎯 **Expressivity**

Write clear, procedural code with variables, control flow, and expressions:

```rust
preinterpret::stream! {
    #{
        let fields = ["name", "age", "email"];
        for field in fields {
            let getter = %[get_ #field].to_ident();
            emit %[
                pub fn #getter(&self) -> &str {
                    &self.#field
                }
            ];
        }
    }
}
```

### 📖 **Readability**

Name concepts clearly and reuse them:

```rust
#{
    let impl_generics = %[<T: Clone>];
    let type_generics = %[<T>];
    let my_type = %[MyStruct #type_generics];

    impl #impl_generics MyTrait for #my_type {
        // ...
    }
}
```

### ✨ **Simplicity**

Avoid the confusing corners of declarative macros:
- No [cartesian product errors](https://github.com/rust-lang/rust/issues/96184#issue-1207293401)
- No [callback patterns](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html)
- No [push-down accumulation](https://veykril.github.io/tlborm/decl-macros/patterns/push-down-acc.html)

Just write clear, procedural code that feels like normal Rust.

## Quick Example

Here's a complete example showing preinterpret in action:

```rust
macro_rules! create_struct_and_getters {
    (
        $name:ident { $($field:ident: $ty:ty),* $(,)? }
    ) => {preinterpret::stream! {
        pub struct $name {
            $($field: $ty,)*
        }

        impl $name {
            $(
                pub fn #(%[get_ $field].to_ident())(&self) -> &$ty {
                    &self.$field
                }
            )*
        }
    }}
}

create_struct_and_getters! {
    User { name: String, age: u32 }
}
```

This generates:
- A `User` struct with `name` and `age` fields
- Getter methods `get_name()` and `get_age()`

## Key Features

- **Token streams as native values** - `%[...]` creates a stream you can manipulate
- **Expression-based** - Everything is an expression that returns a value
- **Control flow** - `if`, `for`, `while`, `loop` with `break` and `continue`
- **Flexible data types** - JavaScript-style objects `%{...}` and arrays `[...]`
- **Powerful parsing** - Parse Rust syntax with `@` patterns
- **Error handling** - `attempt { ... }` blocks for trying alternatives
- **Method chaining** - `.to_ident()`, `.to_string()`, `.to_upper_snake_case()`, etc.

## `stream!` vs `run!`

Preinterpret provides two macros:

- **`stream!`** - Outputs a token stream (for use in macro output)
- **`run!`** - Evaluates to a Rust value (for compile-time computation)

```rust
// stream! outputs tokens
let tokens = preinterpret::stream! {
    struct MyStruct;
};

// run! evaluates to a value
const COUNT: usize = preinterpret::run! {
    let mut sum = 0;
    for i in 1..=10 {
        sum += i;
    }
    sum
};
```

## What's Next?

- **[Getting Started](./getting-started/installation.md)** - Install preinterpret and create your first macro
- **[Core Concepts](./concepts/streams.md)** - Learn about streams, variables, and expressions
- **[Expression Values](./values/overview.md)** - Explore all the value types available
- **[Guides](./guides/errors.md)** - Deep dives into error handling, parsing, and debugging

## Comparison with Other Tools

| Feature | Preinterpret | paste | quote + syn |
|---------|--------------|-------|-------------|
| Concat idents | ✅ Explicit `.to_ident()` | ✅ Magic `[<...>]` | ❌ |
| Variables | ✅ `let x = ...` | ❌ | ✅ Rust code |
| Control flow | ✅ Built-in | ❌ | ✅ Rust code |
| Parsing | ✅ Built-in `@` syntax | ❌ | ✅ Full syn |
| Dependencies | ~1.5s compile | ~0.5s compile | ~5s compile |
| Debugging | ✅ Clear errors | ⚠️ Tricky | ⚠️ Proc macro |

Preinterpret sits in a sweet spot: more powerful than `paste`, easier than proc macros, with faster compile times than full `syn`.
