use super::*;

pub(in crate::expressions) trait MethodResolver {
    /// Resolves a unary operation as a method interface for this type.
    fn resolve_method(&self, method_name: &str) -> Option<MethodInterface>;

    /// Resolves a unary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_unary_operation(
        &self,
        operation: &UnaryOperation,
    ) -> Option<UnaryOperationInterface>;

    /// Resolves a binary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_binary_operation(&self, operation: &BinaryOperation) -> Option<MethodInterface>;
}

impl<T: HierarchicalTypeData> MethodResolver for T {
    fn resolve_method(&self, method_name: &str) -> Option<MethodInterface> {
        match Self::resolve_own_method(method_name) {
            Some(method) => Some(method),
            None => Self::PARENT.and_then(|p| p.resolve_method(method_name)),
        }
    }

    fn resolve_unary_operation(
        &self,
        operation: &UnaryOperation,
    ) -> Option<UnaryOperationInterface> {
        match Self::resolve_own_unary_operation(operation) {
            Some(method) => Some(method),
            None => Self::PARENT.and_then(|p| p.resolve_unary_operation(operation)),
        }
    }

    fn resolve_binary_operation(&self, operation: &BinaryOperation) -> Option<MethodInterface> {
        match Self::resolve_own_binary_operation(operation) {
            Some(method) => Some(method),
            None => Self::PARENT.and_then(|p| p.resolve_binary_operation(operation)),
        }
    }
}

pub(crate) trait HierarchicalTypeData {
    type Parent: HierarchicalTypeData;
    const PARENT: Option<Self::Parent>;

    fn assert_first_argument<T: FromResolved<ValueType = Self>>() {}

    fn resolve_own_method(_method_name: &str) -> Option<MethodInterface> {
        None
    }

    /// Resolves a unary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_own_unary_operation(_operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
        None
    }

    /// Resolves a binary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_own_binary_operation(_operation: &BinaryOperation) -> Option<MethodInterface> {
        None
    }
}

#[allow(unused)]
pub(crate) enum MethodInterface {
    Arity0 {
        method: fn(MethodCallContext) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 0],
    },
    Arity1 {
        method: fn(MethodCallContext, ResolvedValue) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 1],
    },
    Arity2 {
        method:
            fn(MethodCallContext, ResolvedValue, ResolvedValue) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 2],
    },
    Arity3 {
        method: fn(
            MethodCallContext,
            ResolvedValue,
            ResolvedValue,
            ResolvedValue,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 3],
    },
    ArityAny {
        method: fn(MethodCallContext, Vec<ResolvedValue>) -> ExecutionResult<ResolvedValue>,
        argument_ownership: Vec<ResolvedValueOwnership>,
    },
}

impl MethodInterface {
    pub(crate) fn execute(
        &self,
        arguments: Vec<ResolvedValue>,
        context: MethodCallContext,
    ) -> ExecutionResult<ResolvedValue> {
        match self {
            MethodInterface::Arity0 { method, .. } => {
                if !arguments.is_empty() {
                    return context
                        .output_span_range
                        .execution_err("Expected 0 arguments");
                }
                method(context)
            }
            MethodInterface::Arity1 { method, .. } => {
                match <[ResolvedValue; 1]>::try_from(arguments) {
                    Ok([a]) => method(context, a),
                    Err(_) => context
                        .output_span_range
                        .execution_err("Expected 1 argument"),
                }
            }
            MethodInterface::Arity2 { method, .. } => {
                match <[ResolvedValue; 2]>::try_from(arguments) {
                    Ok([a, b]) => method(context, a, b),
                    Err(_) => context
                        .output_span_range
                        .execution_err("Expected 2 arguments"),
                }
            }
            MethodInterface::Arity3 { method, .. } => {
                match <[ResolvedValue; 3]>::try_from(arguments) {
                    Ok([a, b, c]) => method(context, a, b, c),
                    Err(_) => context
                        .output_span_range
                        .execution_err("Expected 3 arguments"),
                }
            }
            MethodInterface::ArityAny { method, .. } => method(context, arguments),
        }
    }

    pub(crate) fn argument_ownerships(&self) -> &[ResolvedValueOwnership] {
        match self {
            MethodInterface::Arity0 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::Arity1 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::Arity2 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::Arity3 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::ArityAny {
                argument_ownership, ..
            } => argument_ownership,
        }
    }
}

pub(crate) struct UnaryOperationInterface {
    pub method: fn(UnaryOperationCallContext, ResolvedValue) -> ExecutionResult<ResolvedValue>,
    pub argument_ownership: ResolvedValueOwnership,
}

impl UnaryOperationInterface {
    pub(crate) fn execute(
        &self,
        input: ResolvedValue,
        operation: &UnaryOperation,
    ) -> ExecutionResult<ResolvedValue> {
        let output_span_range = operation.output_span_range(input.span_range());
        (self.method)(
            UnaryOperationCallContext {
                operation,
                output_span_range,
            },
            input,
        )
    }

    pub(crate) fn argument_ownership(&self) -> ResolvedValueOwnership {
        self.argument_ownership
    }
}
