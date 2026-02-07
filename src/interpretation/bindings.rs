use super::*;

use std::borrow::{Borrow, ToOwned};
use std::rc::Rc;

pub(super) enum VariableState {
    Uninitialized,
    Value(VariableContent),
    Finished,
}

/// Cheaply clonable, non-active content of a variable.
/// By non-active, we mean that the shared/mutable invariants are not required to hold.
/// Before use, they have to be activated. This allows more flexibility over how references
/// can be used and held, without compromising safety.
#[derive(Clone)]
pub(crate) enum VariableContent {
    // Owned, but possibly with pre-existing references
    Referenceable(Referenceable<AnyValue>),
    Shared(DisabledShared<AnyValue>),
    Mutable(DisabledMutable<AnyValue>),
}

impl VariableContent {
    fn into_owned_as_only_owner(self) -> Result<Owned<AnyValue>, VariableContent> {
        match self {
            VariableContent::Referenceable(data) => match Rc::try_unwrap(data) {
                Ok(unique) => Ok(unique.into_inner()),
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

#[allow(unused)]
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
        match self.content {
            VariableContent::Referenceable(referenceable) => {
                let inner = MutableSubRcRefCell::new(referenceable).map_err(|_| {
                    self.variable_span
                        .ownership_error::<FunctionError>(MUTABLE_ERROR_MESSAGE)
                })?;
                Ok(Mutable(inner))
            }
            VariableContent::Mutable(disabled) => disabled.enable(self.variable_span.span_range()),
            VariableContent::Shared(shared) => self
                .variable_span
                .ownership_err(SHARED_TO_MUTABLE_ERROR_MESSAGE),
        }
    }

    fn into_shared(self) -> FunctionResult<AnyValueShared> {
        match self.content {
            VariableContent::Referenceable(referenceable) => {
                let inner = SharedSubRcRefCell::new(referenceable).map_err(|_| {
                    self.variable_span
                        .ownership_error::<FunctionError>(SHARED_ERROR_MESSAGE)
                })?;
                Ok(Shared(inner))
            }
            VariableContent::Mutable(mutable) => mutable
                .into_shared()
                .enable(self.variable_span.span_range()),
            VariableContent::Shared(shared) => shared.enable(self.variable_span.span_range()),
        }
    }

    fn into_late_bound(self) -> FunctionResult<LateBoundValue> {
        match self.content {
            VariableContent::Referenceable(referenceable) => {
                match MutableSubRcRefCell::new(referenceable) {
                    Ok(mutable) => Ok(LateBoundValue::Mutable(Mutable(mutable))),
                    Err(referenceable) => {
                        let shared = SharedSubRcRefCell::new(referenceable).map_err(|_| {
                            self.variable_span
                                .ownership_error::<FunctionError>(SHARED_ERROR_MESSAGE)
                        })?;
                        Ok(LateBoundValue::Shared(LateBoundSharedValue::new(
                            Shared(shared),
                            self.variable_span.syn_error(SHARED_ERROR_MESSAGE),
                        )))
                    }
                }
            }
            VariableContent::Mutable(mutable) => Ok(LateBoundValue::Mutable(
                mutable.enable(self.variable_span.span_range())?,
            )),
            VariableContent::Shared(shared) => {
                Ok(LateBoundValue::Shared(LateBoundSharedValue::new(
                    shared.enable(self.variable_span.span_range())?,
                    self.variable_span
                        .syn_error(SHARED_TO_MUTABLE_ERROR_MESSAGE),
                )))
            }
        }
    }
}

pub(crate) struct LateBoundOwnedValue {
    pub(crate) owned: AnyValueOwned,
    pub(crate) is_from_last_use: bool,
}

/// A shared value where mutable access failed for a specific reason
pub(crate) struct LateBoundSharedValue {
    pub(crate) shared: SharedValue,
    pub(crate) reason_not_mutable: syn::Error,
}

impl LateBoundSharedValue {
    pub(crate) fn new(shared: SharedValue, reason_not_mutable: syn::Error) -> Self {
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
    CopyOnWrite(CopyOnWriteValue),
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
        map_shared: impl FnOnce(AnyValueShared) -> ExecutionResult<AnyValueShared>,
        map_mutable: impl FnOnce(
            AnyValueMutable,
        )
            -> Result<AnyValueMutable, (ExecutionInterrupt, AnyValueMutable)>,
        map_owned: impl FnOnce(AnyValueOwned) -> ExecutionResult<AnyValueOwned>,
    ) -> ExecutionResult<Self> {
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

pub(crate) type AssigneeValue = AnyValueAssignee;

/// A binding of a unique (mutable) reference to a value.
/// See [`ArgumentOwnership::Assignee`] for more details.
///
/// If you need span information, wrap with `Spanned<Assignee<T>>`.
pub(crate) struct Assignee<T: 'static + ?Sized>(pub Mutable<T>);

impl AssigneeValue {
    pub(crate) fn set(&mut self, content: impl IntoAnyValue) {
        *self.0 .0 = content.into_any_value();
    }
}

impl<X: IsValueContent> IsValueContent for Assignee<X> {
    type Type = X::Type;
    type Form = BeAssignee;
}

impl<X: IsSelfValueContent<'static>> IntoValueContent<'static> for Assignee<X>
where
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
    X::Form: IsHierarchicalForm,
    X::Form: LeafAsMutForm,
{
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        self.0
             .0
            .replace(|inner, emplacer| inner.as_mut_value().into_assignee(emplacer))
    }
}

impl<T: 'static + ?Sized> Deref for Assignee<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static + ?Sized> DerefMut for Assignee<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A simple wrapper for a mutable reference to a value.
///
/// Can be destructured as: `Mutable(cell): Mutable<T>`
///
/// If you need span information, wrap with `Spanned<Mutable<T>>`.
pub(crate) struct Mutable<T: 'static + ?Sized>(pub(crate) MutableSubRcRefCell<AnyValue, T>);

impl<T: ?Sized> Mutable<T> {
    pub(crate) fn into_shared(self) -> Shared<T> {
        Shared(self.0.into_shared())
    }

    #[allow(unused)]
    pub(crate) fn map<V: ?Sized>(
        self,
        value_map: impl for<'a> FnOnce(&'a mut T) -> &'a mut V,
    ) -> Mutable<V> {
        Mutable(self.0.map(value_map))
    }

    /// Maps the mutable reference, returning the error and original reference on failure.
    pub(crate) fn try_map<V: ?Sized, E>(
        self,
        value_map: impl for<'a> FnOnce(&'a mut T) -> Result<&'a mut V, E>,
    ) -> Result<Mutable<V>, (E, Mutable<T>)> {
        match self.0.try_map(value_map) {
            Ok(mapped) => Ok(Mutable(mapped)),
            Err((e, original)) => Err((e, Mutable(original))),
        }
    }

    /// Disables this mutable reference, releasing the borrow on the RefCell.
    /// Returns a `DisabledMutable` which can be cloned and later re-enabled.
    pub(crate) fn disable(self) -> DisabledMutable<T> {
        DisabledMutable(self.0.disable())
    }
}

/// A disabled mutable reference that can be safely cloned and dropped.
pub(crate) struct DisabledMutable<T: 'static + ?Sized>(
    pub(crate) DisabledMutableSubRcRefCell<AnyValue, T>,
);

