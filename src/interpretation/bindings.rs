use super::*;

use std::borrow::{Borrow, ToOwned};
use std::cell::*;
use std::rc::Rc;

pub(super) enum VariableState {
    Uninitialized,
    Value(Rc<RefCell<Value>>),
    Finished,
}

impl VariableState {
    pub(crate) fn define(&mut self, value: Value) {
        match self {
            content @ VariableState::Uninitialized => {
                *content = VariableState::Value(Rc::new(RefCell::new(value)));
            }
            VariableState::Value(_) => panic!("Cannot define existing variable"),
            VariableState::Finished => panic!("Cannot define finished variable"),
        }
    }

    pub(crate) fn resolve(
        &mut self,
        variable_span: Span,
        is_final: bool,
        ownership: RequestedOwnership,
        blocked_from_mutation: Option<MutationBlockReason>,
    ) -> ExecutionResult<LateBoundValue> {
        const UNITIALIZED_ERR: &str = "Cannot resolve uninitialized variable. This shouldn't be possible, because all variables are set on first use.";
        const FINISHED_ERR: &str = "Cannot resolve finished variable. This shouldn't be possible, because is_final should be marked correctly. If you see this error, please report a bug to preinterpret on github with a reproduction case.";

        // If blocked from mutation, we technically could allow is_final to work and
        // return a fully owned value without observable mutation,
        // but it's likely confusingly inconsistent, so it's better to just block it entirely.
        let value_rc = if is_final && blocked_from_mutation.is_none() {
            let content = std::mem::replace(self, VariableState::Finished);
            match content {
                VariableState::Uninitialized => panic!("{}", UNITIALIZED_ERR),
                VariableState::Value(ref_cell) => match Rc::try_unwrap(ref_cell) {
                    Ok(ref_cell) => {
                        if matches!(
                            ownership,
                            RequestedOwnership::Concrete(ArgumentOwnership::Assignee { .. })
                        ) {
                            return variable_span.control_flow_err("The final usage of a variable cannot be assigned to. You can use `let _ = ..` to discard a value.");
                        }
                        return Ok(LateBoundValue::Owned(LateBoundOwnedValue {
                            owned: ref_cell.into_inner().into_owned(variable_span.span_range()),
                            is_from_last_use: true,
                        }));
                    }
                    // It's currently referenced, proceed with normal late-bound resolution.
                    // e.g.
                    // * `let x = %[]; x.assert_eq(x, %[]);` - the final `x` resolves to a shared reference
                    // * `let x = %[]; x.assert_eq(x + %[], %[]);` - errors because the final `x` is shared but it needs to be owned
                    Err(rc) => rc,
                },
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        } else {
            match self {
                VariableState::Uninitialized => panic!("{}", UNITIALIZED_ERR),
                VariableState::Value(ref_cell) => Rc::clone(ref_cell),
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        };
        let binding = VariableBinding {
            data: value_rc,
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
        if let Some(mutation_block_reason) = blocked_from_mutation {
            match resolved {
                Ok(LateBoundValue::Mutable(mutable)) => {
                    let reason_not_mutable = mutable
                        .syn_error(mutation_block_reason.error_message("mutate this variable"));
                    Ok(LateBoundValue::Shared(LateBoundSharedValue {
                        shared: mutable.into_shared(),
                        reason_not_mutable,
                    }))
                }
                x => x,
            }
        } else {
            resolved
        }
    }
}

#[derive(Clone)]
pub(crate) struct VariableBinding {
    data: Rc<RefCell<Value>>,
    variable_span: Span,
}

#[allow(unused)]
impl VariableBinding {
    /// Gets the cloned expression value, setting the span range appropriately
    /// This only works if the value can be transparently cloned
    pub(crate) fn into_transparently_cloned(self) -> ExecutionResult<OwnedValue> {
        self.into_shared()?.transparent_clone()
    }

    pub(crate) fn into_mut(self) -> ExecutionResult<MutableValue> {
        MutableValue::new_from_variable(self).map_err(ExecutionInterrupt::ownership_error)
    }

    pub(crate) fn into_shared(self) -> ExecutionResult<SharedValue> {
        SharedValue::new_from_variable(self).map_err(ExecutionInterrupt::ownership_error)
    }

    pub(crate) fn into_late_bound(self) -> ExecutionResult<LateBoundValue> {
        match MutableValue::new_from_variable(self.clone()) {
            Ok(value) => Ok(LateBoundValue::Mutable(value)),
            Err(reason_not_mutable) => {
                // If we get an error with a mutable and shared reference, a mutable reference must already exist.
                // We can just propagate the error from taking the shared reference, it should be good enough.
                let shared = self.into_shared()?;
                Ok(LateBoundValue::Shared(LateBoundSharedValue::new(
                    shared,
                    reason_not_mutable,
                )))
            }
        }
    }
}

pub(crate) struct LateBoundOwnedValue {
    pub(crate) owned: OwnedValue,
    pub(crate) is_from_last_use: bool,
}

impl WithSpanRangeExt for LateBoundOwnedValue {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        Self {
            owned: self.owned.with_span_range(span_range),
            is_from_last_use: self.is_from_last_use,
        }
    }
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

impl WithSpanRangeExt for LateBoundSharedValue {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        Self {
            shared: self.shared.with_span_range(span_range),
            reason_not_mutable: self.reason_not_mutable,
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
    Mutable(MutableValue),
    /// A shared reference where mutable access failed for a specific reason
    Shared(LateBoundSharedValue),
}

impl LateBoundValue {
    pub(crate) fn resolve(self, ownership: ArgumentOwnership) -> ExecutionResult<ArgumentValue> {
        ownership.map_from_late_bound(self)
    }

    pub(crate) fn map_any(
        self,
        map_shared: impl FnOnce(SharedValue) -> ExecutionResult<SharedValue>,
        map_mutable: impl FnOnce(MutableValue) -> ExecutionResult<MutableValue>,
        map_owned: impl FnOnce(OwnedValue) -> ExecutionResult<OwnedValue>,
    ) -> ExecutionResult<Self> {
        Ok(match self {
            LateBoundValue::Owned(owned) => LateBoundValue::Owned(LateBoundOwnedValue {
                owned: map_owned(owned.owned)?,
                is_from_last_use: owned.is_from_last_use,
            }),
            LateBoundValue::CopyOnWrite(copy_on_write) => {
                LateBoundValue::CopyOnWrite(copy_on_write.map_cow(map_shared, map_owned)?)
            }
            LateBoundValue::Mutable(mutable) => LateBoundValue::Mutable(map_mutable(mutable)?),
            LateBoundValue::Shared(LateBoundSharedValue {
                shared,
                reason_not_mutable,
            }) => LateBoundValue::Shared(LateBoundSharedValue::new(
                map_shared(shared)?,
                reason_not_mutable,
            )),
        })
    }
}

impl Deref for LateBoundValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<Value> for LateBoundValue {
    fn as_ref(&self) -> &Value {
        match self {
            LateBoundValue::Owned(owned) => &owned.owned,
            LateBoundValue::CopyOnWrite(cow) => cow.as_ref(),
            LateBoundValue::Mutable(mutable) => mutable.as_ref(),
            LateBoundValue::Shared(shared) => shared.shared.as_ref(),
        }
    }
}

impl HasSpanRange for LateBoundValue {
    fn span_range(&self) -> SpanRange {
        match self {
            LateBoundValue::Owned(owned) => owned.owned.1,
            LateBoundValue::CopyOnWrite(cow) => cow.1,
            LateBoundValue::Mutable(mutable) => mutable.1,
            LateBoundValue::Shared(shared) => shared.shared.1,
        }
    }
}

impl WithSpanRangeExt for LateBoundValue {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        match self {
            LateBoundValue::Owned(owned) => {
                LateBoundValue::Owned(owned.with_span_range(span_range))
            }
            LateBoundValue::CopyOnWrite(cow) => {
                LateBoundValue::CopyOnWrite(cow.with_span_range(span_range))
            }
            LateBoundValue::Mutable(mutable) => {
                LateBoundValue::Mutable(mutable.with_span_range(span_range))
            }
            LateBoundValue::Shared(shared) => {
                LateBoundValue::Shared(shared.with_span_range(span_range))
            }
        }
    }
}

pub(crate) type OwnedValue = Spanned<Owned<Value>>;

/// A wrapper for an owned value
pub(crate) struct Owned<T: 'static>(pub(crate) T);

#[allow(unused)]
impl<T> Owned<T> {
    pub(crate) fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Owned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(unused)]
impl<T> Spanned<Owned<T>> {
    pub(crate) fn new(value: T, span_range: SpanRange) -> Self {
        Spanned(Owned(value), span_range)
    }

    pub(crate) fn into_inner(self) -> T {
        self.0.into_inner()
    }

    pub(crate) fn as_ref_spanned<'a>(&'a self) -> SpannedAnyRef<'a, T> {
        self.0 .0.into_spanned_ref(self.1)
    }

    pub(crate) fn as_mut_spanned<'a>(&'a mut self) -> SpannedAnyRefMut<'a, T> {
        self.0 .0.into_spanned_ref_mut(self.1)
    }

    pub(crate) fn map_owned<V>(
        self,
        value_map: impl FnOnce(T, &SpanRange) -> V,
    ) -> Spanned<Owned<V>> {
        Spanned(Owned(value_map(self.0.into_inner(), &self.1)), self.1)
    }

    pub(crate) fn try_map_owned<V>(
        self,
        value_map: impl FnOnce(T, &SpanRange) -> ExecutionResult<V>,
    ) -> ExecutionResult<Spanned<Owned<V>>> {
        Ok(Spanned(Owned(value_map(self.0.into_inner(), &self.1)?), self.1))
    }

    pub(crate) fn update_span_range(
        self,
        span_range_map: impl FnOnce(SpanRange) -> SpanRange,
    ) -> Self {
        Spanned(self.0, span_range_map(self.1))
    }
}

