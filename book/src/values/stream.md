# Stream

Streams are the core value type in preinterpret, representing sequences of Rust tokens.

See the [Streams concept page](../concepts/streams.md) for a comprehensive guide.

## Quick Reference

```rust
// Create streams
%[struct X]                    // Interpreted
%raw[struct X]                 // Raw (no interpretation)

// Concatenate
%[hello] + %[ world]           // %[hello world]

// Convert
%[get_field].to_ident()        // Identifier
%[hello].to_string()           // String: "hello"
%[32 u32].to_literal()         // Literal: 32u32

// Methods
stream.len()                   // Token count
stream.split(%[,])             // Split by separator
stream.is_empty()              // Check if empty
```

## See Also

- [Streams Concept](../concepts/streams.md)
- [Parsing](../concepts/parsing.md)
