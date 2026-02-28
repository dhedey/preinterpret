use super::*;

use std::borrow::Borrow;

pub(super) enum VariableState {
    Uninitialized,
    Value(VariableContent),
    Finished,
}

/// Cheaply clonable, inactive content of a variable.
/// By inactive, we mean that the shared/mutable aliasing invariants are not required to hold.
/// Before use, they have to be activated. This allows more flexibility over how references
/// can be used and held, without compromising safety.
#[derive(Clone)]
pub(crate) enum VariableContent {
    // Owned, but possibly with pre-existing references
    Referenceable(Referenceable),
    Shared(InactiveShared<AnyValue>),
    Mutable(InactiveMutable<AnyValue>),
}

impl VariableContent {
    fn into_owned_as_only_owner(self) -> Result<Owned<AnyValue>, VariableContent> {
        match self {
            VariableContent::Referenceable(data) => match data.try_into_inner() {
                Ok(owned) => Ok(owned),
                Err(original) => Err(VariableContent::Referenceable(original)),
            },
            other => Err(other),
        }
    }
}

const UNITIALIZED_ERR: &str = "Cannot resolve uninitialized variable. This shouldn't be possible, because all variables are set on first use.";
const FINISHED_ERR: &str = "Cannot resolve finished variable. This shouldn't be possible, because is_final should be marked correctly. If you see this error, please report a bug to preinterpret on github with a reproduction case.";

impl VariableState {
    pub(crate) fn define(&mut self, value: VariableContent) {
        match self {
            content @ VariableState::Uninitialized => {
                *content = VariableState::Value(value);
            }
            VariableState::Value(_) => panic!("Cannot define existing variable"),
            VariableState::Finished => panic!("Cannot define finished variable"),
        }
    }