impl OwnedValue {
    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: Spanned<&Value>,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map_owned(|value, _| value.into_indexed(access, index))
    }

    pub(crate) fn resolve_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map_owned(|value, _| value.into_property(access))
    }

    pub(crate) fn into_statement_result(self) -> ExecutionResult<()> {
        match *self.0 {
            Value::None => Ok(()),
            _ => self
                .1
                .control_flow_err("A non-returning statement must not return a value. If you wish to explicitly discard the expression's result, use `let _ = ...;`. Alternatively, If you wish to output the value into the parent token stream, use `emit ...;`"),
        }
    }

    pub(crate) fn into_value(self) -> Value {
        self.0.into_inner()
    }
}

impl<T: IntoValue> Spanned<Owned<T>> {
    pub(crate) fn into_value_inner(self) -> Value {
        self.0.into_inner().into_value()
    }

    pub(crate) fn into_owned_value(self) -> OwnedValue {
        let span_range = self.1;
        Spanned(Owned(self.into_value_inner()), span_range)
    }
}

impl From<OwnedValue> for Value {
    fn from(value: OwnedValue) -> Self {
        value.0.into_inner()
    }
}

pub(crate) type MutableValue = Spanned<Mutable<Value>>;
pub(crate) type AssigneeValue = Spanned<Assignee<Value>>;

