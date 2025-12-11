# Plan: Remove `Span::call_site()` Fallbacks

This document tracks the fixes needed to remove all `Span::call_site()` occurrences introduced in the SpanRanges refactoring PR.

**Principle**: Spans live on the *outside* via `Spanned<T>` wrappers, not embedded in types.

**Current Status**: IN PROGRESS - Major API changes made, ~103 compilation errors remaining.

---

## Summary of Changes Made

### Core API Changes

1. **`IsArgument::from_argument`** - Now takes `span: SpanRange` parameter
2. **`ResolveAs::resolve_as`** - Now takes `span: SpanRange` parameter
3. **`ResolvableOwned::resolve_value`, `resolve_owned`** - Now take `span: SpanRange` parameter
4. **`ResolvableShared::resolve_shared`** - Now takes `span: SpanRange` parameter
5. **`ResolvableMutable::resolve_mutable`, `resolve_assignee`** - Now take `span: SpanRange` parameter
6. **`HandleBinaryOperation::paired_operation_no_overflow`, `paired_comparison`** - Now take `span: SpanRange` parameter
7. **`Expression::span_range()`** - Added method to compute span from root node
8. **`ExpressionNode::span_range()`** - Added method to compute span recursively

### Files Updated

- `src/expressions/type_resolution/arguments.rs` - Core resolution traits
- `src/expressions/type_resolution/interface_macros.rs` - Apply functions pass spans
- `src/expressions/operations.rs` - Binary operation traits
- `src/expressions/control_flow.rs` - Control flow expressions use proper spans
- `src/expressions/expression.rs` - Added span_range methods
- `src/expressions/evaluation/assignment_frames.rs` - Assignment destructuring
- `src/expressions/evaluation/value_frames.rs` - Object key resolution
- `src/expressions/values/float.rs` - Float operations with [context]
- `src/expressions/values/iterable.rs` - IterableRef from_argument

---

## Remaining Work

### Files Still Needing Updates

1. **`src/expressions/patterns.rs`** - Multiple `resolve_as` calls need span parameter
2. **`src/expressions/values/integer.rs`** - All binary operations need `[context]` and span
3. **`src/expressions/values/integer_subtypes.rs`** - Binary operations
4. **`src/expressions/values/integer_untyped.rs`** - Binary operations
5. **`src/expressions/values/float_subtypes.rs`** - If any binary operations exist
6. **`src/expressions/values/float_untyped.rs`** - Binary operations
7. **`src/expressions/values/array.rs`** - `resolve_as` for array index
8. **`src/expressions/values/object.rs`** - `resolve_as` for object keys
9. **`src/expressions/values/range.rs`** - `resolve_as` for range bounds
10. **`src/expressions/values/parser.rs`** - `resolve_as` for parser values
11. **`src/expressions/values/value.rs`** - Various remaining call_site usages
12. **`src/expressions/statements.rs`** - Statement span handling
13. **`src/misc/field_inputs.rs`** - Field input spans

### Pattern for Fixes

For operations without `[context]`, add `[context]` attribute:
```rust
// Before
fn add(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
    left.paired_comparison(right, |a, b| a + b)
}

// After
[context] fn add(left: Owned<IntegerValue>, right: Owned<IntegerValue>) -> ExecutionResult<IntegerValue> {
    let span = context.output_span_range;
    left.paired_comparison(right, span, |a, b| a + b)
}
```

For `resolve_as` calls, pass the span:
```rust
// Before
value.resolve_as("message")

// After
value.resolve_as(span_range, "message")
```

---

## Progress Tracking

- [x] Category 3: `type_data.rs` operation spans - **DONE**: `Spanned<ArgumentValue>` now used
- [x] Category 2: `value_frames.rs` ownership mappings - **DONE**: All `map_from_*` take spans
- [x] Category 1 partial: `arguments.rs` - **IN PROGRESS**: Core traits updated, call sites need fixing
- [x] Float operations - **DONE**: All operations now use `[context]`
- [ ] Integer operations - Need `[context]` on binary operations
- [ ] Patterns - Need span parameters on `resolve_as` calls
- [ ] Array/Object/Range - Need span parameters on `resolve_as` calls
- [ ] Parser - Need span parameters
- [ ] Remaining value.rs usages

**Remaining: ~103 compilation errors**