    pub(crate) fn resolve_content(
        &mut self,
        is_final: bool,
        blocked_from_mutation: Option<MutationBlockReason>,
    ) -> VariableContent {
        if is_final && blocked_from_mutation.is_none() {
            let content = std::mem::replace(self, VariableState::Finished);
            match content {
                VariableState::Uninitialized => panic!("{}", UNITIALIZED_ERR),
                VariableState::Value(content) => content,
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        } else {
            match self {
                VariableState::Uninitialized => panic!("{}", UNITIALIZED_ERR),
                VariableState::Value(content) => content.clone(),
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        }
    }

    pub(crate) fn resolve(
        &mut self,
        variable_span: Span,
        is_final: bool,
        ownership: RequestedOwnership,
        blocked_from_mutation: Option<MutationBlockReason>,
    ) -> FunctionResult<Spanned<LateBoundValue>> {
        let span_range = variable_span.span_range();

        // If blocked from mutation, we technically could allow is_final to work and
        // return a fully owned value without observable mutation,
        // but it's likely confusingly inconsistent, so it's better to just block it entirely.
        let content = if is_final && blocked_from_mutation.is_none() {
            let content = std::mem::replace(self, VariableState::Finished);
            match content {
                VariableState::Uninitialized => panic!("{}", UNITIALIZED_ERR),
                VariableState::Value(content) => match content.into_owned_as_only_owner() {
                    Ok(owned) => {
                        if matches!(
                            ownership,
                            RequestedOwnership::Concrete(ArgumentOwnership::Assignee { .. })
                        ) {
                            return variable_span.control_flow_err("The final usage of a variable cannot be assigned to. You can use `let _ = ..` to discard a value.");
                        }
                        return Ok(Spanned(
                            LateBoundValue::Owned(LateBoundOwnedValue {
                                owned,
                                is_from_last_use: true,
                            }),
                            span_range,
                        ));
                    }
                    // It's currently referenced elsewhere, proceed with normal late-bound resolution.
                    // e.g.
                    // * `let x = %[]; x.assert_eq(x, %[]);` - the final `x` resolves to a shared reference
                    // * `let x = %[]; x.assert_eq(x + %[], %[]);` - errors because the final `x` is shared but it needs to be owned
                    // Or, it could be captured somewhere else, e.g. in a closure:
                    // * `let x = %[]; let f = || x; let y = x;` - the final `x` resolves to a shared reference
                    Err(content) => content,
                },
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        } else {
            match self {
                VariableState::Uninitialized => panic!("{}", UNITIALIZED_ERR),
                VariableState::Value(content) => content.clone(),
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        };
        let binding = VariableBinding {
            content,
            variable_span,
        };
        let resolved = match ownership {
            RequestedOwnership::LateBound => binding.into_late_bound(),
            RequestedOwnership::Concrete(ownership) => match ownership {
                ArgumentOwnership::Owned => binding.into_transparently_cloned().map(|owned| {
                    LateBoundValue::Owned(LateBoundOwnedValue {
                        owned,
                        is_from_last_use: false,
                    })
                }),
                ArgumentOwnership::Shared => binding
                    .into_shared()
                    .map(CopyOnWrite::shared_in_place_of_shared)
                    .map(LateBoundValue::CopyOnWrite),
                ArgumentOwnership::Assignee { .. } => {
                    binding.into_mut().map(LateBoundValue::Mutable)
                }
                ArgumentOwnership::Mutable => binding.into_mut().map(LateBoundValue::Mutable),
                ArgumentOwnership::CopyOnWrite | ArgumentOwnership::AsIs => binding
                    .into_shared()
                    .map(CopyOnWrite::shared_in_place_of_shared)
                    .map(LateBoundValue::CopyOnWrite),
            },
        };
        let late_bound = if let Some(mutation_block_reason) = blocked_from_mutation {
            match resolved {
                Ok(LateBoundValue::Mutable(mutable)) => {
                    let reason_not_mutable = variable_span
                        .syn_error(mutation_block_reason.error_message("mutate this variable"));
                    Ok(LateBoundValue::Shared(LateBoundSharedValue::new(
                        mutable.into_shared(),
                        reason_not_mutable,
                    )))
                }
                x => x,
            }
        } else {
            resolved
        }?;
        Ok(Spanned(late_bound, span_range))
    }
}

#[derive(Clone)]
pub(crate) struct VariableBinding {
    content: VariableContent,
    variable_span: Span,
}

impl VariableBinding {
    /// Gets the cloned expression value
    /// This only works if the value can be transparently cloned
    pub(crate) fn into_transparently_cloned(self) -> FunctionResult<AnyValue> {
        let span_range = self.variable_span.span_range();
        let shared = self.into_shared()?;
        let value = shared.as_ref().try_transparent_clone(span_range)?;
        Ok(value)
    }

    fn into_mut(self) -> FunctionResult<AnyValueMutable> {
        let span = self.variable_span.span_range();
        match self.content {
            VariableContent::Referenceable(referenceable) => referenceable.new_active_mutable(span),
            VariableContent::Mutable(inactive) => inactive.activate(span),
            VariableContent::Shared(_) => self
                .variable_span
                .ownership_err(SHARED_TO_MUTABLE_ERROR_MESSAGE),
        }
    }

    fn into_shared(self) -> FunctionResult<AnyValueShared> {
        let span = self.variable_span.span_range();
        match self.content {
            VariableContent::Referenceable(referenceable) => referenceable.new_active_shared(span),
            VariableContent::Mutable(inactive) => inactive.into_shared().activate(span),
            VariableContent::Shared(inactive) => inactive.activate(span),
        }
    }

    fn into_late_bound(self) -> FunctionResult<LateBoundValue> {
        let span = self.variable_span.span_range();
        match self.content {
            VariableContent::Referenceable(referenceable) => {
                match referenceable.new_active_mutable(span) {
                    Ok(mutable) => Ok(LateBoundValue::Mutable(mutable)),
                    Err(err) => {
                        // Mutable failed, try shared
                        let shared = referenceable.new_active_shared(span)?;
                        Ok(LateBoundValue::Shared(LateBoundSharedValue::new(
                            shared,
                            err.expect_ownership_error()?,
                        )))
                    }
                }
            }
            VariableContent::Mutable(inactive) => {
                Ok(LateBoundValue::Mutable(inactive.activate(span)?))
            }
            VariableContent::Shared(inactive) => {
                Ok(LateBoundValue::Shared(LateBoundSharedValue::new(
                    inactive.activate(span)?,
                    self.variable_span
                        .syn_error(SHARED_TO_MUTABLE_ERROR_MESSAGE),
                )))
            }
        }
    }
}

static SHARED_TO_MUTABLE_ERROR_MESSAGE: &str = "Cannot mutate a non-mutable reference";

pub(crate) struct LateBoundOwnedValue {
    pub(crate) owned: AnyValueOwned,
    pub(crate) is_from_last_use: bool,
}

/// A shared value where mutable access failed for a specific reason
pub(crate) struct LateBoundSharedValue {
    pub(crate) shared: AnyValueShared,
    pub(crate) reason_not_mutable: syn::Error,
}

impl LateBoundSharedValue {
    pub(crate) fn new(shared: AnyValueShared, reason_not_mutable: syn::Error) -> Self {
        Self {
            shared,
            reason_not_mutable,
        }
    }
}

/// Universal value type that can resolve to any concrete ownership type.
///
/// Sometimes, a value can be accessed, but we don't yet know *how* we need to access it.
/// In this case, we attempt to load it with the most powerful access we can have, and convert it later.
///
/// ## Example of requirement
/// For example, if we have `x[a].method()`, we first need to resolve the type of `x[a]` to know
/// whether the method needs `x[a]` to be a shared reference, mutable reference or an owned value.
///
/// So instead, we take the most powerful access we can have for `x[a]`, and convert it later.
pub(crate) enum LateBoundValue {
    /// An owned value that can be converted to any ownership type
    Owned(LateBoundOwnedValue),
    /// A copy-on-write value that can be converted to an owned value
    CopyOnWrite(AnyValueCopyOnWrite),
    /// A mutable reference
    Mutable(AnyValueMutable),
    /// A shared reference where mutable access failed for a specific reason
    Shared(LateBoundSharedValue),
}

impl Spanned<LateBoundValue> {
    pub(crate) fn resolve(self, ownership: ArgumentOwnership) -> FunctionResult<ArgumentValue> {
        ownership.map_from_late_bound(self)
    }
}

impl LateBoundValue {
    /// Maps the late-bound value through the appropriate accessor function.
    ///
    /// If the mutable mapping fails with a retryable reason, we fall back to `map_shared`.
    /// This allows operations that don't need mutable access (like reading a non-existent
    /// key from an object) to still work, with the mutable error preserved as `reason_not_mutable`.
    pub(crate) fn map_any(
        self,
        map_shared: impl FnOnce(AnyValueShared) -> FunctionResult<AnyValueShared>,
        map_mutable: impl FnOnce(
            AnyValueMutable,
        ) -> Result<AnyValueMutable, (FunctionError, AnyValueMutable)>,
        map_owned: impl FnOnce(AnyValueOwned) -> FunctionResult<AnyValueOwned>,
    ) -> FunctionResult<Self> {
        Ok(match self {
            LateBoundValue::Owned(owned) => LateBoundValue::Owned(LateBoundOwnedValue {
                owned: map_owned(owned.owned)?,
                is_from_last_use: owned.is_from_last_use,
            }),
            LateBoundValue::CopyOnWrite(copy_on_write) => {
                LateBoundValue::CopyOnWrite(copy_on_write.map(map_shared, map_owned)?)
            }
            LateBoundValue::Mutable(mutable) => match map_mutable(mutable) {
                Ok(mapped) => LateBoundValue::Mutable(mapped),
                Err((error, recovered_mutable)) => {
                    // Check if this error can be caught for fallback to shared access
                    let reason_not_mutable = error.into_caught_mutable_map_attempt_error()?;
                    let shared = recovered_mutable.into_shared();
                    let mapped_shared = map_shared(shared)?;
                    LateBoundValue::Shared(LateBoundSharedValue::new(
                        mapped_shared,
                        reason_not_mutable,
                    ))
                }
            },
            LateBoundValue::Shared(LateBoundSharedValue {
                shared,
                reason_not_mutable,
            }) => LateBoundValue::Shared(LateBoundSharedValue::new(
                map_shared(shared)?,
                reason_not_mutable,
            )),
        })
    }

    pub(crate) fn as_value(&self) -> &AnyValue {
        match self {
            LateBoundValue::Owned(owned) => &owned.owned,
            LateBoundValue::CopyOnWrite(cow) => cow.as_ref(),
            LateBoundValue::Mutable(mutable) => mutable.as_ref(),
            LateBoundValue::Shared(shared) => shared.shared.as_ref(),
        }
    }
}

impl Deref for LateBoundValue {
    type Target = AnyValue;

    fn deref(&self) -> &Self::Target {
        self.as_value()
    }
}

// ============================================================================
// CopyOnWrite
// ============================================================================

/// Copy-on-write value that can be either owned or shared
pub(crate) struct CopyOnWrite<T: 'static> {
    pub(crate) inner: CopyOnWriteInner<T>,
}

pub(crate) enum CopyOnWriteInner<T: 'static> {
    /// An owned value that can be used directly
    Owned(Owned<T>),
    /// For use when the CopyOnWrite value effectively represents the owned value (post-clone).
    /// In this case, returning a Cow is just an optimization and we can always clone infallibly.
    SharedWithInfallibleCloning(Shared<T>),
    /// For use when the CopyOnWrite value represents a pre-cloned read-only value.
    /// A transparent clone may fail in this case at use time.
    SharedWithTransparentCloning(Shared<T>),
}

impl<T: 'static> CopyOnWrite<T> {
    pub(crate) fn shared_in_place_of_owned(shared: Shared<T>) -> Self {
        Self {
            inner: CopyOnWriteInner::SharedWithInfallibleCloning(shared),
        }
    }

    pub(crate) fn shared_in_place_of_shared(shared: Shared<T>) -> Self {
        Self {
            inner: CopyOnWriteInner::SharedWithTransparentCloning(shared),
        }
    }

    pub(crate) fn owned(owned: Owned<T>) -> Self {
        Self {
            inner: CopyOnWriteInner::Owned(owned),
        }
    }

    #[allow(unused)]
    pub(crate) fn extract_owned<U>(self, map: impl FnOnce(T) -> Result<U, T>) -> Result<U, Self> {
        match self.inner {
            CopyOnWriteInner::Owned(value) => match map(value) {
                Ok(mapped) => Ok(mapped),
                Err(other) => Err(Self {
                    inner: CopyOnWriteInner::Owned(other),
                }),
            },
            other => Err(Self { inner: other }),
        }
    }

    pub(crate) fn acts_as_shared_reference(&self) -> bool {
        match &self.inner {
            CopyOnWriteInner::Owned { .. } => false,
            CopyOnWriteInner::SharedWithInfallibleCloning { .. } => false,
            CopyOnWriteInner::SharedWithTransparentCloning { .. } => true,
        }
    }

    pub(crate) fn map<S>(
        self,
        map_shared: impl FnOnce(Shared<T>) -> FunctionResult<Shared<S>>,
        map_owned: impl FnOnce(Owned<T>) -> FunctionResult<Owned<S>>,
    ) -> FunctionResult<CopyOnWrite<S>> {
        let inner = match self.inner {
            CopyOnWriteInner::Owned(owned) => CopyOnWriteInner::Owned(map_owned(owned)?),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                CopyOnWriteInner::SharedWithInfallibleCloning(map_shared(shared)?)
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                CopyOnWriteInner::SharedWithTransparentCloning(map_shared(shared)?)
            }
        };
        Ok(CopyOnWrite { inner })
    }

