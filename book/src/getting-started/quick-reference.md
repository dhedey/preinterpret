# Quick Reference

A quick cheat sheet for preinterpret syntax.

## Macros

| Macro | Purpose | Example |
|-------|---------|---------|
| `stream!` | Output token stream | `stream! { struct X; }` |
| `run!` | Evaluate to value | `run! { 1 + 1 }` → `2` |

## Stream Literals

| Syntax | Description | Example |
|--------|-------------|---------|
| `%[...]` | Interpreted stream | `%[hello #x]` |
| `%raw[...]` | Raw stream (no interpretation) | `%raw[hello #x]` → `hello #x` |
| `%group[...]` | Wrapped in transparent group | `%group[x y]` |

## Variables

| Syntax | Description | Example |
|--------|-------------|---------|
| `#(let x = ...;)` | Define variable | `#(let name = "Alice";)` |
| `#x` | Substitute variable | `#name` → `Alice` |
| `#(...)` | Evaluate expression | `#(1 + 1)` → `2` |
| `#{...}` | Expression block | `#{let x = 1; x + 1}` |

## Streams → Other Types

| Method | Output | Example |
|--------|--------|---------|
| `.to_ident()` | Identifier | `%[get_ field].to_ident()` → `get_field` |
| `.to_string()` | String | `%[hello world].to_string()` → `"helloworld"` |
| `.to_literal()` | Literal | `%[32 u32].to_literal()` → `32u32` |

## Case Conversion

| Method | Output | Example |
|--------|--------|---------|
| `.to_ident_snake()` | snake_case ident | `HelloWorld` → `hello_world` |
| `.to_ident_camel()` | CamelCase ident | `hello_world` → `HelloWorld` |
| `.to_ident_upper_snake()` | UPPER_SNAKE ident | `hello` → `HELLO` |
| `.to_lower_snake_case()` | "snake_case" | `HelloWorld` → `"hello_world"` |
| `.to_upper_camel_case()` | "CamelCase" | `hello_world` → `"HelloWorld"` |
| `.to_lower_camel_case()` | "camelCase" | `hello_world` → `"helloWorld"` |
| `.to_title_case()` | "Title Case" | `helloWorld` → `"Hello World"` |
| `.to_uppercase()` | "UPPERCASE" | `hello` → `"HELLO"` |
| `.to_lowercase()` | "lowercase" | `HELLO` → `"hello"` |

## Control Flow

```rust
// if / else
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// for loop
for item in [1, 2, 3] {
    // ...
}

// while loop
while x < 10 {
    x += 1;
}

// loop with break
loop {
    if done { break; }
}

// break with value
let result = loop {
    if found { break value; }
};
```

## Data Structures

```rust
// Arrays
let arr = [1, 2, 3];
arr.push(4);
arr.len()

// Objects (like JS)
let obj = %{ name: "Alice", age: 30 };
obj.name      // Access field
obj["age"]    // Dynamic access

// Ranges
0..10        // Exclusive end
0..=10       // Inclusive end
```

## Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+ - * / %` |
| Comparison | `== != < > <= >=` |
| Logical | `&& \|\|` |
| Bitwise | `& \| ^ << >>` |
| Assignment | `= += -= *= /= %=` |

## Common Patterns

### Concatenate and Create Ident

```rust
#(%[prefix_ $field suffix].to_ident())
```

### Loop with Emit

```rust
#{
    for item in items {
        emit %[
            // tokens to output
        ];
    }
}
```

### Destructure Arrays

```rust
let [first, second, ..rest] = array;
```

### Destructure Objects

```rust
let %{ name, age } = obj;
```

## Error Handling

```rust
// Create compile error
%[].error("Something went wrong")

// Assert condition
%[].assert(x > 0, "x must be positive")

// Try alternatives
attempt {
    // Try this first
    risky_operation()
} => {
    // Fallback if it fails
    safe_default()
}
```

## Debugging

```rust
// Print value and cause compile error
x.debug()

// Convert to debug string
x.to_debug_string()
```

## Parsing (Advanced)

```rust
parse %raw[impl Clone for T] {
    @(impl)
    let trait_name = @IDENT;
    @(for)
    let type_name = @IDENT;
}
// Result: %{ trait_name: %[Clone], type_name: %[T] }
```

## Next Steps

- Learn about [Streams](../concepts/streams.md) in depth
- Understand [Variables](../concepts/variables.md)
- Explore [Control Flow](../concepts/control-flow.md)
