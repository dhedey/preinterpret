# Expressions

Preinterpret is an **expression-based language** - almost everything evaluates to a value. This is similar to Rust, where blocks, if-statements, and loops can all produce values.

## Expression Blocks: `#{...}`

Use `#{...}` to create an expression block:

```rust
preinterpret::stream! {
    const VALUE: u32 = #{
        let x = 10;
        let y = 20;
        x + y  // Final expression is the result
    };  // VALUE = 30
}
```

Inside `#{...}`, you can:
- Declare variables with `let`
- Use control flow (`if`, `for`, `while`, `loop`)
- Perform calculations
- Call methods
- Return a value (the last expression without `;`)

## Inline Expressions: `#(...)`

For simple expressions, use `#(...)`:

```rust
#(1 + 1)                    // 2
#("hello".to_uppercase())   // "HELLO"
#(%[get_ field].to_ident()) // get_field
```

This is shorthand for a single-expression block.

## Literals

### Numeric Literals

```rust
42              // Untyped integer
42u32           // Typed integer
42_000          // With underscores
3.14            // Untyped float
3.14f64         // Typed float
```

### String and Char Literals

```rust
"hello world"   // String
'a'             // Char
r"raw string"   // Raw string
```

### Boolean Literals

```rust
true
false
```

### None

```rust
None            // Represents absence of value
```

## Collections

### Arrays

```rust
let arr = [1, 2, 3, 4, 5];
let mixed = [%[x], %[y], %[z]];  // Array of streams
```

### Objects

Objects are like JavaScript objects - flexible key-value stores:

```rust
let user = %{
    name: "Alice",
    age: 30,
    ["computed_key"]: "value",
};

user.name       // "Alice"
user["age"]     // 30
```

### Ranges

```rust
0..10           // Exclusive end: 0, 1, 2, ..., 9
0..=10          // Inclusive end: 0, 1, 2, ..., 10
1..             // Open-ended: 1, 2, 3, ...
```

## Operators

### Arithmetic

```rust
1 + 2           // Addition
5 - 3           // Subtraction
4 * 3           // Multiplication
10 / 3          // Division
10 % 3          // Modulo
```

Works on integers and floats.

### Comparison

```rust
1 == 1          // Equal
1 != 2          // Not equal
1 < 2           // Less than
2 > 1           // Greater than
1 <= 1          // Less than or equal
2 >= 2          // Greater than or equal
```

Returns a boolean.

### Logical

```rust
true && false   // AND (short-circuits)
true || false   // OR (short-circuits)
!true           // NOT
```

### Bitwise

```rust
5 & 3           // AND: 1
5 | 3           // OR: 7
5 ^ 3           // XOR: 6
5 << 1          // Left shift: 10
5 >> 1          // Right shift: 2
```

### String Concatenation

```rust
"hello" + " " + "world"    // "hello world"
```

### Stream Concatenation

```rust
%[hello] + %[ world]       // %[hello world]
```

## Type Casting

Use `as` to convert between types:

```rust
// To specific integer types
42 as u32
42 as i64

// To generic int/float
42u32 as int
3.14f64 as float

// Stream to stream (flattening)
%[(x) [y] {z}] as stream   // %[x y z]
```

## Method Calls

Methods can be chained:

```rust
%[hello world]
    .to_ident()
    .to_debug_string()
```

Common methods on all values:

```rust
x.clone()           // Create a copy
x.as_mut()          // Get mutable reference
x.take()            // Take value, leaving None
x.debug()           // Debug print (causes compile error)
x.to_debug_string() // Get debug representation
```

## Field Access

### Object Fields

```rust
let obj = %{ name: "Alice", age: 30 };

obj.name            // "Alice"
obj["name"]         // "Alice" (dynamic access)
```

### Array Indexing

```rust
let arr = [10, 20, 30];

arr[0]              // 10
arr[1]              // 20
```

## Blocks as Expressions

Blocks can produce values:

```rust
let result = {
    let x = 10;
    let y = 20;
    x + y       // Returns 30
};
```

Blocks with semicolon return `None`:

```rust
let nothing = {
    let x = 10;
    x + 20;     // Semicolon! Returns None
};
```

## Common Expression Patterns

### Conditional Values

```rust
let max = if a > b { a } else { b };
```

### Computed Names

```rust
let method = #(%[get_ #field_name].to_ident());
```

### Transform and Accumulate

```rust
let result = {
    let mut sum = 0;
    for i in 0..10 {
        sum += i;
    }
    sum
};
```

### Build Streams Procedurally

```rust
let code = {
    let mut s = %[];

    for i in 0..5 {
        s += %[
            const #(%[VALUE_ #i].to_ident()): u32 = #i;
        ];
    }

    s
};
```

## Expression vs Statement Context

Understanding context is important:

```rust
preinterpret::stream! {
    // This is STATEMENT context - tokens output directly
    struct MyStruct;

    #{
        // This is EXPRESSION context - compute values
        let x = 1 + 1;
    }

    // Back to statement context
    impl MyStruct {
        // ...
    }
}
```

Use `emit` to output from expression context:

```rust
#{
    for i in 0..3 {
        emit %[
            const VALUE: u32 = #i;
        ];
    }
}
```

## Next Steps

- Learn about [Control Flow](./control-flow.md) for if/for/while/loop
- Understand [Variables](./variables.md) in expressions
- Explore [Expression Values](../values/overview.md) for all value types