    pub(crate) fn map_into<U>(
        self,
        map_shared: impl FnOnce(Shared<T>) -> U,
        map_owned: impl FnOnce(Owned<T>) -> U,
    ) -> U {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => map_owned(owned),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => map_shared(shared),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => map_shared(shared),
        }
    }

    /// Deactivates this copy-on-write value, releasing any borrow.
    /// Returns a `InactiveCopyOnWrite` which can be cloned and later re-activated.
    pub(crate) fn deactivate(self) -> InactiveCopyOnWrite<T> {
        let inner = match self.inner {
            CopyOnWriteInner::Owned(owned) => InactiveCopyOnWriteInner::Owned(owned),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                InactiveCopyOnWriteInner::SharedWithInfallibleCloning(shared.deactivate())
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                InactiveCopyOnWriteInner::SharedWithTransparentCloning(shared.deactivate())
            }
        };
        InactiveCopyOnWrite { inner }
    }
}

/// A disabled copy-on-write value that can be safely cloned and dropped.
pub(crate) struct InactiveCopyOnWrite<T: 'static> {
    inner: InactiveCopyOnWriteInner<T>,
}

enum InactiveCopyOnWriteInner<T: 'static> {
    Owned(Owned<T>),
    SharedWithInfallibleCloning(InactiveShared<T>),
    SharedWithTransparentCloning(InactiveShared<T>),
}

