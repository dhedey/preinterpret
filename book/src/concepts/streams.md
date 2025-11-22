# Streams

**Streams** are the foundation of preinterpret. A stream is a sequence of Rust tokens that can be manipulated, transformed, and output as code.

## What is a Stream?

In Rust's macro system, everything is made of **tokens**: identifiers (`my_var`), punctuation (`+`, `::`), literals (`42`, `"hello"`), and groups (`(...)`, `[...]`, `{...}`).

A stream is simply a sequence of these tokens. Preinterpret makes streams a first-class value type that you can:
- Store in variables
- Pass to functions/methods
- Transform with methods
- Combine with other streams

## Creating Streams

### Stream Literals: `%[...]`

The most common way to create a stream is with `%[...]`:

```rust
preinterpret::stream! {
    #{
        let my_stream = %[struct MyStruct { x: u32 }];
    }

    // Output the stream
    #my_stream
}
```

This outputs:
```rust
struct MyStruct { x: u32 }
```

### Interpolation in Streams

Inside `%[...]`, you can interpolate variables and expressions:

```rust
#{
    let type_name = %[User];
    let field_count = 3;

    let stream = %[
        struct #type_name {
            field0: u32,
            field1: u32,
            field2: u32,
            count: usize = #field_count,
        }
    ];
}
```

### Raw Streams: `%raw[...]`

Use `%raw[...]` when you want to capture tokens **exactly as written**, without interpretation:

```rust
#{
    let x = "not_substituted";

    // Interpreted stream
    let normal = %[hello #x world];
    // Result: hello "not_substituted" world

    // Raw stream
    let raw = %raw[hello #x world];
    // Result: hello #x world (literally)
}
```

Raw streams are useful when working with macro variables from outer declarative macros:

```rust
macro_rules! my_macro {
    ($dollar_var:ident) => {preinterpret::stream! {
        #{
            // Capture the macro variable without preinterpret trying to substitute it
            let var = %raw[$dollar_var];
        }
        // Now we can use it
        let x = #var;
    }}
}
```

## Manipulating Streams

### Concatenation with `+`

Streams can be concatenated with the `+` operator:

```rust
#{
    let prefix = %[get_];
    let name = %[user];
    let method_name = prefix + name;  // %[get_user]
}
```

### Concatenation with `+=`

```rust
#{
    let stream = %[struct X];
    stream += %[ { value: u32 }];
    // stream is now: struct X { value: u32 }
}
```

## Converting Streams

Streams can be converted to other types using methods:

### To Identifier: `.to_ident()`

Concatenate stream tokens into a single identifier:

```rust
%[hello world].to_ident()         // hello_world (ident)
%[get_ field_name].to_ident()     // get_field_name (ident)
%[My "Struct" Name].to_ident()    // MyStructName (ident)
```

### To String: `.to_string()`

Concatenate stream into a string literal:

```rust
%[hello world].to_string()        // "helloworld"
%[Error: #code].to_string()       // "Error: 42" (if code = 42)
```

### To Literal: `.to_literal()`

Create any literal from tokens:

```rust
%[32 u32].to_literal()            // 32u32 (literal)
%['"' hello '"'].to_literal()     // "hello" (string literal)
```

## Stream Methods

### Length: `.len()`

Get the number of token trees in a stream:

```rust
%[a b c].len()           // 3
%[(x) [y] {z}].len()     // 3 (groups count as one token)
%[].len()                // 0
```

### Check if Empty: `.is_empty()`

```rust
%[].is_empty()           // true
%[x].is_empty()          // false
```

### Split: `.split(separator)`

Split a stream by a separator:

```rust
%[a, b, c].split(%[,])
// Returns: [%[a], %[b], %[c]]

%[x + y + z].split(%[+])
// Returns: [%[x], %[y], %[z]]
```

Options:
```rust
%[a, b, c,].split(%[,], %{ allow_trailing: true })
%[a;; b; c].split(%[;], %{ allow_empty: true })
```

## Streams in Expressions vs Output

### In Expression Blocks: `#{...}`

Inside `#{...}`, streams are values you manipulate:

```rust
#{
    let s = %[hello];
    s += %[ world];
    // s is a value of type stream
}
```

### In Output Context

Outside expression blocks, tokens are output directly:

```rust
preinterpret::stream! {
    struct MyStruct;
    impl MyStruct {
        fn method() {}
    }
}
```

### Emitting Streams: `emit`

Use `emit` inside `#{...}` to output a stream to the result:

```rust
preinterpret::stream! {
    #{
        for i in 0..3 {
            let const_name = %[CONST_ #i].to_ident();
            emit %[
                const #const_name: u32 = #i;
            ];
        }
    }
}
```

Outputs:
```rust
const CONST_0: u32 = 0;
const CONST_1: u32 = 1;
const CONST_2: u32 = 2;
```

## Transparent Groups: `%group[...]`

Sometimes you need to wrap tokens in a group:

```rust
%group[x y z]
```

This creates a "transparent" group - a group with no visible delimiters that keeps tokens together.

Useful when:
- Passing multiple tokens as a single item to iteration
- Preserving token structure

## Advanced: Stream as Iteration Source

Streams can be iterated over (each token tree becomes an item):

```rust
#{
    for token in %[a b c] {
        // token is %[a], then %[b], then %[c]
    }
}
```

## Common Patterns

### Build Dynamic Method Names

```rust
#{
    let field = %[username];
    let getter = %[get_ #field].to_ident();
    let setter = %[set_ #field].to_ident();

    emit %[
        fn #getter(&self) -> &str { &self.#field }
        fn #setter(&mut self, val: String) { self.#field = val; }
    ];
}
```

### Conditional Stream Building

```rust
#{
    let mut stream = %[struct X];

    if with_debug {
        stream += %[ #[derive(Debug)] ];
    }

    stream += %[ { value: u32 } ];

    emit stream;
}
```

### Stream Transformation Pipeline

```rust
#{
    let names = %[hello world foo];
    let constants = for name in names.split(%[ ]) {
        %[MY_ #name].to_ident_upper_snake()
    };
}
```

## Next Steps

- Learn about [Variables](./variables.md) to store and reuse streams
- Understand [Expressions](./expressions.md) for manipulating streams
- Explore [Control Flow](./control-flow.md) for building streams dynamically