impl<T: ?Sized> Clone for DisabledMutable<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> DisabledMutable<T> {
    /// Re-enables this disabled mutable reference by re-acquiring the borrow.
    pub(crate) fn enable(self, span: SpanRange) -> FunctionResult<Mutable<T>> {
        self.0
            .enable()
            .map(Mutable)
            .map_err(|_| span.ownership_error(MUTABLE_ERROR_MESSAGE))
    }

    pub(crate) fn into_shared(self) -> DisabledShared<T> {
        DisabledShared(self.0.into_shared())
    }
}

pub(crate) static MUTABLE_ERROR_MESSAGE: &str =
    "The variable cannot be modified as it is already being modified";
pub(crate) static SHARED_TO_MUTABLE_ERROR_MESSAGE: &str =
    "The variable cannot be modified as it is a shared reference";

impl Spanned<AnyValueMutable> {
    pub(crate) fn transparent_clone(&self) -> FunctionResult<AnyValue> {
        let value = self.0.as_ref().try_transparent_clone(self.1)?;
        Ok(value)
    }
}

impl AnyValueMutable {
    pub(crate) fn new_from_owned(value: AnyValue) -> Self {
        Mutable(MutableSubRcRefCell::new_from_owned(value))
    }
}

impl<X: IsValueContent> IsValueContent for Mutable<X> {
    type Type = X::Type;
    type Form = BeMutable;
}

