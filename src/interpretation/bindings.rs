use super::*;

use std::cell::*;
use std::rc::Rc;

pub(super) struct VariableContent {
    value: Rc<RefCell<ExpressionValue>>,
    #[allow(unused)]
    definition_span_range: SpanRange,
}

impl VariableContent {
    pub(super) fn new(tokens: ExpressionValue, definition_span_range: SpanRange) -> Self {
        Self {
            value: Rc::new(RefCell::new(tokens)),
            definition_span_range,
        }
    }

    pub(super) fn binding(&self, variable: &(impl IsVariable + ?Sized)) -> VariableBinding {
        VariableBinding {
            data: self.value.clone(),
            variable_span_range: variable.span_range(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct VariableBinding {
    data: Rc<RefCell<ExpressionValue>>,
    variable_span_range: SpanRange,
}

#[allow(unused)]
impl VariableBinding {
    /// Gets the cloned expression value, setting the span range appropriately
    /// This only works if the value can be transparently cloned
    pub(crate) fn into_transparently_cloned(self) -> ExecutionResult<OwnedValue> {
        self.into_shared()?.transparent_clone()
    }

    /// Gets the cloned expression value, setting the span range appropriately
    /// This works for any value, but may be more expensive
    pub(crate) fn into_infallibly_cloned(self) -> ExecutionResult<OwnedValue> {
        Ok(self.into_shared()?.infallible_clone())
    }

    pub(crate) fn into_mut(self) -> ExecutionResult<MutableValue> {
        MutableValue::new_from_variable(self)
    }

    pub(crate) fn into_shared(self) -> ExecutionResult<SharedValue> {
        SharedValue::new_from_variable(self)
    }

    pub(crate) fn into_late_bound(self) -> ExecutionResult<LateBoundValue> {
        match self.clone().into_mut() {
            Ok(value) => Ok(LateBoundValue::Mutable(value)),
            Err(ExecutionInterrupt::Error(reason_not_mutable)) => {
                // If we get an error with a mutable and shared reference, a mutable reference must already exist.
                // We can just propogate the error from taking the shared reference, it should be good enough.
                let shared = self.into_shared()?;
                Ok(LateBoundValue::Shared(LateBoundSharedValue::new(
                    shared,
                    reason_not_mutable,
                )))
            }
            // Propogate any other errors, these shouldn't happen mind
            Err(err) => Err(err),
        }
    }
}

impl HasSpanRange for VariableBinding {
    fn span_range(&self) -> SpanRange {
        self.variable_span_range
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
    Owned(OwnedValue),
    /// A copy-on-write value that can be converted to an owned value
    CopyOnWrite(CopyOnWriteValue),
    /// A mutable reference
    Mutable(MutableValue),
    /// A shared reference where mutable access failed for a specific reason
    Shared(LateBoundSharedValue),
}

impl LateBoundValue {
    pub(crate) fn resolve(
        self,
        ownership: ResolvedValueOwnership,
    ) -> ExecutionResult<ResolvedValue> {
        ownership.map_from_late_bound(self)
    }

    pub(crate) fn map_any(
        self,
        map_shared: impl FnOnce(SharedValue) -> ExecutionResult<SharedValue>,
        map_mutable: impl FnOnce(MutableValue) -> ExecutionResult<MutableValue>,
        map_owned: impl FnOnce(OwnedValue) -> ExecutionResult<OwnedValue>,
    ) -> ExecutionResult<Self> {
        Ok(match self {
            LateBoundValue::Owned(owned) => LateBoundValue::Owned(map_owned(owned)?),
            LateBoundValue::CopyOnWrite(copy_on_write) => {
                LateBoundValue::CopyOnWrite(copy_on_write.map_any(map_shared, map_owned)?)
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

impl AsRef<ExpressionValue> for LateBoundValue {
    fn as_ref(&self) -> &ExpressionValue {
        match self {
            LateBoundValue::Owned(owned) => owned.as_ref(),
            LateBoundValue::CopyOnWrite(cow) => cow.as_ref(),
            LateBoundValue::Mutable(mutable) => mutable.as_ref(),
            LateBoundValue::Shared(shared) => shared.shared.as_ref(),
        }
    }
}

impl HasSpanRange for LateBoundValue {
    fn span_range(&self) -> SpanRange {
        self.as_ref().span_range()
    }
}

pub(crate) type OwnedValue = Owned<ExpressionValue>;

/// A binding of an owned value along with a span of the whole access.
///
/// For example, for `x.y[4]`, this would capture both:
/// * The owned value
/// * The lexical span of the tokens `x.y[4]`
pub(crate) struct Owned<T: 'static> {
    inner: T,
    /// The span-range of the current binding to the value.
    span_range: SpanRange,
}

#[allow(unused)]
impl<T> Owned<T> {
    pub(crate) fn new(inner: T, span_range: SpanRange) -> Self {
        Self { inner, span_range }
    }

    pub(crate) fn deconstruct(self) -> (T, SpanRange) {
        (self.inner, self.span_range)
    }

    pub(crate) fn into_inner(self) -> T {
        self.inner
    }

    pub(crate) fn as_ref(&self) -> &T {
        &self.inner
    }

    pub(crate) fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub(crate) fn map<V>(self, value_map: impl FnOnce(T, &SpanRange) -> V) -> Owned<V> {
        Owned {
            inner: value_map(self.inner, &self.span_range),
            span_range: self.span_range,
        }
    }

    pub(crate) fn try_map<V>(
        self,
        value_map: impl FnOnce(T, &SpanRange) -> ExecutionResult<V>,
    ) -> ExecutionResult<Owned<V>> {
        Ok(Owned {
            inner: value_map(self.inner, &self.span_range)?,
            span_range: self.span_range,
        })
    }

    pub(crate) fn update_span_range(
        self,
        span_range_map: impl FnOnce(SpanRange) -> SpanRange,
    ) -> Self {
        Self {
            inner: self.inner,
            span_range: span_range_map(self.span_range),
        }
    }
}

impl OwnedValue {
    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: &ExpressionValue,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map(|value, _| value.into_indexed(access, index))
    }

    pub(crate) fn resolve_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map(|value, _| value.into_property(access))
    }
}

impl<T: ToExpressionValue> Owned<T> {
    pub(crate) fn into_value(self) -> ExpressionValue {
        self.inner.to_value(self.span_range)
    }
}

impl<T> HasSpanRange for Owned<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl From<OwnedValue> for ExpressionValue {
    fn from(value: OwnedValue) -> Self {
        value.inner
    }
}

impl Deref for OwnedValue {
    type Target = ExpressionValue;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for OwnedValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// NOTE: This should be deleted when we remove the span range from ExpressionValue
impl From<ExpressionValue> for OwnedValue {
    fn from(value: ExpressionValue) -> Self {
        let span_range = value.span_range();
        Self {
            inner: value,
            span_range,
        }
    }
}

impl<T> WithSpanRangeExt for Owned<T> {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        Self {
            inner: self.inner,
            span_range,
        }
    }
}

pub(crate) type MutableValue = Mutable<ExpressionValue>;

/// A binding of a unique (mutable) reference to a value
/// (e.g. inside a variable) along with a span of the whole access.
///
/// For example, for `x.y[4]`, this captures both:
/// * The mutable reference to the location under `x`
/// * The lexical span of the tokens `x.y[4]`
pub(crate) struct Mutable<T: 'static> {
    mut_cell: MutSubRcRefCell<ExpressionValue, T>,
    span_range: SpanRange,
}

impl<T> Mutable<T> {
    pub(crate) fn into_shared(self) -> Shared<T> {
        Shared {
            shared_cell: self.mut_cell.into_shared(),
            span_range: self.span_range,
        }
    }

    #[allow(unused)]
    pub(crate) fn map<V>(
        self,
        value_map: impl for<'a> FnOnce(&'a mut T) -> &'a mut V,
    ) -> Mutable<V> {
        Mutable {
            mut_cell: self.mut_cell.map(value_map),
            span_range: self.span_range,
        }
    }

    pub(crate) fn try_map<V>(
        self,
        value_map: impl for<'a, 'b> FnOnce(&'a mut T, &'b SpanRange) -> ExecutionResult<&'a mut V>,
    ) -> ExecutionResult<Mutable<V>> {
        Ok(Mutable {
            mut_cell: self
                .mut_cell
                .try_map(|value| value_map(value, &self.span_range))?,
            span_range: self.span_range,
        })
    }

    pub(crate) fn update_span_range(
        self,
        span_range_map: impl FnOnce(SpanRange) -> SpanRange,
    ) -> Self {
        Self {
            mut_cell: self.mut_cell,
            span_range: span_range_map(self.span_range),
        }
    }
}

#[allow(unused)]
impl Mutable<ExpressionValue> {
    pub(crate) fn new_from_owned(value: OwnedValue) -> Self {
        let span_range = value.span_range;
        Self {
            // Unwrap is safe because it's a new refcell
            mut_cell: MutSubRcRefCell::new(Rc::new(RefCell::new(value.inner))).unwrap(),
            span_range,
        }
    }

    pub(crate) fn transparent_clone(&self) -> ExecutionResult<OwnedValue> {
        let value = self.as_ref().try_transparent_clone(self.span_range)?;
        Ok(OwnedValue::new(value, self.span_range))
    }

    fn new_from_variable(reference: VariableBinding) -> ExecutionResult<Self> {
        Ok(Self {
            mut_cell: MutSubRcRefCell::new(reference.data).map_err(|_| {
                reference.variable_span_range.execution_error(
                    "The variable cannot be modified as it is already being modified",
                )
            })?,
            span_range: reference.variable_span_range,
        })
    }

    pub(crate) fn into_stream(self) -> ExecutionResult<Mutable<OutputStream>> {
        self.try_map(|value, span_range| match value {
            ExpressionValue::Stream(stream) => Ok(&mut stream.value),
            _ => span_range.execution_err("The variable is not a stream"),
        })
    }

    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: &ExpressionValue,
        auto_create: bool,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map(|value, _| value.index_mut(access, index, auto_create))
    }

    pub(crate) fn resolve_property(
        self,
        access: &PropertyAccess,
        auto_create: bool,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map(|value, _| value.property_mut(access, auto_create))
    }

    pub(crate) fn set(&mut self, content: impl ToExpressionValue) {
        *self.mut_cell = content.to_value(self.span_range);
    }
}

impl<T> AsMut<T> for Mutable<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.mut_cell
    }
}

impl<T> AsRef<T> for Mutable<T> {
    fn as_ref(&self) -> &T {
        &self.mut_cell
    }
}

impl<T: 'static> HasSpanRange for Mutable<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl WithSpanRangeExt for Mutable<ExpressionValue> {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        Self {
            mut_cell: self.mut_cell,
            span_range,
        }
    }
}