/// A binding of a unique (mutable) reference to a value
/// See [`ArgumentOwnership::Assignee`] for more details.
pub(crate) struct Assignee<T: 'static + ?Sized>(pub Mutable<T>);

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

impl Spanned<Assignee<Value>> {
    pub(crate) fn set(&mut self, content: impl IntoValue) {
        *self.0 .0 .0 = content.into_value();
    }
}

/// A wrapper for a mutable reference to a value
pub(crate) struct Mutable<T: 'static + ?Sized>(pub(crate) MutSubRcRefCell<Value, T>);

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

    /// SAFETY:
    /// * Must be paired with a call to `enable()` before any further use of the value.
    /// * Must not use the value while disabled.
    pub(crate) unsafe fn disable(&mut self) {
        self.0.disable();
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

static MUTABLE_ERROR_MESSAGE: &str =
    "The variable cannot be modified as it is already being modified";

impl<T: ?Sized> Spanned<Mutable<T>> {
    pub(crate) fn into_shared(self) -> Spanned<Shared<T>> {
        Spanned(self.0.into_shared(), self.1)
    }

    #[allow(unused)]
    pub(crate) fn map_mutable<V: ?Sized>(
        self,
        value_map: impl for<'a> FnOnce(&'a mut T) -> &'a mut V,
    ) -> Spanned<Mutable<V>> {
        Spanned(self.0.map(value_map), self.1)
    }

    pub(crate) fn try_map_mutable<V: ?Sized>(
        self,
        value_map: impl for<'a, 'b> FnOnce(&'a mut T, &'b SpanRange) -> ExecutionResult<&'a mut V>,
    ) -> ExecutionResult<Spanned<Mutable<V>>> {
        Ok(Spanned(
            Mutable(self.0 .0.try_map(|value| value_map(value, &self.1))?),
            self.1,
        ))
    }

    pub(crate) fn update_span_range(
        self,
        span_range_map: impl FnOnce(SpanRange) -> SpanRange,
    ) -> Self {
        Spanned(self.0, span_range_map(self.1))
    }

    /// SAFETY:
    /// * Must only be used after a call to `disable()`.
    pub(crate) unsafe fn enable(&mut self) -> ExecutionResult<()> {
        self.0
             .0
            .enable()
            .map_err(|_| self.1.ownership_error(MUTABLE_ERROR_MESSAGE))
    }
}