impl<X: IsSelfValueContent<'static>> IntoValueContent<'static> for Mutable<X>
where
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
    X::Form: IsHierarchicalForm,
    X::Form: LeafAsMutForm,
{
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        self.0
            .replace(|inner, emplacer| inner.as_mut_value().into_mutable(emplacer))
    }
}

impl<T: ?Sized> AsMut<T> for Mutable<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized> AsRef<T> for Mutable<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> Deref for Mutable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for Mutable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) type SharedValue = AnyValueShared;

/// A simple wrapper for a shared (immutable) reference to a value.
///
/// Can be destructured as: `Shared(cell): Shared<T>`
///
/// If you need span information, wrap with `Spanned<Shared<T>>`.
pub(crate) struct Shared<T: 'static + ?Sized>(pub(crate) SharedSubRcRefCell<AnyValue, T>);

#[allow(unused)]
impl<T: ?Sized> Shared<T> {
    pub(crate) fn clone(this: &Shared<T>) -> Self {
        Shared(SharedSubRcRefCell::clone(&this.0))
    }

    /// Maps the shared reference, returning the error and original reference on failure.
    pub(crate) fn try_map<V: ?Sized, E>(
        self,
        value_map: impl for<'a> FnOnce(&'a T) -> Result<&'a V, E>,
    ) -> Result<Shared<V>, (E, Shared<T>)> {
        match self.0.try_map(value_map) {
            Ok(mapped) => Ok(Shared(mapped)),
            Err((e, original)) => Err((e, Shared(original))),
        }
    }

    pub(crate) fn map<V: ?Sized>(self, value_map: impl FnOnce(&T) -> &V) -> Shared<V> {
        Shared(self.0.map(value_map))
    }

    /// Disables this shared reference, releasing the borrow on the RefCell.
    /// Returns a `DisabledShared` which can be cloned and later re-enabled.
    pub(crate) fn disable(self) -> DisabledShared<T> {
        DisabledShared(self.0.disable())
    }
}

/// A disabled shared reference that can be safely cloned and dropped.
pub(crate) struct DisabledShared<T: 'static + ?Sized>(
    pub(crate) DisabledSharedSubRcRefCell<AnyValue, T>,
);

impl<T: ?Sized> Clone for DisabledShared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> DisabledShared<T> {
    /// Re-enables this disabled shared reference by re-acquiring the borrow.
    pub(crate) fn enable(self, span: SpanRange) -> FunctionResult<Shared<T>> {
        self.0
            .enable()
            .map(Shared)
            .map_err(|_| span.ownership_error(SHARED_ERROR_MESSAGE))
    }
}

pub(crate) static SHARED_ERROR_MESSAGE: &str =
    "The variable cannot be read as it is already being modified";

impl Spanned<AnyValueShared> {
    pub(crate) fn transparent_clone(&self) -> FunctionResult<AnyValue> {
        let value = self.0.as_ref().try_transparent_clone(self.1)?;
        Ok(value)
    }
}

impl AnyValueShared {
    pub(crate) fn new_from_owned(value: AnyValue) -> Self {
        Shared(SharedSubRcRefCell::new_from_owned(value))
    }

    pub(crate) fn infallible_clone(&self) -> AnyValue {
        self.0.clone()
    }
}

impl<X: IsValueContent> IsValueContent for Shared<X> {
    type Type = X::Type;
    type Form = BeShared;
}

impl<X: IsSelfValueContent<'static>> IntoValueContent<'static> for Shared<X>
where
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
    X::Form: IsHierarchicalForm,
    X::Form: LeafAsRefForm,
{
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        self.0
            .replace(|inner, emplacer| inner.as_ref_value().into_shared(emplacer))
    }
}

impl<T: ?Sized> AsRef<T> for Shared<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Copy-on-write value that can be either owned or shared
pub(crate) struct CopyOnWrite<T: 'static + ToOwned + ?Sized> {
    pub(crate) inner: CopyOnWriteInner<T>,
}