impl<T> Deref for Mutable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.mut_cell
    }
}

impl<T> DerefMut for Mutable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mut_cell
    }
}

pub(crate) type SharedValue = Shared<ExpressionValue>;

/// A binding of a shared (immutable) reference to a value
/// (e.g. inside a variable) along with a span of the whole access.
///
/// For example, for `x.y[4]`, this captures both:
/// * The mutable reference to the location under `x`
/// * The lexical span of the tokens `x.y[4]`
pub(crate) struct Shared<T: 'static> {
    shared_cell: SharedSubRcRefCell<ExpressionValue, T>,
    span_range: SpanRange,
}

#[allow(unused)]
impl<T> Shared<T> {
    pub(crate) fn clone(this: &Shared<T>) -> Self {
        Self {
            shared_cell: SharedSubRcRefCell::clone(&this.shared_cell),
            span_range: this.span_range,
        }
    }

    pub(crate) fn try_map<V>(
        self,
        value_map: impl for<'a, 'b> FnOnce(&'a T, &'b SpanRange) -> ExecutionResult<&'a V>,
    ) -> ExecutionResult<Shared<V>> {
        Ok(Shared {
            shared_cell: self
                .shared_cell
                .try_map(|value| value_map(value, &self.span_range))?,
            span_range: self.span_range,
        })
    }