#[allow(unused)]
impl MutableValue {
    pub(crate) fn new_from_owned(value: OwnedValue) -> Self {
        let Spanned(Owned(inner), span_range) = value;
        Spanned(
            // Unwrap is safe because it's a new refcell
            Mutable(MutSubRcRefCell::new(Rc::new(RefCell::new(inner))).unwrap()),
            span_range,
        )
    }

    pub(crate) fn transparent_clone(&self) -> ExecutionResult<OwnedValue> {
        let value = self.as_ref().try_transparent_clone(self.1)?;
        Ok(OwnedValue::new(value, self.1))
    }

    pub(super) fn new_from_variable(reference: VariableBinding) -> syn::Result<Self> {
        Ok(Spanned(
            Mutable(
                MutSubRcRefCell::new(reference.data)
                    .map_err(|_| reference.variable_span.syn_error(MUTABLE_ERROR_MESSAGE))?,
            ),
            reference.variable_span.span_range(),
        ))
    }

    pub(crate) fn into_stream(self) -> ExecutionResult<Spanned<Mutable<OutputStream>>> {
        self.try_map_mutable(|value, span_range| match value {
            Value::Stream(stream) => Ok(&mut stream.value),
            _ => span_range.type_err("The variable is not a stream"),
        })
    }

    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: Spanned<&Value>,
        auto_create: bool,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map_mutable(|value, _| value.index_mut(access, index, auto_create))
    }

    pub(crate) fn resolve_property(
        self,
        access: &PropertyAccess,
        auto_create: bool,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map_mutable(|value, _| value.property_mut(access, auto_create))
    }

    pub(crate) fn set(&mut self, content: impl IntoValue) {
        *self.0 .0 = content.into_value();
    }
}

pub(crate) type SharedValue = Spanned<Shared<Value>>;

/// A wrapper for a shared (immutable) reference to a value
pub(crate) struct Shared<T: 'static + ?Sized>(pub(crate) SharedSubRcRefCell<Value, T>);

#[allow(unused)]
impl<T: ?Sized> Shared<T> {
    pub(crate) fn clone(this: &Shared<T>) -> Self {
        Self(SharedSubRcRefCell::clone(&this.0))
    }

    pub(crate) fn map<V: ?Sized>(self, value_map: impl FnOnce(&T) -> &V) -> Shared<V> {
        Shared(self.0.map(value_map))
    }

