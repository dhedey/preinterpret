# Error Handling

Preinterpret provides several mechanisms for handling errors during code generation.

## Creating Errors

### Basic Error

Use `.error()` to create a compile error:

```rust
#{
    if invalid_condition {
        %[].error("Something went wrong!");
    }
}
```

This produces a compiler error at the macro call site.

### Error with Span

Provide a token stream to error at a specific location:

```rust
#{
    let token = %raw[$user_input];
    token.error("Invalid token provided");
}
```

The error will point to the span of `$user_input`.

### Using `%[_]` for Current Location

```rust
%[_].error("Error at this line");
```

## Assertions

### Basic Assert

```rust
#{
    %[].assert(count > 0, "Count must be positive");
}
```

If the condition is false, creates a compile error.

### Assert with Span

```rust
#{
    let input = %raw[$token];
    input.assert(input.len() > 0, "Input cannot be empty");
}
```

## The `attempt` Expression

`attempt` lets you try multiple alternatives gracefully:

```rust
#{
    let result = attempt {
        // Try first option
        {
            let parsed = risky_parse(input);
            parsed  // Success!
        } => {
            processed(parsed)
        },

        // Try second option
        {
            let alternative = other_parse(input);
            alternative
        } => {
            processed(alternative)
        },

        // Fallback
        {} => {
            default_value()
        }
    };
}
```

### How `attempt` Works

1. Each arm is tried in order
2. If the body before `=>` succeeds, execute the handler after `=>`
3. If it fails (e.g., parse error, assertion), try next arm
4. If no arms succeed, the last error is propagated

### Pattern Matching with attempt

```rust
#{
    let info = attempt {
        // Try parsing as trait impl
        {
            let %[@(impl) @trait_name=IDENT @(for) @type_name=IDENT] = input;
        } => {
            %{ kind: "trait_impl", trait: #trait_name, type: #type_name }
        },

        // Try parsing as struct
        {
            let %[@(struct) @name=IDENT @rest=REST] = input;
        } => {
            %{ kind: "struct", name: #name }
        },

        // Couldn't parse
        {} => {
            %[].error("Unrecognized syntax")
        }
    };
}
```

### Optional Values with attempt

```rust
#{
    let maybe_value = attempt {
        {
            let %[@(value) @(=) @val=IDENT] = input;
        } => { val },

        {} => { None }  // Return None if not found
    };
}
```

## Error Messages

### Clear Error Messages

Write clear, actionable error messages:

```rust
// Bad
%[].error("Error");

// Good
%[].error("Expected field name to be an identifier, found number");

// Better
field_token.error(
    "Expected field name to be an identifier, found number. \
     Field names must be valid Rust identifiers."
);
```

### Include Context

```rust
#{
    for field in fields {
        field.assert(
            field.len() > 0,
            format!("Field '{}' cannot be empty", field.to_debug_string())
        );
    }
}
```

## Debugging Errors

### `.debug()` Method

Use `.debug()` to inspect values:

```rust
#{
    let x = complex_computation();
    x.debug();  // Causes compile error showing x's value
}
```

### `.to_debug_string()`

Get debug representation as a string:

```rust
#{
    let repr = value.to_debug_string();
    %[].error("Unexpected value: " + repr);
}
```

## Common Error Patterns

### Validate Input

```rust
macro_rules! my_macro {
    ($($field:ident),*) => {preinterpret::stream! {
        #{
            let fields = [%raw[$($field),*]];
            %[].assert(fields.len() > 0, "At least one field is required");

            for field in fields {
                field.assert(
                    field.len() == 1,
                    "Each field must be a single identifier"
                );
            }
        }
        // ... generate code
    }}
}
```

### Provide Helpful Fallbacks

```rust
#{
    let config = attempt {
        { parse_config(input) } => { config },
        {} => {
            %[_].error(
                "Failed to parse configuration. \
                 Expected format: key = value, key = value"
            )
        }
    };
}
```

### Guard Against Edge Cases

```rust
#{
    if items.is_empty() {
        %[].error("Cannot generate code for empty list");
    }

    if items.len() > 100 {
        %[].error("Too many items (limit: 100)");
    }
}
```

## Error Recovery

Use `attempt` for graceful degradation:

```rust
#{
    let features = for item in items {
        attempt {
            { parse_feature(item) } => { feature },
            {} => { None }  // Skip invalid items
        }
    };

    // Filter out None values
    let valid_features = features.filter(|f| f != None);
}
```

## Next Steps

- Learn about [Spans](./spans.md) for better error locations
- Explore [Debugging](./debugging.md) techniques
- See [Parsing](../concepts/parsing.md) with error handling
