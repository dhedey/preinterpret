#![allow(unused)] // TODO: Remove when type resolution is implemented
use super::*;

pub(super) trait ResolvedTypeDetails {
    /// Whether the type can be transparently cloned.
    fn supports_copy(&self) -> bool;

    /// Resolves a method for this resolved type with the given arguments.
    fn resolve_method(&self, name: &str, num_arguments: usize) -> ExecutionResult<ResolvedMethod>;
}

pub(super) struct ResolvedType {
    inner: Box<dyn ResolvedTypeDetails>,
}

impl ResolvedTypeDetails for ResolvedType {
    fn supports_copy(&self) -> bool {
        self.inner.supports_copy()
    }

    fn resolve_method(&self, name: &str, num_arguments: usize) -> ExecutionResult<ResolvedMethod> {
        self.inner.resolve_method(name, num_arguments)
    }
}

pub(super) struct ResolvedMethod {
    // TODO: Add some reference to the code here
    object_location_kind: RequestedValueOwnership,
    parameter_location_kinds: Vec<RequestedValueOwnership>,
}

impl ResolvedMethod {
    fn execute(
        &self,
        object: ResolvedValue,
        parameters: Vec<ResolvedValue>,
    ) -> ExecutionResult<ResolvedValue> {
        unimplemented!("Method execution is not implemented yet");
    }
}
