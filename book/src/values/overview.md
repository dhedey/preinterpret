# Expression Values

Preinterpret provides a rich set of value types that combine Rust-like and JavaScript-like semantics for maximum expressiveness in code generation.

## Value Types

| Type | Description | Example |
|------|-------------|---------|
| **Stream** | Token sequence | `%[struct X]` |
| **Integer** | Whole numbers | `42`, `42u32` |
| **Float** | Decimal numbers | `3.14`, `3.14f64` |
| **Boolean** | True/false | `true`, `false` |
| **Char** | Single character | `'a'`, `'\n'` |
| **String** | Text | `"hello"`, `r"raw"` |
| **Array** | Ordered collection | `[1, 2, 3]` |
| **Object** | Key-value map | `%{ name: "Alice" }` |
| **Range** | Number sequence | `0..10`, `0..=10` |
| **Iterator** | Lazy sequence | From `.into_iter()` |
| **None** | Absence of value | `None` |

## Type System

### Static vs Dynamic

Preinterpret uses **type inference** for most operations:

```rust
#{
    let x = 42;        // Inferred as untyped integer
    let y = 42u32;     // Explicitly u32
    let z = 3.14;      // Inferred as untyped float
}
```

### Type Coercion

Types coerce naturally in expressions:

```rust
#{
    let a = 42;        // Untyped int
    let b = 10u32;     // u32

    let c = a + b;     // Coerces: both become u32
}
```

### Explicit Casting

Use `as` for explicit conversions:

```rust
#{
    let x = 42u32;
    let y = x as i64;      // Convert to i64
    let z = x as int;      // Convert to untyped int
}
```

## Value Properties

### Ownership

Values follow Rust-like ownership:
- Primitives (int, bool, char) are **Copy**
- Complex types (Stream, Array, Object) are **moved**

```rust
#{
    let s = %[hello];
    let s2 = s;          // s is moved
    // Can't use s anymore

    let x = 42;
    let y = x;           // x is copied
    // Can still use x
}
```

### Cloning

Use `.clone()` to duplicate complex values:

```rust
#{
    let s = %[hello];
    let s2 = s.clone();  // Both usable
}
```

### Mutation

Values are immutable by default. Mutation requires explicit `.as_mut()`:

```rust
#{
    let arr = [1, 2, 3];
    arr.as_mut().push(4);  // Now arr = [1, 2, 3, 4]
}
```

## Common Methods

All values support:

```rust
.clone()              // Duplicate the value
.as_mut()             // Get mutable reference
.take()               // Take from mutable ref, leaving None
.debug()              // Debug print (compile error with value)
.to_debug_string()    // Debug string representation
```

## Converting Between Types

### To Stream

Most values can convert to streams:

```rust
%[...]              // Stream literal
42.to_string()      // Integer to string (not stream directly)
[1, 2].to_stream()  // Array to stream (if method exists)
```

### To String

```rust
"hello"                     // String literal
%[hello world].to_string()  // Stream to string: "helloworld"
42.to_debug_string()        // Integer to string: "42"
```

### To Ident

Only streams convert to identifiers:

```rust
%[MyStruct].to_ident()              // Ident: MyStruct
%[get_ field].to_ident()            // Ident: get_field
%[hello_world].to_ident_camel()     // Ident: HelloWorld
```

## Iteration

Many types are **iterable**:

```rust
#{
    // Ranges
    for i in 0..5 { }

    // Arrays
    for item in [1, 2, 3] { }

    // Streams (token by token)
    for token in %[a b c] { }

    // Objects (key-value pairs)
    for %{ key, value } in obj { }
}
```

## Type-Specific Behavior

Each type has unique capabilities:

### Streams
- Token concatenation with `+`
- Conversion to idents/strings/literals
- Parsing with `@` patterns

### Integers/Floats
- Arithmetic operators
- Bitwise operations
- Type suffixes (`u32`, `f64`, etc.)

### Strings
- Concatenation with `+`
- Case conversion methods
- Interpolation in streams

### Arrays
- Indexing: `arr[0]`
- Methods: `.push()`, `.len()`
- Destructuring: `let [a, b] = arr;`

### Objects
- Field access: `obj.field`
- Dynamic access: `obj["field"]`
- Destructuring: `let %{ x, y } = obj;`

### Ranges
- Iteration
- Exclusive (`..`) or inclusive (`..=`)
- Open-ended: `1..`

## Best Practices

1. **Use type inference** - Let preinterpret figure out types
2. **Clone explicitly** - Don't rely on implicit copying
3. **Stream for tokens** - Use streams for code generation
4. **Objects for data** - Use objects for structured data
5. **Arrays for lists** - Use arrays for ordered collections

## Detailed References

Explore each value type in depth:

- [Stream](./stream.md) - Token sequences for code generation
- [Integer](./integers.md) - Whole number types
- [Float](./floats.md) - Decimal number types
- [Boolean](./boolean.md) - True and false values
- [Char](./char.md) - Single characters
- [String](./string.md) - Text values
- [Array](./array.md) - Ordered collections
- [Object](./object.md) - Key-value maps
- [Range](./range.md) - Number sequences
- [Iterator](./iterator.md) - Lazy iteration
- [None](./none.md) - Absence of value