impl<T: 'static + Clone> Clone for InactiveCopyOnWrite<T> {
    fn clone(&self) -> Self {
        let inner = match &self.inner {
            InactiveCopyOnWriteInner::Owned(owned) => {
                InactiveCopyOnWriteInner::Owned(owned.clone())
            }
            InactiveCopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                InactiveCopyOnWriteInner::SharedWithInfallibleCloning(shared.clone())
            }
            InactiveCopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                InactiveCopyOnWriteInner::SharedWithTransparentCloning(shared.clone())
            }
        };
        Self { inner }
    }
}

impl<T: 'static> InactiveCopyOnWrite<T> {
    /// Re-activates this inactive copy-on-write value by re-acquiring any borrow.
    pub(crate) fn activate(self, span: SpanRange) -> FunctionResult<CopyOnWrite<T>> {
        let inner = match self.inner {
            InactiveCopyOnWriteInner::Owned(owned) => CopyOnWriteInner::Owned(owned),
            InactiveCopyOnWriteInner::SharedWithInfallibleCloning(inactive) => {
                CopyOnWriteInner::SharedWithInfallibleCloning(inactive.activate(span)?)
            }
            InactiveCopyOnWriteInner::SharedWithTransparentCloning(inactive) => {
                CopyOnWriteInner::SharedWithTransparentCloning(inactive.activate(span)?)
            }
        };
        Ok(CopyOnWrite { inner })
    }
}

