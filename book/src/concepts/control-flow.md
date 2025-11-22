# Control Flow

Preinterpret provides Rust-like control flow for building code dynamically.

## `if` Expressions

### Basic if/else

```rust
#{
    if x > 10 {
        "large"
    } else {
        "small"
    }
}
```

### if/else if/else

```rust
#{
    if x < 0 {
        "negative"
    } else if x == 0 {
        "zero"
    } else {
        "positive"
    }
}
```

### if as Expression

```rust
#{
    let category = if age >= 18 { "adult" } else { "minor" };
}
```

### if in Output

```rust
preinterpret::stream! {
    struct MyStruct {
        #(if with_id { %[id: u32,] })
        name: String,
    }
}
```

## `for` Loops

### Iterate Over Ranges

```rust
#{
    for i in 0..5 {
        // i is 0, then 1, then 2, then 3, then 4
    }

    for i in 0..=5 {
        // i is 0, then 1, then 2, then 3, then 4, then 5
    }
}
```

### Iterate Over Arrays

```rust
#{
    let fields = ["name", "age", "email"];

    for field in fields {
        emit %[
            pub #(%[get_ #field].to_ident())(&self) -> &str {
                &self.#field
            }
        ];
    }
}
```

### Iterate Over Streams

Streams iterate token-by-token:

```rust
#{
    for token in %[a b c] {
        // token is %[a], then %[b], then %[c]
    }
}
```

### Destructuring in for Loops

```rust
#{
    let pairs = [
        %{ name: "Alice", age: 30 },
        %{ name: "Bob", age: 25 },
    ];

    for %{ name, age } in pairs {
        emit %[
            const #(%[NAME_].to_ident_upper_snake()): &str = #name;
        ];
    }
}
```

### for Returns an Array

Loops return arrays of their values:

```rust
#{
    let squares = for i in 0..5 {
        i * i
    };
    // squares = [0, 1, 4, 9, 16]
}
```

Use with `emit` to output without collecting:

```rust
#{
    for i in 0..5 {
        emit %[ const V: u32 = #i; ];
    }
    // Doesn't create an array, just outputs
}
```

## `while` Loops

```rust
#{
    let mut i = 0;
    while i < 10 {
        i += 1;
    }
    // i = 10
}
```

`while` loops don't return values (they return `None`).

## `loop`

Infinite loops with `loop`:

```rust
#{
    let mut count = 0;
    loop {
        count += 1;
        if count >= 10 {
            break;
        }
    }
}
```

## `break` and `continue`

### `break`

Exit a loop early:

```rust
#{
    for i in 0..100 {
        if i == 10 {
            break;
        }
    }
}
```

### `break` with Value

Return a value from a loop:

```rust
#{
    let result = loop {
        if found {
            break value;
        }
    };
}
```

```rust
#{
    let first_even = for i in 0..100 {
        if i % 2 == 0 {
            break i;
        }
    };
}
```

### `continue`

Skip to next iteration:

```rust
#{
    for i in 0..10 {
        if i % 2 == 0 {
            continue;  // Skip even numbers
        }
        // Process odd numbers
    }
}
```

## Labeled Loops

Label loops for nested break/continue:

```rust
#{
    'outer: for i in 0..10 {
        for j in 0..10 {
            if i * j > 50 {
                break 'outer;  // Break outer loop
            }
        }
    }
}
```

```rust
#{
    'outer: loop {
        loop {
            break 'outer;  // Exit outer loop
        }
    }
}
```

### Labeled Blocks

You can also label blocks:

```rust
#{
    let result = 'block: {
        if condition {
            break 'block early_value;
        }
        normal_value
    };
}
```

## Pattern Matching with `attempt`

The `attempt` expression tries multiple alternatives:

```rust
#{
    let input = %raw[impl Clone for User];

    attempt {
        // Try parsing as trait impl
        {
            let %[@(impl) @trait_name=IDENT @(for) @type_name=IDENT] = input;
        } => {
            %{ trait: #trait_name, type: #type_name }
        },

        // Fallback
        {} => {
            %{ trait: None, type: None }
        }
    }
}
```

`attempt` blocks:
- Try each arm in order
- On success, execute the `=>` branch
- On failure, try next arm
- Return the result of successful arm

## emit for Output

Use `emit` inside expression blocks to output tokens:

```rust
preinterpret::stream! {
    #{
        for i in 0..3 {
            emit %[
                const #(%[VALUE_ #i].to_ident()): u32 = #i;
            ];
        }
    }
}
```

Outputs:
```rust
const VALUE_0: u32 = 0;
const VALUE_1: u32 = 1;
const VALUE_2: u32 = 2;
```

Without `emit`, the loop would just return an array.

## Common Patterns

### Generate Multiple Items

```rust
#{
    let types = ["String", "u32", "bool"];

    for ty in types {
        emit %[
            fn #(%[process_ #ty].to_ident_snake())(x: #ty) {
                // ...
            }
        ];
    }
}
```

### Conditional Generation

```rust
#{
    if cfg_feature_enabled {
        emit %[
            #[cfg(feature = "extra")]
            fn extra_method() {}
        ];
    }
}
```

### Accumulate with Loops

```rust
#{
    let mut field_list = %[];

    for (name, ty) in fields {
        field_list += %[#name: #ty,];
    }

    emit %[
        struct MyStruct {
            #field_list
        }
    ];
}
```

### Build Arrays from Ranges

```rust
#{
    let indices = for i in 0..10 { i };
    // indices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

    let const_names = for i in 0..5 {
        %[CONST_ #i].to_ident()
    };
    // const_names = [CONST_0, CONST_1, CONST_2, CONST_3, CONST_4]
}
```

### Find First Match

```rust
#{
    let first_match = for item in items {
        if item.matches(condition) {
            break item;
        }
    };
}
```

Or with `attempt`:

```rust
#{
    let result = attempt {
        {
            // Try something
        } => { success_value },

        {} => { default_value }
    };
}
```

## Next Steps

- Learn about [Expressions](./expressions.md) for the full expression syntax
- Understand [Variables](./variables.md) in control flow
- Explore [Parsing](./parsing.md) with `attempt` blocks