pub(crate) enum CopyOnWriteInner<T: 'static + ToOwned + ?Sized> {
    /// An owned value that can be used directly
    Owned(Owned<T::Owned>),
    /// For use when the CopyOnWrite value effectively represents the owned value (post-clone).
    /// In this case, returning a Cow is just an optimization and we can always clone infallibly.
    SharedWithInfallibleCloning(Shared<T>),
    /// For use when the CopyOnWrite value represents a pre-cloned read-only value.
    /// A transparent clone may fail in this case at use time.
    SharedWithTransparentCloning(Shared<T>),
}

impl<T: 'static + ToOwned + ?Sized> CopyOnWrite<T> {
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

    pub(crate) fn owned(owned: Owned<T::Owned>) -> Self {
        Self {
            inner: CopyOnWriteInner::Owned(owned),
        }
    }

    #[allow(unused)]
    pub(crate) fn extract_owned<U>(
        self,
        map: impl FnOnce(T::Owned) -> Result<U, T::Owned>,
    ) -> Result<U, Self> {
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

    pub(crate) fn map<O: ToOwned + ?Sized>(
        self,
        map_shared: impl FnOnce(Shared<T>) -> ExecutionResult<Shared<O>>,
        map_owned: impl FnOnce(Owned<T::Owned>) -> ExecutionResult<Owned<O::Owned>>,
    ) -> ExecutionResult<CopyOnWrite<O>> {
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
        map_owned: impl FnOnce(Owned<T::Owned>) -> U,
    ) -> U {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => map_owned(owned),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => map_shared(shared),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => map_shared(shared),
        }
    }

    /// Disables this copy-on-write value, releasing any borrow on the RefCell.
    /// Returns a `DisabledCopyOnWrite` which can be cloned and later re-enabled.
    pub(crate) fn disable(self) -> DisabledCopyOnWrite<T> {
        let inner = match self.inner {
            CopyOnWriteInner::Owned(owned) => DisabledCopyOnWriteInner::Owned(owned),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                DisabledCopyOnWriteInner::SharedWithInfallibleCloning(shared.disable())
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                DisabledCopyOnWriteInner::SharedWithTransparentCloning(shared.disable())
            }
        };
        DisabledCopyOnWrite { inner }
    }
}

/// A disabled copy-on-write value that can be safely cloned and dropped.
pub(crate) struct DisabledCopyOnWrite<T: 'static + ToOwned + ?Sized> {
    inner: DisabledCopyOnWriteInner<T>,
}

enum DisabledCopyOnWriteInner<T: 'static + ToOwned + ?Sized> {
    Owned(Owned<T::Owned>),
    SharedWithInfallibleCloning(DisabledShared<T>),
    SharedWithTransparentCloning(DisabledShared<T>),
}

impl<T: 'static + ToOwned + ?Sized> Clone for DisabledCopyOnWrite<T>
where
    T::Owned: Clone,
{
    fn clone(&self) -> Self {
        let inner = match &self.inner {
            DisabledCopyOnWriteInner::Owned(owned) => {
                DisabledCopyOnWriteInner::Owned(owned.clone())
            }
            DisabledCopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                DisabledCopyOnWriteInner::SharedWithInfallibleCloning(shared.clone())
            }
            DisabledCopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                DisabledCopyOnWriteInner::SharedWithTransparentCloning(shared.clone())
            }
        };
        Self { inner }
    }
}

impl<T: 'static + ToOwned + ?Sized> DisabledCopyOnWrite<T> {
    /// Re-enables this disabled copy-on-write value by re-acquiring any borrow.
    pub(crate) fn enable(self, span: SpanRange) -> FunctionResult<CopyOnWrite<T>> {
        let inner = match self.inner {
            DisabledCopyOnWriteInner::Owned(owned) => CopyOnWriteInner::Owned(owned),
            DisabledCopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                CopyOnWriteInner::SharedWithInfallibleCloning(shared.enable(span)?)
            }
            DisabledCopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                CopyOnWriteInner::SharedWithTransparentCloning(shared.enable(span)?)
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

impl<T: ?Sized + ToOwned> AsRef<T> for CopyOnWrite<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized + ToOwned> Deref for CopyOnWrite<T> {
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
    pub(crate) fn into_shared(self) -> SharedValue {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => SharedValue::new_from_owned(owned),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => shared,
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared,
        }
    }
}

pub(crate) type CopyOnWriteValue = CopyOnWrite<AnyValue>;
