# Variables

Variables in preinterpret work similarly to Rust, with some conveniences for code generation.

## Declaring Variables

Use `let` to declare a variable:

```rust
#{
    let x = 42;
    let name = "Alice";
    let tokens = %[struct MyStruct];
}
```

Variables are:
- **Block-scoped** - only visible within their enclosing `{...}` block
- **Immutable by default** - cannot be reassigned unless you use mutation
- **Type-inferred** - the type is determined from the value

## Variable Substitution

Use `#variable_name` to substitute a variable's value:

```rust
#{
    let greeting = "Hello";
    let name = "World";

    let message = #greeting + ", " + #name + "!";
    // message = "Hello, World!"
}
```

In stream literals:

```rust
#{
    let type_name = %[User];

    emit %[
        struct #type_name {
            id: u32,
        }
    ];
}
```

## Expression Substitution

Use `#(...)` to evaluate an expression and substitute the result:

```rust
preinterpret::stream! {
    const VALUE: u32 = #(10 * 5 + 2);  // 52
}
```

With methods:

```rust
preinterpret::stream! {
    #{
        let field = "username";
    }

    fn #(%[get_ #field].to_ident())() -> &str {
        // fn get_username() -> &str
        &self.username
    }
}
```

## Assignment

### Simple Assignment

```rust
#{
    let x = 10;
    x = 20;  // Reassign
}
```

### Compound Assignment

```rust
#{
    let mut sum = 0;
    sum += 10;  // sum = 10
    sum += 5;   // sum = 15

    let mut s = %[hello];
    s += %[ world];  // s = %[hello world]
}
```

Supported operators: `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`

## Destructuring

### Array Destructuring

```rust
#{
    let [first, second, third] = [1, 2, 3];
    // first = 1, second = 2, third = 3

    let [head, ..tail] = [1, 2, 3, 4, 5];
    // head = 1, tail = [2, 3, 4, 5]

    let [a, b, ..] = [1, 2, 3, 4];
    // a = 1, b = 2, rest ignored
}
```

### Object Destructuring

```rust
#{
    let user = %{ name: "Alice", age: 30 };

    let %{ name, age } = user;
    // name = "Alice", age = 30

    let %{ name: user_name, age } = user;
    // user_name = "Alice", age = 30

    let %{ name, .. } = user;
    // name = "Alice", age ignored
}
```

### Stream Destructuring with Parsing

```rust
#{
    let input = %raw[impl Clone for User];

    let %[@(impl) @trait_name=IDENT @(for) @type_name=IDENT] = input;
    // trait_name = %[Clone], type_name = %[User]
}
```

## Ignore Bindings: `_`

Use `_` to ignore a value:

```rust
#{
    let _ = expensive_computation();  // Run but ignore result

    let [first, _, third] = [1, 2, 3];
    // first = 1, third = 3, second element ignored
}
```

This is useful in declarative macros to iterate without outputting:

```rust
macro_rules! count {
    ($($item:ident),*) => {preinterpret::stream! {
        #{
            let count = 0;
            $(
                let _ = %raw[$item];  // Iterate but don't use
                count += 1;
            )*
        }
        #count
    }}
}
```

## Scoping

Variables follow block scoping rules:

```rust
#{
    let x = 10;

    {
        let x = 20;  // Shadows outer x
        // x = 20 here
    }

    // x = 10 here

    if condition {
        let y = 30;  // Only visible in this block
    }
    // y not visible here
}
```

## Variable Lifecycle

### Ownership and Moving

By default, using a variable **moves** it (for streams, objects, and arrays):

```rust
#{
    let s = %[hello];
    let s2 = s;  // s is moved
    // Can't use s anymore
}
```

### Cloning

Use `.clone()` to create a copy:

```rust
#{
    let s = %[hello];
    let s2 = s.clone();  // Both s and s2 are usable
}
```

### Mutable References

Sometimes you need `as_mut()`:

```rust
#{
    let x = [1, 2, 3];
    let y = x;  // Move

    let a = [1, 2, 3];
    a.as_mut().push(4);  // Explicit mutation
}
```

## Working with Declarative Macro Variables

Variables from declarative macros (`$var`) are used directly in streams:

```rust
macro_rules! my_macro {
    ($ty:ty) => {preinterpret::stream! {
        #{
            // Capture the macro variable
            let rust_type = %raw[$ty];

            // Now use it in preinterpret
            emit %[
                type MyType = #rust_type;
            ];
        }
    }}
}
```

### Iterating Macro Repetitions

```rust
macro_rules! gen_fields {
    ($($field:ident),*) => {preinterpret::stream! {
        #{
            $(
                let field_name = %raw[$field];
                emit %[
                    pub #field_name: String,
                ];
            )*
        }
    }}
}
```

## Common Patterns

### Building Names Dynamically

```rust
#{
    let base = "field";
    let index = 0;

    let getter = %[get_ #base _ #index].to_ident();
    // getter = get_field_0
}
```

### Accumulating Results

```rust
#{
    let mut result = %[];

    for item in items {
        result += %[#item,];
    }

    emit %[
        const ITEMS: &[&str] = &[#result];
    ];
}
```

### Conditional Definitions

```rust
#{
    let mut attributes = %[];

    if with_debug {
        attributes += %[#[derive(Debug)]];
    }

    if with_clone {
        attributes += %[#[derive(Clone)]];
    }

    emit %[
        #attributes
        struct MyStruct;
    ];
}
```

### Stream Building

```rust
#{
    let type_name = %[User];
    let fields = [%[name: String], %[age: u32]];

    let field_list = fields.intersperse(%[,]);

    emit %[
        struct #type_name {
            #field_list
        }
    ];
}
```

## Next Steps

- Learn about [Expressions](./expressions.md) for computing values
- Understand [Control Flow](./control-flow.md) for dynamic generation
- See how to use [Streams](./streams.md) as variable values
