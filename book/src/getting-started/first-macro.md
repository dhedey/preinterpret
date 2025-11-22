# Your First Macro

Let's create a simple but useful macro using preinterpret. We'll build a macro that generates getter methods for struct fields.

## The Goal

We want to write:

```rust
create_getters! {
    User { name: String, age: u32, email: String }
}
```

And have it generate:

```rust
pub struct User {
    name: String,
    age: u32,
    email: String,
}

impl User {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_age(&self) -> &u32 {
        &self.age
    }

    pub fn get_email(&self) -> &String {
        &self.email
    }
}
```

## The Macro

Here's how we implement it with preinterpret:

```rust
macro_rules! create_getters {
    (
        $name:ident { $($field:ident: $ty:ty),* $(,)? }
    ) => {preinterpret::stream! {
        // First, define the struct
        pub struct $name {
            $($field: $ty,)*
        }

        // Then, generate getter methods
        impl $name {
            $(
                pub fn #(%[get_ $field].to_ident())(&self) -> &$ty {
                    &self.$field
                }
            )*
        }
    }}
}
```

## How It Works

Let's break down the preinterpret code:

### 1. The `stream!` Macro

```rust
preinterpret::stream! {
    // ... code generation here
}
```

The `stream!` macro processes preinterpret code and outputs a token stream that becomes part of your Rust code.

### 2. Stream Literals

```rust
%[get_ $field]
```

The `%[...]` syntax creates a **stream literal** - a sequence of tokens. Here, we're concatenating `get_` with the field name.

### 3. Method Calls

```rust
.to_ident()
```

Methods transform values. `.to_ident()` concatenates the stream into a single identifier.

So `%[get_ name].to_ident()` becomes the identifier `get_name`.

### 4. Variable Substitution

```rust
#(%[get_ $field].to_ident())
```

The `#(...)` syntax evaluates the expression and substitutes its result into the output.

## Testing It

Add this to your code:

```rust
create_getters! {
    User { name: String, age: u32 }
}

fn main() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    println!("Name: {}", user.get_name());
    println!("Age: {}", user.get_age());
}
```

## Making It Better

Let's add some preinterpret logic to make the getters smarter:

```rust
macro_rules! smart_getters {
    (
        $name:ident { $($field:ident: $ty:ty),* $(,)? }
    ) => {preinterpret::stream! {
        pub struct $name {
            $($field: $ty,)*
        }

        impl $name {
            #{
                // Use preinterpret expressions for more power!
                for field in [%[$($field),*]].split(%[,]) {
                    let getter_name = %[get_ #field].to_ident();

                    emit %[
                        pub fn #getter_name(&self) -> &$ty {
                            &self.#field
                        }
                    ];
                }
            }
        }
    }}
}
```

Here we use:
- `#{...}` - Expression blocks for procedural logic
- `for` loops - Iterate over fields
- `emit` - Output tokens to the result stream
- Variables - Store and reuse values

## Next Steps

- Check out the [Quick Reference](./quick-reference.md) for a syntax cheat sheet
- Learn more about [Streams](../concepts/streams.md)
- Explore [Expression Values](../values/overview.md)