impl<X: IsValueContent + Clone> IsValueContent for CopyOnWrite<X> {
    type Type = X::Type;
    type Form = BeCopyOnWrite;
}

impl<X: Clone + Sized> IntoValueContent<'static> for CopyOnWrite<X>
where
    X: IsValueContent<Form = BeOwned> + IsSelfValueContent<'static> + IntoValueContent<'static>,
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
{
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => {
                AnyLevelCopyOnWrite::<X::Type>::Owned(owned).into_copy_on_write()
            }
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                AnyLevelCopyOnWrite::<X::Type>::SharedWithInfallibleCloning(shared.into_content())
                    .into_copy_on_write()
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                AnyLevelCopyOnWrite::<X::Type>::SharedWithTransparentCloning(shared.into_content())
                    .into_copy_on_write()
            }
        }
    }
}

impl<T> AsRef<T> for CopyOnWrite<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> Deref for CopyOnWrite<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self.inner {
            CopyOnWriteInner::Owned(ref owned) => (*owned).borrow(),
            CopyOnWriteInner::SharedWithInfallibleCloning(ref shared) => shared.as_ref(),
            CopyOnWriteInner::SharedWithTransparentCloning(ref shared) => shared.as_ref(),
        }
    }
}

impl CopyOnWrite<AnyValue> {
    /// Converts to owned, cloning if necessary
    pub(crate) fn clone_to_owned_infallible(self) -> AnyValueOwned {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => owned,
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => shared.infallible_clone(),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared.infallible_clone(),
        }
    }

    /// Converts to owned, using transparent clone for shared values where cloning was not requested
    pub(crate) fn clone_to_owned_transparently(
        self,
        span: SpanRange,
    ) -> FunctionResult<AnyValueOwned> {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => Ok(owned),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => Ok(shared.infallible_clone()),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                let value = shared.as_ref().try_transparent_clone(span)?;
                Ok(value)
            }
        }
    }

    /// Converts to shared reference
    pub(crate) fn into_shared(self, span: SpanRange) -> AnyValueShared {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => AnyValueShared::new_from_owned(owned, None, span),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => shared,
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared,
        }
    }
}
