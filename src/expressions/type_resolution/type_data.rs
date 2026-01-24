#![allow(clippy::type_complexity)]
use super::*;

pub(crate) trait TypeFeatureResolver {
    /// Resolves a method with the given name defined on this type.
    /// A method is like a function, but guaranteed to have a first argument
    /// (the receiver) which is assignable from the type.
    fn resolve_method(&self, method_name: &str) -> Option<FunctionInterface>;

    /// Resolves a unary operation as a method interface for this type.
    fn resolve_unary_operation(
        &self,
        operation: &UnaryOperation,
    ) -> Option<UnaryOperationInterface>;

    /// Resolves a binary operation as a method interface for this type.
    fn resolve_binary_operation(
        &self,
        operation: &BinaryOperation,
    ) -> Option<BinaryOperationInterface>;

    /// Resolves a function with the given name defined on this type.
    fn resolve_type_function(&self, function_name: &str) -> Option<FunctionInterface>;

    /// Resolves a property of this type.
    fn resolve_type_property(&self, _property_name: &str) -> Option<AnyValue>;

    /// Resolves property access capability for this type (e.g., `obj.field`).
    /// Returns Some if this type supports property access, None otherwise.
    fn resolve_property_access(&self) -> Option<PropertyAccessInterface> {
        None
    }

    /// Resolves index access capability for this type (e.g., `arr[0]`).
    /// Returns Some if this type supports indexing, None otherwise.
    fn resolve_index_access(&self) -> Option<IndexAccessInterface> {
        None
    }
}

pub(crate) trait TypeData {
    /// Returns None if the method is not supported on this type itself.
    /// The method may still be supported on a type further up the resolution chain.
    fn resolve_own_method(_method_name: &str) -> Option<FunctionInterface> {
        None
    }

    /// Returns None if the operation is not supported on this type itself.
    /// The operation may still be supported on a type further up the resolution chain.
    fn resolve_own_unary_operation(_operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        None
    }

    /// Returns None if the operation is not supported on this type itself.
    /// The operation may still be supported on a type further up the resolution chain.
    fn resolve_own_binary_operation(
        _operation: &BinaryOperation,
    ) -> Option<BinaryOperationInterface> {
        None
    }

    /// Returns a property on the type.
    /// Properties are *not* currently resolved up the resolution chain... but maybe they should be?
    /// ... similarly, maybe functions should be too, when they are added?
    fn resolve_type_property(_property_name: &str) -> Option<AnyValue> {
        None
    }

    /// Returns None if the function is not supported on this type itself.
    fn resolve_type_function(_function_name: &str) -> Option<FunctionInterface> {
        None
    }

    /// Returns the property access interface for this type, if supported.
    fn resolve_own_property_access() -> Option<PropertyAccessInterface> {
        None
    }

    /// Returns the index access interface for this type, if supported.
    fn resolve_own_index_access() -> Option<IndexAccessInterface> {
        None
    }
}

#[allow(unused)]
// It's good enough for our needs for now
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum FunctionInterface {
    Arity0 {
        method: fn(&mut FunctionCallContext) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 0],
    },
    Arity1 {
        method:
            fn(&mut FunctionCallContext, Spanned<ArgumentValue>) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 1],
    },
    /// 1 argument, 1 optional argument
    Arity1PlusOptional1 {
        method: fn(
            &mut FunctionCallContext,
            Spanned<ArgumentValue>,
            Option<Spanned<ArgumentValue>>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 2],
    },
    Arity2 {
        method: fn(
            &mut FunctionCallContext,
            Spanned<ArgumentValue>,
            Spanned<ArgumentValue>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 2],
    },
    Arity2PlusOptional1 {
        method: fn(
            &mut FunctionCallContext,
            Spanned<ArgumentValue>,
            Spanned<ArgumentValue>,
            Option<Spanned<ArgumentValue>>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 3],
    },
    Arity3 {
        method: fn(
            &mut FunctionCallContext,
            Spanned<ArgumentValue>,
            Spanned<ArgumentValue>,
            Spanned<ArgumentValue>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 3],
    },
    Arity3PlusOptional1 {
        method: fn(
            &mut FunctionCallContext,
            Spanned<ArgumentValue>,
            Spanned<ArgumentValue>,
            Spanned<ArgumentValue>,
            Option<Spanned<ArgumentValue>>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 4],
    },
    ArityAny {
        method: fn(
            &mut FunctionCallContext,
            Vec<Spanned<ArgumentValue>>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: Vec<ArgumentOwnership>,
    },
}