    pub(crate) fn map<V>(self, value_map: impl FnOnce(&T) -> &V) -> ExecutionResult<Shared<V>> {
        Ok(Shared {
            shared_cell: self.shared_cell.map(value_map),
            span_range: self.span_range,
        })
    }

    pub(crate) fn update_span_range(
        self,
        span_range_map: impl FnOnce(SpanRange) -> SpanRange,
    ) -> Self {
        Self {
            shared_cell: self.shared_cell,
            span_range: span_range_map(self.span_range),
        }
    }
}

impl Shared<ExpressionValue> {
    pub(crate) fn new_from_owned(value: OwnedValue) -> Self {
        let span_range = value.span_range;
        Self {
            // Unwrap is safe because it's a new refcell
            shared_cell: SharedSubRcRefCell::new(Rc::new(RefCell::new(value.inner))).unwrap(),
            span_range,
        }
    }

    pub(crate) fn transparent_clone(&self) -> ExecutionResult<OwnedValue> {
        let value = self.as_ref().try_transparent_clone(self.span_range)?;
        Ok(OwnedValue::new(value, self.span_range))
    }

    pub(crate) fn infallible_clone(&self) -> OwnedValue {
        let value = self.as_ref().clone().with_span_range(self.span_range);
        OwnedValue::new(value, self.span_range)
    }