    /// SAFETY:
    /// * Must be paired with a call to `enable()` before any further use of the value.
    /// * Must not use the value while disabled.
    pub(crate) unsafe fn disable(&mut self) {
        self.0.disable();
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

static SHARED_ERROR_MESSAGE: &str = "The variable cannot be read as it is already being modified";

#[allow(unused)]
impl<T: ?Sized> Spanned<Shared<T>> {
    pub(crate) fn clone_shared(this: &Spanned<Shared<T>>) -> Self {
        Spanned(Shared::clone(&this.0), this.1)
    }

    pub(crate) fn as_spanned(&self) -> Spanned<&T> {
        self.0.as_ref().spanned(self.1)
    }

    pub(crate) fn try_map_shared<V: ?Sized>(
        self,
        value_map: impl for<'a, 'b> FnOnce(&'a T, &'b SpanRange) -> ExecutionResult<&'a V>,
    ) -> ExecutionResult<Spanned<Shared<V>>> {
        Ok(Spanned(
            Shared(self.0 .0.try_map(|value| value_map(value, &self.1))?),
            self.1,
        ))
    }

    pub(crate) fn map_shared<V: ?Sized>(
        self,
        value_map: impl FnOnce(&T) -> &V,
    ) -> Spanned<Shared<V>> {
        Spanned(self.0.map(value_map), self.1)
    }

    pub(crate) fn update_span_range(
        self,
        span_range_map: impl FnOnce(SpanRange) -> SpanRange,
    ) -> Self {
        Spanned(self.0, span_range_map(self.1))
    }

    /// SAFETY:
    /// * Must only be used after a call to `disable()`.
    pub(crate) unsafe fn enable(&mut self) -> ExecutionResult<()> {
        self.0
             .0
            .enable()
            .map_err(|_| self.1.ownership_error(SHARED_ERROR_MESSAGE))
    }
}

impl SharedValue {
    pub(crate) fn new_from_owned(value: OwnedValue) -> Self {
        let Spanned(Owned(inner), span_range) = value;
        Spanned(
            // Unwrap is safe because it's a new refcell
            Shared(SharedSubRcRefCell::new(Rc::new(RefCell::new(inner))).unwrap()),
            span_range,
        )
    }

    pub(crate) fn transparent_clone(&self) -> ExecutionResult<OwnedValue> {
        let value = self.as_ref().try_transparent_clone(self.1)?;
        Ok(OwnedValue::new(value, self.1))
    }

    pub(crate) fn infallible_clone(&self) -> OwnedValue {
        self.as_ref().clone().into_owned(self.1)
    }

    pub(super) fn new_from_variable(reference: VariableBinding) -> syn::Result<Self> {
        Ok(Spanned(
            Shared(
                SharedSubRcRefCell::new(reference.data)
                    .map_err(|_| reference.variable_span.syn_error(SHARED_ERROR_MESSAGE))?,
            ),
            reference.variable_span.span_range(),
        ))
    }

    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: Spanned<&Value>,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|old_span| SpanRange::new_between(old_span, access))
            .try_map_shared(|value, _| value.index_ref(access, index))
    }

    pub(crate) fn resolve_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        self.update_span_range(|old_span| SpanRange::new_between(old_span, access.span_range()))
            .try_map_shared(|value, _| value.property_ref(access))
    }
}

/// Copy-on-write value that can be either owned or shared (without span information).
/// Use `Spanned<CopyOnWrite<T>>` to include span information.
pub(crate) struct CopyOnWrite<T: 'static + ToOwned + ?Sized> {
    inner: CopyOnWriteInner<T>,
}