impl FunctionInterface {
    pub(crate) fn execute(
        &self,
        arguments: Vec<Spanned<ArgumentValue>>,
        context: &mut FunctionCallContext,
    ) -> ExecutionResult<Spanned<ReturnedValue>> {
        let output_value = match self {
            FunctionInterface::Arity0 { method, .. } => {
                if !arguments.is_empty() {
                    return context.output_span_range.type_err("Expected 0 arguments");
                }
                method(context)
            }
            FunctionInterface::Arity1 { method, .. } => {
                match <[Spanned<ArgumentValue>; 1]>::try_from(arguments) {
                    Ok([a]) => method(context, a),
                    Err(_) => context.output_span_range.type_err("Expected 1 argument"),
                }
            }
            FunctionInterface::Arity1PlusOptional1 { method, .. } => match arguments.len() {
                1 => {
                    let [a] = <[Spanned<ArgumentValue>; 1]>::try_from(arguments)
                        .ok()
                        .unwrap();
                    method(context, a, None)
                }
                2 => {
                    let [a, b] = <[Spanned<ArgumentValue>; 2]>::try_from(arguments)
                        .ok()
                        .unwrap();
                    method(context, a, Some(b))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 1 or 2 arguments"),
            },
            FunctionInterface::Arity2 { method, .. } => {
                match <[Spanned<ArgumentValue>; 2]>::try_from(arguments) {
                    Ok([a, b]) => method(context, a, b),
                    Err(_) => context.output_span_range.type_err("Expected 2 arguments"),
                }
            }
            FunctionInterface::Arity2PlusOptional1 { method, .. } => match arguments.len() {
                2 => {
                    let [a, b] = <[Spanned<ArgumentValue>; 2]>::try_from(arguments)
                        .ok()
                        .unwrap();
                    method(context, a, b, None)
                }
                3 => {
                    let [a, b, c] = <[Spanned<ArgumentValue>; 3]>::try_from(arguments)
                        .ok()
                        .unwrap();
                    method(context, a, b, Some(c))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 2 or 3 arguments"),
            },
            FunctionInterface::Arity3 { method, .. } => {
                match <[Spanned<ArgumentValue>; 3]>::try_from(arguments) {
                    Ok([a, b, c]) => method(context, a, b, c),
                    Err(_) => context.output_span_range.type_err("Expected 3 arguments"),
                }
            }
            FunctionInterface::Arity3PlusOptional1 { method, .. } => match arguments.len() {
                3 => {
                    let [a, b, c] = <[Spanned<ArgumentValue>; 3]>::try_from(arguments)
                        .ok()
                        .unwrap();
                    method(context, a, b, c, None)
                }
                4 => {
                    let [a, b, c, d] = <[Spanned<ArgumentValue>; 4]>::try_from(arguments)
                        .ok()
                        .unwrap();
                    method(context, a, b, c, Some(d))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 3 or 4 arguments"),
            },
            FunctionInterface::ArityAny { method, .. } => method(context, arguments),
        };
        output_value.map(|v| v.spanned(context.output_span_range))
    }

    /// Returns (argument_ownerships, required_argument_count)
    pub(crate) fn argument_ownerships(&self) -> (&[ArgumentOwnership], usize) {
        match self {
            FunctionInterface::Arity0 {
                argument_ownership, ..
            } => (argument_ownership, 0),
            FunctionInterface::Arity1 {
                argument_ownership, ..
            } => (argument_ownership, 1),
            FunctionInterface::Arity1PlusOptional1 {
                argument_ownership, ..
            } => (argument_ownership, 1),
            FunctionInterface::Arity2 {
                argument_ownership, ..
            } => (argument_ownership, 2),
            FunctionInterface::Arity2PlusOptional1 {
                argument_ownership, ..
            } => (argument_ownership, 2),
            FunctionInterface::Arity3 {
                argument_ownership, ..
            } => (argument_ownership, 3),
            FunctionInterface::Arity3PlusOptional1 {
                argument_ownership, ..
            } => (argument_ownership, 3),
            FunctionInterface::ArityAny {
                argument_ownership, ..
            } => (argument_ownership, 0),
        }
    }
}

pub(crate) struct UnaryOperationInterface {
    pub method:
        fn(UnaryOperationCallContext, Spanned<ArgumentValue>) -> ExecutionResult<ReturnedValue>,
    pub argument_ownership: ArgumentOwnership,
}

impl UnaryOperationInterface {
    pub(crate) fn execute(
        &self,
        Spanned(input, input_span): Spanned<ArgumentValue>,
        operation: &UnaryOperation,
    ) -> ExecutionResult<Spanned<ReturnedValue>> {
        let output_span_range = operation.output_span_range(input_span);
        Ok((self.method)(
            UnaryOperationCallContext { operation },
            Spanned(input, input_span),
        )?
        .spanned(output_span_range))
    }

    pub(crate) fn argument_ownership(&self) -> ArgumentOwnership {
        self.argument_ownership
    }
}

pub(crate) struct BinaryOperationInterface {
    pub method: fn(
        BinaryOperationCallContext,
        Spanned<ArgumentValue>,
        Spanned<ArgumentValue>,
    ) -> ExecutionResult<ReturnedValue>,
    pub lhs_ownership: ArgumentOwnership,
    pub rhs_ownership: ArgumentOwnership,
}

impl BinaryOperationInterface {
    pub(crate) fn execute(
        &self,
        Spanned(lhs, lhs_span): Spanned<ArgumentValue>,
        Spanned(rhs, rhs_span): Spanned<ArgumentValue>,
        operation: &BinaryOperation,
    ) -> ExecutionResult<Spanned<ReturnedValue>> {
        let output_span_range = SpanRange::new_between(lhs_span, rhs_span);
        Ok((self.method)(
            BinaryOperationCallContext { operation },
            Spanned(lhs, lhs_span),
            Spanned(rhs, rhs_span),
        )?
        .spanned(output_span_range))
    }

    pub(crate) fn lhs_ownership(&self) -> ArgumentOwnership {
        self.lhs_ownership
    }

    pub(crate) fn rhs_ownership(&self) -> ArgumentOwnership {
        self.rhs_ownership
    }
}

// ============================================================================
// Property Access Interface
// ============================================================================

/// Context provided to property access methods.
#[derive(Clone, Copy)]
pub(crate) struct PropertyAccessCallContext<'a> {
    pub property: &'a PropertyAccess,
}

/// Interface for property access on a type (e.g., `obj.field`).
///
/// Unlike unary/binary operations which return owned values, property access
/// returns references into the source value. This requires three separate
/// access methods for shared, mutable, and owned access patterns.
pub(crate) struct PropertyAccessInterface {
    /// Access a property by shared reference.
    pub shared_access:
        for<'a> fn(PropertyAccessCallContext, &'a AnyValue) -> ExecutionResult<&'a AnyValue>,
    /// Access a property by mutable reference, optionally auto-creating if missing.
    pub mutable_access: for<'a> fn(
        PropertyAccessCallContext,
        &'a mut AnyValue,
        bool,
    ) -> ExecutionResult<&'a mut AnyValue>,
    /// Extract a property from an owned value.
    pub owned_access: fn(PropertyAccessCallContext, AnyValue) -> ExecutionResult<AnyValue>,
}

// ============================================================================
// Index Access Interface
// ============================================================================

/// Context provided to index access methods.
#[derive(Clone, Copy)]
pub(crate) struct IndexAccessCallContext<'a> {
    pub access: &'a IndexAccess,
}

/// Interface for index access on a type (e.g., `arr[0]` or `obj["key"]`).
///
/// Similar to property access, but the index is an evaluated expression
/// rather than a static identifier.
pub(crate) struct IndexAccessInterface {
    /// The ownership requirement for the index value.
    pub index_ownership: ArgumentOwnership,
    /// Access an element by shared reference.
    pub shared_access: for<'a> fn(
        IndexAccessCallContext,
        &'a AnyValue,
        Spanned<AnyValueRef>,
    ) -> ExecutionResult<&'a AnyValue>,
    /// Access an element by mutable reference, optionally auto-creating if missing.
    pub mutable_access: for<'a> fn(
        IndexAccessCallContext,
        &'a mut AnyValue,
        Spanned<AnyValueRef>,
        bool,
    ) -> ExecutionResult<&'a mut AnyValue>,
    /// Extract an element from an owned value.
    pub owned_access:
        fn(IndexAccessCallContext, AnyValue, Spanned<AnyValueRef>) -> ExecutionResult<AnyValue>,
}