    fn new_from_variable(reference: VariableBinding) -> ExecutionResult<Self> {
        Ok(Self {
            shared_cell: SharedSubRcRefCell::new(reference.data).map_err(|_| {
                reference
                    .variable_span_range
                    .execution_error("The variable cannot be read as it is already being modified")
            })?,
            span_range: reference.variable_span_range,
        })
    }

    pub(crate) fn resolve_indexed(
        self,
        access: IndexAccess,
        index: &ExpressionValue,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|old_span| SpanRange::new_between(old_span, access))
            .try_map(|value, _| value.index_ref(access, index))
    }

    pub(crate) fn resolve_property(self, access: &PropertyAccess) -> ExecutionResult<Self> {
        self.update_span_range(|old_span| SpanRange::new_between(old_span, access.span_range()))
            .try_map(|value, _| value.property_ref(access))
    }
}

impl<T> AsRef<T> for Shared<T> {
    fn as_ref(&self) -> &T {
        &self.shared_cell
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.shared_cell
    }
}

impl<T: 'static> HasSpanRange for Shared<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl<T: 'static> WithSpanRangeExt for Shared<T> {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        Self {
            shared_cell: self.shared_cell,
            span_range,
        }
    }
}

/// Copy-on-write value that can be either owned or shared
pub(crate) struct CopyOnWrite<T: 'static> {
    inner: CopyOnWriteInner<T>,
}

enum CopyOnWriteInner<T: 'static> {
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

    /// Gets a shared reference to the value
    pub(crate) fn as_ref(&self) -> &T {
        match &self.inner {
            CopyOnWriteInner::Owned(owned) => owned.as_ref(),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => shared.as_ref(),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared.as_ref(),
        }
    }

    pub(crate) fn acts_as_shared_reference(&self) -> bool {
        match &self.inner {
            CopyOnWriteInner::Owned { .. } => false,
            CopyOnWriteInner::SharedWithInfallibleCloning { .. } => false,
            CopyOnWriteInner::SharedWithTransparentCloning { .. } => true,
        }
    }
}

impl CopyOnWrite<ExpressionValue> {
    /// Converts to owned, cloning if necessary
    pub(crate) fn into_owned_infallible(self) -> OwnedValue {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => owned,
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => shared.infallible_clone(),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared.infallible_clone(),
        }
    }

    /// Converts to owned, cloning if necessary
    pub(crate) fn into_owned_transparently(self) -> ExecutionResult<OwnedValue> {
        match self.inner {
            CopyOnWriteInner::Owned(owned) => Ok(owned),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => Ok(shared.infallible_clone()),
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => shared.transparent_clone(),
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

impl<T> WithSpanRangeExt for CopyOnWrite<T> {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        let inner = match self.inner {
            CopyOnWriteInner::Owned(owned) => {
                CopyOnWriteInner::Owned(owned.with_span_range(span_range))
            }
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                CopyOnWriteInner::SharedWithInfallibleCloning(shared.with_span_range(span_range))
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                CopyOnWriteInner::SharedWithTransparentCloning(shared.with_span_range(span_range))
            }
        };
        Self { inner }
    }
}

impl<T: 'static + HasSpanRange> HasSpanRange for CopyOnWrite<T> {
    fn span_range(&self) -> SpanRange {
        self.as_ref().span_range()
    }
}

pub(crate) type CopyOnWriteValue = CopyOnWrite<ExpressionValue>;

impl<T> CopyOnWrite<T> {
    pub(crate) fn map_any<O>(
        self,
        map_shared: impl FnOnce(Shared<T>) -> ExecutionResult<Shared<O>>,
        map_owned: impl FnOnce(Owned<T>) -> ExecutionResult<Owned<O>>,
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
}