enum CopyOnWriteInner<T: 'static + ToOwned + ?Sized> {
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
            CopyOnWriteInner::Owned(owned) => match map(owned.into_inner()) {
                Ok(mapped) => Ok(mapped),
                Err(other) => Err(Self {
                    inner: CopyOnWriteInner::Owned(Owned(other)),
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
        map_shared: impl FnOnce(Shared<T>) -> Shared<O>,
        map_owned: impl FnOnce(Owned<T::Owned>) -> Owned<O::Owned>,
    ) -> CopyOnWrite<O> {
        let inner = match self.inner {
            CopyOnWriteInner::Owned(owned) => CopyOnWriteInner::Owned(map_owned(owned)),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                CopyOnWriteInner::SharedWithInfallibleCloning(map_shared(shared))
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                CopyOnWriteInner::SharedWithTransparentCloning(map_shared(shared))
            }
        };
        CopyOnWrite { inner }
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

    /// SAFETY:
    /// * Must be paired with a call to `enable()` before any further use of the value.
    /// * Must not use the value while disabled.
    pub(crate) unsafe fn disable(&mut self) {
        match &mut self.inner {
            CopyOnWriteInner::Owned(_) => {}
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => shared.disable(),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared.disable(),
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
            CopyOnWriteInner::Owned(ref owned) => (**owned).borrow(),
            CopyOnWriteInner::SharedWithInfallibleCloning(ref shared) => shared.as_ref(),
            CopyOnWriteInner::SharedWithTransparentCloning(ref shared) => shared.as_ref(),
        }
    }
}

pub(crate) type CopyOnWriteValue = Spanned<CopyOnWrite<Value>>;

static COW_ERROR_MESSAGE: &str = "The variable cannot be read as it is already being modified";

impl<T: 'static + ToOwned + ?Sized> Spanned<CopyOnWrite<T>> {
    pub(crate) fn shared_in_place_of_owned(shared: Spanned<Shared<T>>) -> Self {
        Spanned(CopyOnWrite::shared_in_place_of_owned(shared.0), shared.1)
    }

    pub(crate) fn shared_in_place_of_shared(shared: Spanned<Shared<T>>) -> Self {
        Spanned(CopyOnWrite::shared_in_place_of_shared(shared.0), shared.1)
    }

    pub(crate) fn owned(owned: Spanned<Owned<T::Owned>>) -> Self {
        Spanned(CopyOnWrite::owned(owned.0), owned.1)
    }

    pub(crate) fn acts_as_shared_reference(&self) -> bool {
        self.0.acts_as_shared_reference()
    }

    pub(crate) fn map_cow<O: ToOwned + ?Sized>(
        self,
        map_shared: impl FnOnce(Spanned<Shared<T>>) -> ExecutionResult<Spanned<Shared<O>>>,
        map_owned: impl FnOnce(Spanned<Owned<T::Owned>>) -> ExecutionResult<Spanned<Owned<O::Owned>>>,
    ) -> ExecutionResult<Spanned<CopyOnWrite<O>>> {
        let span_range = self.1;
        match self.0.inner {
            CopyOnWriteInner::Owned(owned) => {
                let mapped = map_owned(Spanned(owned, span_range))?;
                Ok(Spanned(CopyOnWrite::owned(mapped.0), mapped.1))
            }
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                let mapped = map_shared(Spanned(shared, span_range))?;
                Ok(Spanned(
                    CopyOnWrite::shared_in_place_of_owned(mapped.0),
                    mapped.1,
                ))
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                let mapped = map_shared(Spanned(shared, span_range))?;
                Ok(Spanned(
                    CopyOnWrite::shared_in_place_of_shared(mapped.0),
                    mapped.1,
                ))
            }
        }
    }

    pub(crate) fn map_into<U>(
        self,
        map_shared: impl FnOnce(Spanned<Shared<T>>) -> U,
        map_owned: impl FnOnce(Spanned<Owned<T::Owned>>) -> U,
    ) -> U {
        let span_range = self.1;
        match self.0.inner {
            CopyOnWriteInner::Owned(owned) => map_owned(Spanned(owned, span_range)),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                map_shared(Spanned(shared, span_range))
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                map_shared(Spanned(shared, span_range))
            }
        }
    }

    pub(crate) fn update_span_range(
        self,
        span_range_map: impl FnOnce(SpanRange) -> SpanRange,
    ) -> Self {
        Spanned(self.0, span_range_map(self.1))
    }

    /// SAFETY:
    /// * Must only be used after a call to `disable()`.
    pub(crate) unsafe fn enable(&mut self) -> ExecutionResult<()> {
        match &mut self.0.inner {
            CopyOnWriteInner::Owned(_) => Ok(()),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => shared
                .0
                .enable()
                .map_err(|_| self.1.ownership_error(COW_ERROR_MESSAGE)),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared
                .0
                .enable()
                .map_err(|_| self.1.ownership_error(COW_ERROR_MESSAGE)),
        }
    }
}

impl CopyOnWriteValue {
    /// Converts to owned, cloning if necessary
    pub(crate) fn into_owned_infallible(self) -> OwnedValue {
        let span_range = self.1;
        match self.0.inner {
            CopyOnWriteInner::Owned(owned) => Spanned(owned, span_range),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                Spanned(shared, span_range).infallible_clone()
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                Spanned(shared, span_range).infallible_clone()
            }
        }
    }

    /// Converts to owned, cloning if necessary
    pub(crate) fn into_owned_transparently(self) -> ExecutionResult<OwnedValue> {
        let span_range = self.1;
        match self.0.inner {
            CopyOnWriteInner::Owned(owned) => Ok(Spanned(owned, span_range)),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                Ok(Spanned(shared, span_range).infallible_clone())
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                Spanned(shared, span_range).transparent_clone()
            }
        }
    }

    /// Converts to shared reference
    pub(crate) fn into_shared(self) -> SharedValue {
        let span_range = self.1;
        match self.0.inner {
            CopyOnWriteInner::Owned(owned) => SharedValue::new_from_owned(Spanned(owned, span_range)),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => Spanned(shared, span_range),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => Spanned(shared, span_range),
        }
    }
}
