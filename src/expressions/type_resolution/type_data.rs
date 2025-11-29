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
    fn resolve_binary_operation(
        &self,
        operation: &BinaryOperation,
    ) -> Option<BinaryOperationInterface>;
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

    fn resolve_binary_operation(
        &self,
        operation: &BinaryOperation,
    ) -> Option<BinaryOperationInterface> {
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

    fn assert_output_type<T: ResolvableOutput>() {}

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
    fn resolve_own_binary_operation(
        _operation: &BinaryOperation,
    ) -> Option<BinaryOperationInterface> {
        None
    }
}

#[allow(unused)]
pub(crate) enum MethodInterface {
    Arity0 {
        method: fn(&mut MethodCallContext) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 0],
    },
    Arity1 {
        method: fn(&mut MethodCallContext, ResolvedValue) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 1],
    },
    /// 1 argument, 1 optional argument
    Arity1PlusOptional1 {
        method: fn(
            &mut MethodCallContext,
            ResolvedValue,
            Option<ResolvedValue>,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 2],
    },
    Arity2 {
        method: fn(
            &mut MethodCallContext,
            ResolvedValue,
            ResolvedValue,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 2],
    },
    Arity2PlusOptional1 {
        method: fn(
            &mut MethodCallContext,
            ResolvedValue,
            ResolvedValue,
            Option<ResolvedValue>,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 3],
    },
    Arity3 {
        method: fn(
            &mut MethodCallContext,
            ResolvedValue,
            ResolvedValue,
            ResolvedValue,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 3],
    },
    Arity3PlusOptional1 {
        method: fn(
            &mut MethodCallContext,
            ResolvedValue,
            ResolvedValue,
            ResolvedValue,
            Option<ResolvedValue>,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 4],
    },
    ArityAny {
        method: fn(&mut MethodCallContext, Vec<ResolvedValue>) -> ExecutionResult<ResolvedValue>,
        argument_ownership: Vec<ResolvedValueOwnership>,
    },
}

impl MethodInterface {
    pub(crate) fn execute(
        &self,
        arguments: Vec<ResolvedValue>,
        context: &mut MethodCallContext,
    ) -> ExecutionResult<ResolvedValue> {
        match self {
            MethodInterface::Arity0 { method, .. } => {
                if !arguments.is_empty() {
                    return context.output_span_range.type_err("Expected 0 arguments");
                }
                method(context)
            }
            MethodInterface::Arity1 { method, .. } => {
                match <[ResolvedValue; 1]>::try_from(arguments) {
                    Ok([a]) => method(context, a),
                    Err(_) => context.output_span_range.type_err("Expected 1 argument"),
                }
            }
            MethodInterface::Arity1PlusOptional1 { method, .. } => match arguments.len() {
                1 => {
                    let [a] = <[ResolvedValue; 1]>::try_from(arguments).ok().unwrap();
                    method(context, a, None)
                }
                2 => {
                    let [a, b] = <[ResolvedValue; 2]>::try_from(arguments).ok().unwrap();
                    method(context, a, Some(b))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 1 or 2 arguments"),
            },
            MethodInterface::Arity2 { method, .. } => {
                match <[ResolvedValue; 2]>::try_from(arguments) {
                    Ok([a, b]) => method(context, a, b),
                    Err(_) => context.output_span_range.type_err("Expected 2 arguments"),
                }
            }
            MethodInterface::Arity2PlusOptional1 { method, .. } => match arguments.len() {
                2 => {
                    let [a, b] = <[ResolvedValue; 2]>::try_from(arguments).ok().unwrap();
                    method(context, a, b, None)
                }
                3 => {
                    let [a, b, c] = <[ResolvedValue; 3]>::try_from(arguments).ok().unwrap();
                    method(context, a, b, Some(c))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 2 or 3 arguments"),
            },
            MethodInterface::Arity3 { method, .. } => {
                match <[ResolvedValue; 3]>::try_from(arguments) {
                    Ok([a, b, c]) => method(context, a, b, c),
                    Err(_) => context.output_span_range.type_err("Expected 3 arguments"),
                }
            }
            MethodInterface::Arity3PlusOptional1 { method, .. } => match arguments.len() {
                3 => {
                    let [a, b, c] = <[ResolvedValue; 3]>::try_from(arguments).ok().unwrap();
                    method(context, a, b, c, None)
                }
                4 => {
                    let [a, b, c, d] = <[ResolvedValue; 4]>::try_from(arguments).ok().unwrap();
                    method(context, a, b, c, Some(d))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 3 or 4 arguments"),
            },
            MethodInterface::ArityAny { method, .. } => method(context, arguments),
        }
    }

    /// Returns (argument_ownerships, required_argument_count)
    pub(crate) fn argument_ownerships(&self) -> (&[ResolvedValueOwnership], usize) {
        match self {
            MethodInterface::Arity0 {
                argument_ownership, ..
            } => (argument_ownership, 0),
            MethodInterface::Arity1 {
                argument_ownership, ..
            } => (argument_ownership, 1),
            MethodInterface::Arity1PlusOptional1 {
                argument_ownership, ..
            } => (argument_ownership, 1),
            MethodInterface::Arity2 {
                argument_ownership, ..
            } => (argument_ownership, 2),
            MethodInterface::Arity2PlusOptional1 {
                argument_ownership, ..
            } => (argument_ownership, 2),
            MethodInterface::Arity3 {
                argument_ownership, ..
            } => (argument_ownership, 3),
            MethodInterface::Arity3PlusOptional1 {
                argument_ownership, ..
            } => (argument_ownership, 3),
            MethodInterface::ArityAny {
                argument_ownership, ..
            } => (argument_ownership, 0),
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

pub(crate) struct BinaryOperationInterface {
    pub method: fn(
        BinaryOperationCallContext,
        ResolvedValue,
        ResolvedValue,
    ) -> ExecutionResult<ResolvedValue>,
    pub lhs_ownership: ResolvedValueOwnership,
    pub rhs_ownership: ResolvedValueOwnership,
}

impl BinaryOperationInterface {
    pub(crate) fn execute(
        &self,
        lhs: ResolvedValue,
        rhs: ResolvedValue,
        operation: &BinaryOperation,
    ) -> ExecutionResult<ResolvedValue> {
        let output_span_range = SpanRange::new_between(lhs.span_range(), rhs.span_range());
        (self.method)(
            BinaryOperationCallContext {
                operation,
                output_span_range,
            },
            lhs,
            rhs,
        )
    }

    pub(crate) fn lhs_ownership(&self) -> ResolvedValueOwnership {
        self.lhs_ownership
    }

    pub(crate) fn rhs_ownership(&self) -> ResolvedValueOwnership {
        self.rhs_ownership
    }
}
