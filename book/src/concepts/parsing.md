# Parsing

Preinterpret provides powerful parsing capabilities for working with Rust token streams. You can parse streams using `@` patterns in destructuring contexts.

## Basic Parsing

### Parse in let Bindings

Use `%[@... ]` pattern to parse a stream:

```rust
#{
    let input = %raw[impl Clone for User];

    let %[@(impl) @trait_name=IDENT @(for) @type_name=IDENT] = input;

    // trait_name = %[Clone]
    // type_name = %[User]
}
```

## Parsing Patterns

### Exact Tokens: `@(...)`

Match exact tokens:

```rust
let %[@(impl) @(Clone) @(for) @(User)] = input;
```

### Named Parsers

#### `@IDENT` - Parse Identifier

```rust
let %[@type_name=IDENT] = %raw[User];
// type_name = %[User]
```

#### `@LITERAL` - Parse Literal

```rust
let %[@value=LITERAL] = %raw[42];
// value = %[42]
```

#### `@PUNCT` - Parse Punctuation

```rust
let %[@op=PUNCT] = %raw[+];
// op = %[+]
```

#### `@REST` - Parse Remaining

```rust
let %[@(struct) @name=IDENT @rest=REST] = %raw[struct User { x: u32 }];
// name = %[User]
// rest = %[{ x: u32 }]
```

### Group Parsing

#### `@[GROUP ...]` - Parse Group Content

```rust
let %[@[GROUP @content=REST]] = %raw[(hello world)];
// content = %[hello world] (without parentheses)
```

#### Match Any Group

```rust
let %[@body=GROUP] = %raw[{ x: u32 }];
// body = %[{ x: u32 }]
```

## Advanced Patterns

### `@[UNTIL separator]` - Parse Until Token

```rust
let %[@prefix=UNTIL(::) @(::) @suffix=REST] = %raw[std::vec::Vec];
// prefix = %[std]
// suffix = %[vec::Vec]
```

### `@[EXACT(...)]` - Exact Match

```rust
let %[@[EXACT(%[impl Clone])]] = input;
// Ensures input starts with exactly "impl Clone"
```

## Using Parsed Values

### Build New Code

```rust
#{
    let input = %raw[impl Display for User];
    let %[@(impl) @trait_name=IDENT @(for) @type_name=IDENT] = input;

    emit %[
        // Generate Debug impl too
        impl Debug for #type_name {
            // ...
        }
    ];
}
```

### Transform Patterns

```rust
#{
    let %[@(fn) @name=IDENT @args=GROUP @body=GROUP] = method_def;

    // Create wrapper
    emit %[
        fn #name #args {
            log_call(stringify!(#name));
            #body
        }
    ];
}
```

## Error Handling with `attempt`

Use `attempt` to try different parse patterns:

```rust
#{
    let result = attempt {
        // Try as trait impl
        {
            let %[@(impl) @trait_name=IDENT @(for) @type_name=IDENT] = input;
        } => {
            %{ kind: "trait_impl", trait: #trait_name, type: #type_name }
        },

        // Try as struct
        {
            let %[@(struct) @name=IDENT @body=REST] = input;
        } => {
            %{ kind: "struct", name: #name }
        },

        // Fallback
        {} => {
            %{ kind: "unknown" }
        }
    };
}
```

## Common Parsing Patterns

### Parse Function Signature

```rust
let %[@(fn) @name=IDENT @args=GROUP @ret=REST] = input;
```

### Parse Type with Generics

```rust
let %[@base=IDENT @generics=UNTIL(::)] = %raw[Vec<T>::new];
```

### Parse Field Definition

```rust
let %[@name=IDENT @(:) @ty=IDENT] = %raw[field: u32];
```

### Split on Separators

For lists, use `.split()` instead of parsing:

```rust
let items = %[a, b, c].split(%[,]);
// items = [%[a], %[b], %[c]]
```

## Working with Groups

### Extract Group Contents

```rust
let %[@[GROUP @inner=REST]] = %raw[(x, y, z)];
// inner = %[x, y, z]
```

### Parse Nested Structures

```rust
let %[
    @(struct) @name=IDENT
    @[GROUP
        @(#) @[GROUP @attr=REST]
        @rest=REST
    ]
] = %raw[struct User { #[derive(Debug)] x: u32 }];
```

## Combining with Control Flow

### Parse Multiple Items

```rust
#{
    let items = input.split(%[,]);

    for item in items {
        let %[@name=IDENT @(=) @value=IDENT] = item;

        emit %[
            const #name: u32 = #value;
        ];
    }
}
```

### Conditional Parsing

```rust
#{
    attempt {
        // Check if async
        {
            let %[@(async) @rest=REST] = input;
            emit %[/* async version */];
        },

        // Normal version
        {} => {
            emit %[/* sync version */];
        }
    }
}
```

## Tips

1. **Use `%raw[...]`** to capture macro variables without interpretation
2. **Name your captures** with `@name=PATTERN` for clarity
3. **Try `attempt` first** for ambiguous syntax
4. **Split before parsing** when dealing with lists
5. **Use `.to_debug_string()`** to inspect parsed values

## Next Steps

- Learn about [Error Handling](../guides/errors.md) with parsing
- Explore [Advanced Parsing](../guides/parsing.md) techniques
- Understand [Streams](./streams.md) that you're parsing
