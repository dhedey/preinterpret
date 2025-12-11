# Plan: Remove `Span::call_site()` Fallbacks

This document tracks the fixes needed to remove all `Span::call_site()` occurrences introduced in the SpanRanges refactoring PR.

**Principle**: Spans live on the *outside* via `Spanned<T>` wrappers, not embedded in types.

**Current Status**: IN PROGRESS - Build compiles, ~30 call_site() usages remain.

---

## Summary of Changes Made

### Core API Changes

1. **`Evaluate` trait** - Renamed methods:
   - `evaluate()` - returns unspanned value
   - `evaluate_spanned()` - returns `Spanned<RequestedValue>`

2. **`Expression` evaluation methods** - Now return Spanned types:
   - `evaluate()` → `Spanned<RequestedValue>`
   - `evaluate_owned()` → `Spanned<OwnedValue>`
   - `evaluate_shared()` → `Spanned<SharedValue>`

3. **`ResolveAs` trait** - Now implemented on `Spanned<X>` instead of taking span parameter:
   - `impl ResolveAs<T> for Spanned<Owned<V>>`
   - `impl ResolveAs<Owned<T>> for Spanned<Owned<Value>>`
   - `impl ResolveAs<Shared<T>> for Spanned<Shared<Value>>`
   - `impl ResolveAs<&T> for Spanned<&Value>`
   - `impl ResolveAs<&mut T> for Spanned<&mut Value>`
   - `impl ResolveAs<Mutable<T>> for Spanned<Mutable<Value>>`

4. **`HandleBinaryOperation` helper methods** - Updated signatures:
   - `paired_operation_no_overflow` - now takes `impl ResolveAs<Self>` (Spanned wrapper handles span)
   - `paired_comparison` - now takes `impl ResolveAs<Self>` (Spanned wrapper handles span)

5. **UntypedInteger/UntypedFloat operations** - Take `Spanned<Owned<...>>` for rhs

### Files Updated

- `src/expressions/type_resolution/arguments.rs` - ResolveAs trait refactored
- `src/expressions/evaluation/evaluator.rs` - Evaluate trait method renaming
- `src/expressions/expression.rs` - evaluate methods return Spanned
- `src/interpretation/bindings.rs` - Added `impl Spanned<OwnedValue>::into_statement_result()`
- `src/expressions/control_flow.rs` - All control flow uses new patterns
- `src/expressions/operations.rs` - Binary operation helper methods
- `src/expressions/values/float.rs` - All binary ops wrap rhs in Spanned
- `src/expressions/values/float_untyped.rs` - paired_operation takes Spanned<Owned<FloatValue>>
- `src/expressions/values/integer.rs` - All binary ops wrap rhs in Spanned
- `src/expressions/values/integer_untyped.rs` - paired operations take Spanned<Owned<IntegerValue>>
- `src/expressions/values/array.rs` - Array index resolution
- `src/expressions/values/object.rs` - Object key resolution
- `src/expressions/values/range.rs` - Range bound resolution
- `src/expressions/values/parser.rs` - Parser value resolution
- `src/expressions/patterns.rs` - Pattern destructuring
- `src/expressions/statements.rs` - EmitStatement uses span from evaluate
- `src/expressions/evaluation/assignment_frames.rs` - Assignment destructuring
- `src/expressions/evaluation/value_frames.rs` - Object key resolution
- `src/misc/field_inputs.rs` - Field input macro

---

## Remaining `call_site()` Usages (~30)

These are categorized by their nature:

### Category 1: Entry Points / Public API (Acceptable)
- `src/lib.rs` - Entry points for macro parsing (3 usages)

### Category 2: Debug/Display Helpers (Low Priority)
- `src/expressions/values/stream.rs` - Debug formatting (4 usages)

### Category 3: Error Fallbacks (Need Investigation)
- `src/misc/errors.rs` - Error default span (1 usage)
- `src/expressions/values/integer_untyped.rs` - Overflow error (1 usage)
- `src/expressions/values/integer_subtypes.rs` - Overflow error (1 usage)

### Category 4: Needs Span Threading
- `src/expressions/values/value.rs` - Various method implementations (~8 usages)
- `src/expressions/values/parser.rs` - Parser fallback spans (3 usages)
- `src/misc/field_inputs.rs` - Field input fallback span (1 usage)
- `src/interpretation/refs.rs` - Ref default spans (2 usages)
- `src/interpretation/bindings.rs` - HasSpan defaults (2 usages)

### Category 5: Test Code (Acceptable)
- `src/sandbox/gat_value.rs` - Test helper (1 usage)
- `tests/compilation_failures/` - Test comments

---

## Design Principles Established

1. **Spanned wrappers over span parameters**: When a span is conceptually tied to a value, use `Spanned<T>` rather than passing the span as a separate parameter.

2. **Expression evaluation returns Spanned**: `Expression::evaluate_owned()` returns `Spanned<OwnedValue>` so callers can chain directly to `.resolve_as("target")?`

3. **ResolveAs on Spanned types**: The `ResolveAs` trait is implemented on `Spanned<Owned<V>>`, `Spanned<&Value>`, etc. so the span is carried with the value being resolved.

4. **Control flow simplification**: Conditions like `if cond.evaluate_owned()?.resolve_as("condition")?` now work without intermediate destructuring.

---

## Progress Tracking

- [x] Core `Evaluate` trait refactored
- [x] Expression evaluation returns Spanned
- [x] `ResolveAs` trait refactored to use Spanned
- [x] Float operations use Spanned wrapper
- [x] Integer operations use Spanned wrapper
- [x] Control flow uses new patterns
- [x] Pattern destructuring updated
- [x] Assignment frames updated
- [x] All compilation errors fixed
- [x] All tests passing
- [ ] Remaining ~30 call_site() usages need investigation
