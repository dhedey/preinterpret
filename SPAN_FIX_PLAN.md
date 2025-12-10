# Plan: Remove `Span::call_site()` Fallbacks

This document tracks the fixes needed to remove all `Span::call_site()` occurrences introduced in the SpanRanges refactoring PR.

**Principle**: Spans live on the *outside* via `Spanned<T>` wrappers, not embedded in types.

---

## Category 3: Operation Interface Spans (`type_data.rs`)

**Status**: [ ] Not started

**Files**: `src/expressions/type_resolution/type_data.rs`

**Problem**: `UnaryOperationInterface::execute` and `BinaryOperationInterface::execute` need input spans to compute output spans.

**Original code**:
```rust
fn execute(&self, input: ArgumentValue, operation: &UnaryOperation) {
    let output_span_range = operation.output_span_range(input.span_range());
}
```

**Current (broken)**:
```rust
fn execute(&self, input: ArgumentValue, operation: &UnaryOperation) {
    let fallback_span = Span::call_site().span_range();  // BAD
    let output_span_range = operation.output_span_range(fallback_span);
}
```

**Fix**: Change to take `Spanned<ArgumentValue>`:
```rust
fn execute(&self, Spanned(input, input_span): Spanned<ArgumentValue>, operation: &UnaryOperation) {
    let output_span_range = operation.output_span_range(input_span);
}
```

**Call sites to update** (spans already available!):
- `value_frames.rs:731` - has `operand_span`
- `value_frames.rs:858` - has `left_span`, `right_span`
- `operations.rs:95` - needs span threaded through `UnaryOperation::evaluate`

---

## Category 2: Ownership Mapping Errors (`value_frames.rs`)

**Status**: [ ] Not started

**Files**: `src/expressions/evaluation/value_frames.rs`

**Problem**: `ArgumentOwnership::map_from_*` methods need spans for ownership error messages.

**Occurrences** (10 total):
- `map_from_late_bound` - `LateBoundOwnedValue` span_range field
- `map_from_copy_on_write` - 3 ownership errors
- `map_from_shared` - 1 ownership error
- `map_from_mutable` - 1 ownership error
- `map_from_owned` - 3 ownership errors

**Fix**: Add `span_range: SpanRange` parameter to all `map_from_*` methods.

---

## Category 1: Type Resolution Functions (`arguments.rs`)

**Status**: [ ] Not started

**Files**: `src/expressions/type_resolution/arguments.rs`

**Problem**: `ResolvableOwned`/`ResolvableShared`/`ResolvableMutable` methods lost access to spans.

**Occurrences** (6 total):
- `IsArgument for Spanned<T>::from_argument`
- `ResolvableOwned::resolve_value`
- `ResolvableOwned::resolve_owned`
- `ResolvableShared::resolve_shared`
- `ResolvableMutable::resolve_mutable`

**Fix**: These methods should take `Spanned<Owned<T>>`, `Spanned<Shared<T>>`, etc.

---

## Categories 4-8: Value Type Operations

**Status**: [ ] Not started

### Category 4: `integer.rs` (2 occurrences)
- `coerce_to_other_type` - needs `Spanned<Owned<IntegerValue>>`
- `coerce_to_target_type` - needs `Spanned<Owned<IntegerValue>>`

### Category 5: `integer_subtypes.rs` / `integer_untyped.rs` (2 occurrences)
- Signed integer `neg` overflow error
- Untyped integer `neg` overflow error

### Category 6: `iterable.rs` (1 occurrence)
- `IterableRef::from_argument` type error

### Category 7: `iterator.rs` (1 occurrence)
- `cast_singleton_to_value` error

### Category 8: `stream.rs` (multiple occurrences)
- `cast_to_value` coercion error
- Debug output methods
- Concatenation operations

**Fix**: These unary operations receive arguments via the interface system. The span should flow through `UnaryOperationCallContext` (available as `output_span_range`) or operations should receive `Spanned<Owned<T>>`.

---

## Category 9: Parser Errors (`parser.rs`)

**Status**: [ ] Not started

**Files**: `src/expressions/values/parser.rs`

**Occurrences** (3):
- `as_parse_stream` - needs span from `Shared<ParserValue>`
- `open` delimiter error - needs span from `Owned<char>`
- `close` delimiter error - needs span from `Owned<char>`

**Fix**: Take `Spanned<Shared<ParserValue>>` and `Spanned<Owned<char>>`.

---

## Category 10: Statement/Expression Spans

**Status**: [ ] Keep using leaf spans (per user guidance)

**Files**: `src/expressions/expression.rs`, `src/expressions/statements.rs`

**Guidance**: Use leaf spans (e.g., `block.span().span_range()`). Avoid functional changes.

---

## Execution Order

1. **Category 3 first** - Operation interfaces are foundational
2. **Category 2** - Ownership mapping uses operation results
3. **Categories 4-8** - Value operations depend on interface changes
4. **Category 1** - Resolution functions (may cascade from above)
5. **Category 9** - Parser (relatively isolated)
6. **Category 10** - Already handled with leaf spans

---

## Progress Tracking

- [x] Category 3: `type_data.rs` operation spans - **DONE**: `Spanned<ArgumentValue>` now used
- [x] Category 2 partial: `value_frames.rs` `map_from_owned` - **DONE**: spans threaded through
- [ ] Category 2 remaining: `value_frames.rs` `map_from_copy_on_write`, `map_from_shared`, `map_from_mutable` (6 usages)
- [ ] Category 1: `arguments.rs` resolution functions (5 usages)
- [ ] Category 4: `integer.rs` coercion (2 usages)
- [ ] Category 5: `integer_subtypes.rs` / `integer_untyped.rs` negation (2 usages)
- [ ] Category 6: `iterable.rs` type error (1 usage)
- [x] Category 7: `iterator.rs` singleton cast - **DONE**: uses `context.output_span_range`
- [x] Category 8 partial: `stream.rs` cast_to_value - **DONE**: uses `context.output_span_range`
- [ ] Category 8 remaining: `stream.rs` debug output and concatenation (~7 usages)
- [ ] Category 9: `parser.rs` errors (3 usages)
- [ ] Category 10: `expression.rs` / `statements.rs` (2 usages - may want leaf spans)

**Remaining: ~28 call_site usages**
