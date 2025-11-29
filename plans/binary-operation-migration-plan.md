# BinaryOperation Migration Plan

## Overview

This document outlines the plan to migrate binary operations from the current `HandleBinaryOperation` trait pattern to the new method resolution pattern (similar to unary operations).

**Current State:**
- Binary operations use `HandleBinaryOperation` trait with `handle_paired_binary_operation()` and `handle_integer_binary_operation()` methods
- Evaluation flows through `ExpressionValuePair::handle_paired_binary_operation()`
- Type matching happens via `ExpressionValue::expect_value_pair()` which creates typed pairs

**Target State:**
- Binary operations use method resolution via `resolve_own_binary_operation()` on `HierarchicalTypeData`
- Operations are defined in `binary_operations` modules within each type's `define_interface!` block
- Type coercion happens at resolution time, similar to unary operations

---

## First PR Scope: Infrastructure + Addition on UntypedInteger

### Goals

1. **Establish the infrastructure** for binary operation method resolution
2. **Migrate one operator** (`+`) for one type (`UntypedInteger`) as proof of concept
3. **Maintain backwards compatibility** - existing tests must pass

### Non-Goals (Deferred to Later PRs)

- Migrating all operators
- Migrating typed integers (i8, u8, i16, etc.)
- Implementing `MaybeTypedInt<X>` wrapper
- Implementing `CoercedInt<u32>` for shift operators
- CompoundAssignment migration

---

## Implementation Steps

### Step 1: Add BinaryOperationInterface

**File:** `src/expressions/type_resolution/type_data.rs`

Add a new interface struct similar to `UnaryOperationInterface`:

```rust
pub struct BinaryOperationInterface {
    pub method: fn(BinaryOperationCallContext, ResolvedValue, ResolvedValue) -> ExecutionResult<ResolvedValue>,
    pub lhs_ownership: ResolvedValueOwnership,
    pub rhs_ownership: ResolvedValueOwnership,
}

pub struct BinaryOperationCallContext<'a> {
    pub operation: &'a BinaryOperation,
    pub output_span_range: SpanRange,
}
```

### Step 2: Add resolve_own_binary_operation to HierarchicalTypeData

**File:** `src/expressions/type_resolution/type_data.rs`

Extend the `HierarchicalTypeData` trait:

```rust
fn resolve_own_binary_operation(
    &self,
    operation: &BinaryOperation,
    rhs_kind: &ExpressionValueKind,
) -> Option<BinaryOperationInterface> {
    None  // Default implementation
}
```

Note: We include `rhs_kind` to enable type-aware resolution (e.g., `UntypedInteger + i32` can resolve differently than `UntypedInteger + UntypedInteger`).

### Step 3: Add binary_operations to define_interface! macro

**File:** `src/expressions/type_resolution/interface_macros.rs`

Extend the macro to support:

```rust
define_interface! {
    struct UntypedIntegerTypeData,
    pub(crate) mod untyped_integer_interface {
        pub(crate) mod binary_operations {
            [context] fn add(this: UntypedInteger, rhs: IntegerExpression) -> ExecutionResult<ResolvedValue> {
                // implementation
            }
        }
        interface_items {
            fn resolve_own_binary_operation(
                operation: &BinaryOperation,
                rhs_kind: &ExpressionValueKind,
            ) -> Option<BinaryOperationInterface> {
                // resolution logic
            }
        }
    }
}
```

### Step 4: Implement Addition for UntypedInteger

**File:** `src/expressions/values/integer.rs`

Add binary operation resolution for `UntypedInteger`:

```rust
pub(crate) mod binary_operations {
    [context] fn add(this: UntypedInteger, rhs: IntegerExpression) -> ExecutionResult<ResolvedValue> {
        match rhs.value {
            IntegerExpressionValue::Untyped(rhs) => {
                // Both untyped: use existing handle_paired_binary_operation
                this.handle_paired_binary_operation(rhs, PairedBinaryOperation::Add)
                    .map(|v| ResolvedValue::from_value(v, context.output_span_range))
            }
            typed_rhs => {
                // LHS untyped, RHS typed: coerce LHS to RHS's type
                let lhs = this.to_kind(typed_rhs.kind())?;
                PairedBinaryOperation::Add.evaluate_typed(lhs, typed_rhs)
                    .map(|v| ResolvedValue::from_value(v, context.output_span_range))
            }
        }
    }
}

interface_items {
    fn resolve_own_binary_operation(
        operation: &BinaryOperation,
        rhs_kind: &ExpressionValueKind,
    ) -> Option<BinaryOperationInterface> {
        // Only handle cases where RHS is an integer
        if !matches!(rhs_kind, ExpressionValueKind::Integer(_)) {
            return None;
        }

        match operation {
            BinaryOperation::Paired(PairedBinaryOperation::Add) => {
                Some(binary_definitions::add())
            }
            _ => None,  // Other operators use old path for now
        }
    }
}
```

### Step 5: Update Evaluation Flow

**File:** `src/expressions/evaluation/value_frames.rs`

Modify the binary operation evaluation to try method resolution first:

```rust
// In the binary operation evaluation path:
if let Some(interface) = lhs.kind().resolve_binary_operation(&operation, &rhs.kind()) {
    // Use new method resolution path
    return interface.execute(lhs, rhs, context);
}

// Fall back to old HandleBinaryOperation path
// ... existing code ...
```

### Step 6: Add Tests

Add tests verifying:
1. `UntypedInteger + UntypedInteger` works via new path
2. `UntypedInteger + i32` coerces correctly
3. `i32 + UntypedInteger` still works (via old path initially)
4. All existing integer addition tests pass

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/expressions/type_resolution/type_data.rs` | Add `BinaryOperationInterface`, extend `HierarchicalTypeData` |
| `src/expressions/type_resolution/interface_macros.rs` | Add `binary_operations` block support |
| `src/expressions/values/integer.rs` | Add `binary_operations` for `UntypedInteger` |
| `src/expressions/evaluation/value_frames.rs` | Try method resolution before old path |
| `src/expressions/operations.rs` | Possibly add helper methods |

---

## Future PRs

### PR 2: Migrate All Arithmetic Operators for UntypedInteger
- `-`, `*`, `/`, `%` for UntypedInteger
- Establish pattern for other operators

### PR 3: Migrate Typed Integers
- Introduce `MaybeTypedInt<X>` wrapper type
- Apply pattern to all 12 typed integer kinds
- Refactor to reduce code duplication

### PR 4: Comparison Operators
- `==`, `!=`, `<`, `<=`, `>`, `>=`
- Special handling for `==` to avoid clones

### PR 5: Bitwise Operators
- `&`, `|`, `^`

### PR 6: Shift Operators with Optimization
- `<<`, `>>`
- Introduce `CoercedInt<u32>` for RHS
- Use `.checked_shl(u32)` / `.checked_shr(u32)`

### PR 7: Other Value Types
- Floats, Booleans, Strings, Arrays, Objects, Streams

### PR 8: CompoundAssignment Migration
- `+=`, `-=`, etc.

### PR 9: Cleanup
- Remove old `HandleBinaryOperation` trait
- Address remaining `TODO[operation-refactor]` comments
- Clean up `ExpressionValuePair` if no longer needed

---

## Success Criteria

For the first PR:
- [ ] All existing tests pass
- [ ] `UntypedInteger + UntypedInteger` uses new method resolution
- [ ] `UntypedInteger + <TypedInteger>` coerces and operates correctly
- [ ] No regression in error messages or spans
- [ ] Code follows existing patterns in the codebase

---

## Design Decisions

1. **RHS type in resolution**: `resolve_own_binary_operation` takes just the RHS kind, not the full value. This may make short-circuiting operators harder (see below), but we'll try this approach first.

2. **Symmetric operations**: Define interfaces as asymmetric based on LHS. For `i32 + UntypedInteger`, the `i32`'s resolver handles it. Implementation can delegate internally (e.g., `a + b` can call the inner method for `b + a` when `a != b`).

3. **Error span handling**: Add an error span range to the context, derived from the operator token.

---

## Short-Circuiting Considerations

The short-circuiting operators (`&&` and `||`) present a challenge for this architecture:

**The Problem:**
- Currently, `BinaryOperation::lazy_evaluate()` handles `&&` and `||` by evaluating LHS first, then conditionally evaluating RHS
- If `resolve_own_binary_operation` requires `rhs_kind`, we'd need to evaluate RHS to get its kind, which defeats short-circuiting
- We don't currently have type data before evaluation

**Possible Approaches:**

1. **Don't migrate `&&` and `||`** - Keep them on the old evaluation path. Simple, but leaves the migration incomplete.

2. **Resolution without RHS kind for short-circuit ops** - Have a separate resolution path or allow `rhs_kind` to be `None` for these operators. The resolved method would receive a thunk/closure for the RHS.

3. **Two-phase evaluation** - For short-circuit ops:
   - Phase 1: Resolve based on LHS only, get a "lazy" interface
   - Phase 2: The interface method evaluates RHS if needed and handles type checking internally

4. **Type inference** - If we had static type information from earlier passes, we could resolve without evaluating. But this would be a larger architectural change.

**Recommendation for First PR:**
Exclude `&&` and `||` from the migration initially. They can remain on the old path while we migrate the eager operators. This keeps the first PR focused and avoids premature architectural decisions.

**Future Consideration:**
When we do tackle short-circuiting, approach #2 or #3 seems most aligned with the current architecture. The method signature could be:

```rust
// Option: Lazy RHS parameter
fn and(this: Boolean, rhs: impl FnOnce() -> ExecutionResult<ResolvedValue>) -> ExecutionResult<ResolvedValue> {
    if !this.value {
        return Ok(ResolvedValue::from(false));
    }
    let rhs = rhs()?;
    // ... type check and compute
}
```

Or we could have `BinaryOperationInterface` include a `is_short_circuit: bool` flag that changes how evaluation is handled.

---

## References

- Current unary operation implementation: `src/expressions/operations.rs:80-222`
- Method resolution pattern: `src/expressions/type_resolution/type_data.rs`
- TODO.md section: `plans/TODO.md:44-97`
