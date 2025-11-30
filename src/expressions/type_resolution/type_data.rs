use super::*;

pub(in crate::expressions) trait MethodResolver {
    /// Resolves a unary operation as a method interface for this type.
    fn resolve_method(&self, method_name: &str) -> Option<MethodInterface>;

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

    /// Resolves a compound assignment operation as a method interface for this type.
    #[allow(unused)]
    fn resolve_compound_assignment_operation(
        &self,
        operation: &CompoundAssignmentOperation,
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

    fn resolve_compound_assignment_operation(
        &self,
        operation: &CompoundAssignmentOperation,
    ) -> Option<BinaryOperationInterface> {
        match Self::resolve_own_compound_assignment_operation(operation) {
            Some(method) => Some(method),
            None => Self::PARENT.and_then(|p| p.resolve_compound_assignment_operation(operation)),
        }
    }
}

pub(crate) trait HierarchicalTypeData {
    type Parent: HierarchicalTypeData;
    const PARENT: Option<Self::Parent>;

    fn assert_first_argument<T: IsArgument<ValueType = Self>>() {}

    fn assert_output_type<T: IsReturnable>() {}

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
        operation: &BinaryOperation,
    ) -> Option<BinaryOperationInterface> {
        match operation {
            BinaryOperation::Paired(operation) => Self::resolve_paired_binary_operation(operation),
            BinaryOperation::Integer(operation) => {
                Self::resolve_integer_binary_operation(operation)
            }
        }
    }

    fn resolve_paired_binary_operation(
        _operation: &PairedBinaryOperation,
    ) -> Option<BinaryOperationInterface> {
        None
    }

    fn resolve_integer_binary_operation(
        _operation: &IntegerBinaryOperation,
    ) -> Option<BinaryOperationInterface> {
        None
    }

    #[allow(unused)]
    fn resolve_own_compound_assignment_operation(
        _operation: &CompoundAssignmentOperation,
    ) -> Option<BinaryOperationInterface> {
        None
    }
}

#[allow(unused)]
pub(crate) enum MethodInterface {
    Arity0 {
        method: fn(&mut MethodCallContext) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 0],
    },
    Arity1 {
        method: fn(&mut MethodCallContext, ArgumentValue) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 1],
    },
    /// 1 argument, 1 optional argument
    Arity1PlusOptional1 {
        method: fn(
            &mut MethodCallContext,
            ArgumentValue,
            Option<ArgumentValue>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 2],
    },
    Arity2 {
        method: fn(
            &mut MethodCallContext,
            ArgumentValue,
            ArgumentValue,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 2],
    },
    Arity2PlusOptional1 {
        method: fn(
            &mut MethodCallContext,
            ArgumentValue,
            ArgumentValue,
            Option<ArgumentValue>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 3],
    },
    Arity3 {
        method: fn(
            &mut MethodCallContext,
            ArgumentValue,
            ArgumentValue,
            ArgumentValue,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 3],
    },
    Arity3PlusOptional1 {
        method: fn(
            &mut MethodCallContext,
            ArgumentValue,
            ArgumentValue,
            ArgumentValue,
            Option<ArgumentValue>,
        ) -> ExecutionResult<ReturnedValue>,
        argument_ownership: [ArgumentOwnership; 4],
    },
    ArityAny {
        method: fn(&mut MethodCallContext, Vec<ArgumentValue>) -> ExecutionResult<ReturnedValue>,
        argument_ownership: Vec<ArgumentOwnership>,
    },
}

impl MethodInterface {
    pub(crate) fn execute(
        &self,
        arguments: Vec<ArgumentValue>,
        context: &mut MethodCallContext,
    ) -> ExecutionResult<ReturnedValue> {
        match self {
            MethodInterface::Arity0 { method, .. } => {
                if !arguments.is_empty() {
                    return context.output_span_range.type_err("Expected 0 arguments");
                }
                method(context)
            }
            MethodInterface::Arity1 { method, .. } => {
                match <[ArgumentValue; 1]>::try_from(arguments) {
                    Ok([a]) => method(context, a),
                    Err(_) => context.output_span_range.type_err("Expected 1 argument"),
                }
            }
            MethodInterface::Arity1PlusOptional1 { method, .. } => match arguments.len() {
                1 => {
                    let [a] = <[ArgumentValue; 1]>::try_from(arguments).ok().unwrap();
                    method(context, a, None)
                }
                2 => {
                    let [a, b] = <[ArgumentValue; 2]>::try_from(arguments).ok().unwrap();
                    method(context, a, Some(b))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 1 or 2 arguments"),
            },
            MethodInterface::Arity2 { method, .. } => {
                match <[ArgumentValue; 2]>::try_from(arguments) {
                    Ok([a, b]) => method(context, a, b),
                    Err(_) => context.output_span_range.type_err("Expected 2 arguments"),
                }
            }
            MethodInterface::Arity2PlusOptional1 { method, .. } => match arguments.len() {
                2 => {
                    let [a, b] = <[ArgumentValue; 2]>::try_from(arguments).ok().unwrap();
                    method(context, a, b, None)
                }
                3 => {
                    let [a, b, c] = <[ArgumentValue; 3]>::try_from(arguments).ok().unwrap();
                    method(context, a, b, Some(c))
                }
                _ => context
                    .output_span_range
                    .type_err("Expected 2 or 3 arguments"),
            },
            MethodInterface::Arity3 { method, .. } => {
                match <[ArgumentValue; 3]>::try_from(arguments) {
                    Ok([a, b, c]) => method(context, a, b, c),
                    Err(_) => context.output_span_range.type_err("Expected 3 arguments"),
                }
            }
            MethodInterface::Arity3PlusOptional1 { method, .. } => match arguments.len() {
                3 => {
                    let [a, b, c] = <[ArgumentValue; 3]>::try_from(arguments).ok().unwrap();
                    method(context, a, b, c, None)
                }
                4 => {
                    let [a, b, c, d] = <[ArgumentValue; 4]>::try_from(arguments).ok().unwrap();
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
    pub(crate) fn argument_ownerships(&self) -> (&[ArgumentOwnership], usize) {
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
    pub method: fn(UnaryOperationCallContext, ArgumentValue) -> ExecutionResult<ReturnedValue>,
    pub argument_ownership: ArgumentOwnership,
}

impl UnaryOperationInterface {
    pub(crate) fn execute(
        &self,
        input: ArgumentValue,
        operation: &UnaryOperation,
    ) -> ExecutionResult<ReturnedValue> {
        let output_span_range = operation.output_span_range(input.span_range());
        (self.method)(
            UnaryOperationCallContext {
                operation,
                output_span_range,
            },
            input,
        )
    }

    pub(crate) fn argument_ownership(&self) -> ArgumentOwnership {
        self.argument_ownership
    }
}

pub(crate) struct BinaryOperationInterface {
    pub method: fn(
        BinaryOperationCallContext,
        ArgumentValue,
        ArgumentValue,
    ) -> ExecutionResult<ReturnedValue>,
    pub lhs_ownership: ArgumentOwnership,
    pub rhs_ownership: ArgumentOwnership,
}

impl BinaryOperationInterface {
    pub(crate) fn execute(
        &self,
        lhs: ArgumentValue,
        rhs: ArgumentValue,
        operation: &BinaryOperation,
    ) -> ExecutionResult<ReturnedValue> {
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

    pub(crate) fn lhs_ownership(&self) -> ArgumentOwnership {
        self.lhs_ownership
    }

    pub(crate) fn rhs_ownership(&self) -> ArgumentOwnership {
        self.rhs_ownership
    }
}
