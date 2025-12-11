# Plan: Remove `Span::call_site()` Fallbacks

This document tracks the fixes needed to remove all `Span::call_site()` occurrences introduced in the SpanRanges refactoring PR.

**Principle**: Spans live on the *outside* via `Spanned<T>` wrappers, not embedded in types.

**Current Status**: COMPLETE - Build compiles, ~18 call_site() usages remain (all acceptable).

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

6. **`enable()` methods** - Now take span parameter for error reporting:
   - `Mutable<Value>::enable(span_range)`
   - `Shared<Value>::enable(span_range)`
   - `CopyOnWrite<T>::enable(span_range)`
   - `ArgumentValue::enable(span_range)`

7. **Interface unary operations** - Added `[context]` attribute for span access:
   - `cast_to_string`, `cast_to_stream` on Value
   - `neg` on integer types (for overflow errors)

### Files Updated

- `src/expressions/type_resolution/arguments.rs` - ResolveAs trait refactored
- `src/expressions/evaluation/evaluator.rs` - Evaluate trait method renaming
- `src/expressions/expression.rs` - evaluate methods return Spanned
- `src/interpretation/bindings.rs` - enable() methods take span, ownership errors use span
- `src/interpretation/refs.rs` - Removed From impls that used call_site(), added ToSpannedRef impl
- `src/expressions/control_flow.rs` - All control flow uses new patterns
- `src/expressions/operations.rs` - Binary operation helper methods
- `src/expressions/values/float.rs` - All binary ops wrap rhs in Spanned
- `src/expressions/values/float_untyped.rs` - paired_operation takes Spanned<Owned<FloatValue>>
- `src/expressions/values/integer.rs` - All binary ops wrap rhs in Spanned
- `src/expressions/values/integer_untyped.rs` - paired operations take Spanned, neg uses [context]
- `src/expressions/values/integer_subtypes.rs` - neg uses [context] for overflow errors
- `src/expressions/values/value.rs` - Methods use [context] for spans, into_stream takes span
- `src/expressions/values/array.rs` - Array index resolution
- `src/expressions/values/object.rs` - Object key resolution
- `src/expressions/values/range.rs` - Range bound resolution
- `src/expressions/values/parser.rs` - Parser methods use context.output_span_range
- `src/expressions/patterns.rs` - Pattern destructuring
- `src/expressions/statements.rs` - EmitStatement uses span from evaluate
- `src/expressions/evaluation/assignment_frames.rs` - Assignment destructuring
- `src/expressions/evaluation/value_frames.rs` - enable() calls pass span
- `src/misc/field_inputs.rs` - from_object_value takes span parameter

---

## Remaining `call_site()` Usages (~18)

These are categorized by their nature and considered acceptable:

### Category 1: Entry Points / Public API (Acceptable)
- `src/lib.rs` - Entry points for macro parsing (5 usages)

### Category 2: Debug/Display Helpers (Acceptable - Low Priority)
- `src/expressions/values/stream.rs` - Debug formatting (3 usages in Debug impl, equality testing)

### Category 3: Error/Parse Fallbacks (Acceptable)
- `src/misc/errors.rs` - Default span for errors (1 usage)
- `src/extensions/parsing.rs` - Parse error fallback (1 usage)
- `src/extensions/errors_and_spans.rs` - Token iterator fallback (1 usage)

### Category 4: Stream Method Fallbacks (Acceptable)
- `src/expressions/values/stream.rs` - Stream methods use `resolve_content_span_range().unwrap_or(...)` (4 usages)

### Category 5: Generated Token Span (Acceptable)
- `src/expressions/values/value.rs` - `new_token_span()` for generated tokens (1 usage)

### Category 6: Test Code (Acceptable)
- `src/sandbox/gat_value.rs` - Test helper (1 usage)

---

## Design Principles Established

1. **Spanned wrappers over span parameters**: When a span is conceptually tied to a value, use `Spanned<T>` rather than passing the span as a separate parameter.

2. **Expression evaluation returns Spanned**: `Expression::evaluate_owned()` returns `Spanned<OwnedValue>` so callers can chain directly to `.resolve_as("target")?`

3. **ResolveAs on Spanned types**: The `ResolveAs` trait is implemented on `Spanned<Owned<V>>`, `Spanned<&Value>`, etc. so the span is carried with the value being resolved.

4. **Control flow simplification**: Conditions like `if cond.evaluate_owned()?.resolve_as("condition")?` now work without intermediate destructuring.

5. **[context] attribute for interface methods**: When an interface method needs span access, add `[context]` attribute to get `context.output_span_range`.

6. **Fallback spans are acceptable in specific cases**: Entry points, debug formatting, error defaults, and generated tokens legitimately use call_site().

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
- [x] Remaining call_site() usages investigated and categorized as acceptable
