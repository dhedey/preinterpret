use crate::internal_prelude::*;

use super::IsVariable;
use std::cell::*;
use std::rc::Rc;

pub(crate) struct Interpreter {
    config: InterpreterConfig,
    variable_data: VariableData,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            config: Default::default(),
            variable_data: VariableData::new(),
        }
    }

    pub(crate) fn define_variable(
        &mut self,
        variable: &(impl IsVariable + ?Sized),
        value: ExpressionValue,
    ) {
        self.variable_data.define_variable(variable, value)
    }

    pub(crate) fn get_variable_reference(
        &self,
        variable: &(impl IsVariable + ?Sized),
        make_error: impl FnOnce() -> SynError,
    ) -> ExecutionResult<VariableReference> {
        self.variable_data.get_reference(variable, make_error)
    }

    pub(crate) fn start_iteration_counter<'s, S: HasSpanRange>(
        &self,
        span_source: &'s S,
    ) -> IterationCounter<'s, S> {
        IterationCounter {
            span_source,
            count: 0,
            iteration_limit: self.config.iteration_limit,
        }
    }

    pub(crate) fn set_iteration_limit(&mut self, limit: Option<usize>) {
        self.config.iteration_limit = limit;
    }
}

struct VariableData {
    variable_data: HashMap<String, VariableContent>,
}

impl VariableData {
    fn new() -> Self {
        Self {
            variable_data: HashMap::new(),
        }
    }

    fn define_variable(&mut self, variable: &(impl IsVariable + ?Sized), value: ExpressionValue) {
        self.variable_data.insert(
            variable.get_name(),
            VariableContent::new(value, variable.span_range()),
        );
    }

    fn get_reference(
        &self,
        variable: &(impl IsVariable + ?Sized),
        make_error: impl FnOnce() -> SynError,
    ) -> ExecutionResult<VariableReference> {
        let reference = self
            .variable_data
            .get(&variable.get_name())
            .ok_or_else(make_error)?
            .create_reference(variable);
        Ok(reference)
    }
}

struct VariableContent {
    value: Rc<RefCell<ExpressionValue>>,
    #[allow(unused)]
    definition_span_range: SpanRange,
}

impl VariableContent {
    fn new(tokens: ExpressionValue, definition_span_range: SpanRange) -> Self {
        Self {
            value: Rc::new(RefCell::new(tokens)),
            definition_span_range,
        }
    }

    fn create_reference(&self, variable: &(impl IsVariable + ?Sized)) -> VariableReference {
        VariableReference {
            data: self.value.clone(),
            variable_span_range: variable.span_range(),
        }
    }
}

pub(crate) struct IterationCounter<'a, S: HasSpanRange> {
    span_source: &'a S,
    count: usize,
    iteration_limit: Option<usize>,
}

impl<S: HasSpanRange> IterationCounter<'_, S> {
    pub(crate) fn increment_and_check(&mut self) -> ExecutionResult<()> {
        self.count += 1;
        self.check()
    }

    pub(crate) fn check(&self) -> ExecutionResult<()> {
        if let Some(limit) = self.iteration_limit {
            if self.count > limit {
                return self.span_source.execution_err(format!("Iteration limit of {} exceeded.\nIf needed, the limit can be reconfigured with [!settings! {{ iteration_limit: X }}]", limit));
            }
        }
        Ok(())
    }
}

pub(crate) struct InterpreterConfig {
    iteration_limit: Option<usize>,
}

pub(crate) const DEFAULT_ITERATION_LIMIT: usize = 1000;
pub(crate) const DEFAULT_ITERATION_LIMIT_STR: &str = "1000";

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            iteration_limit: Some(DEFAULT_ITERATION_LIMIT),
        }
    }
}

#[derive(Clone)]
pub(crate) struct VariableReference {
    data: Rc<RefCell<ExpressionValue>>,
    variable_span_range: SpanRange,
}

impl VariableReference {
    pub(crate) fn get_value_ref(&self) -> ExecutionResult<Ref<'_, ExpressionValue>> {
        self.data.try_borrow().map_err(|_| {
            self.execution_error("The variable cannot be read if it is currently being modified")
        })
    }

    // Gets the cloned expression value, setting the span range appropriately
    pub(crate) fn get_value_transparently_cloned(&self) -> ExecutionResult<ExpressionValue> {
        self.get_value_ref()?
            .try_transparent_clone(self.variable_span_range)
    }

    // Gets the cloned expression value, setting the span range appropriately
    pub(crate) fn get_value_cloned(&self) -> ExecutionResult<ExpressionValue> {
        Ok(self
            .get_value_ref()?
            .clone()
            .with_span_range(self.variable_span_range))
    }

    pub(crate) fn into_mut(self) -> ExecutionResult<MutableValue> {
        MutableValue::new_from_variable(self)
    }

    pub(crate) fn into_shared(self) -> ExecutionResult<SharedValue> {
        SharedValue::new_from_variable(self)
    }

    pub(crate) fn into_late_bound(self) -> ExecutionResult<Place> {
        match self.clone().into_mut() {
            Ok(value) => Ok(Place::MutableReference { mut_ref: value }),
            Err(ExecutionInterrupt::Error(reason_not_mutable)) => {
                // If we get an error with a mutable and shared reference, a mutable reference must already exist.
                // We can just propogate the error from taking the shared reference, it should be good enough.
                let value = self.into_shared()?;
                Ok(Place::SharedReference {
                    shared_ref: value,
                    reason_not_mutable: Some(reason_not_mutable),
                })
            }
            // Propogate any other errors, these shouldn't happen mind
            Err(err) => Err(err),
        }
    }
}

/// A rough equivalent of a Rust place (lvalue), as per:
/// https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
///
/// In preinterpret, references are (currently) only to variables, or sub-values of variables.
///
/// # Late Binding
///
/// Sometimes, a value which can be accessed, but we don't yet know *how* we need to access it.
/// In this case, we attempt to load it as a Mutable place, and failing that, as a Shared place.
///
/// ## Example of requirement
/// For example, if we have `x[a].y(z)`, we first need to resolve the type of `x[a]` to know
/// whether `x[a]` takes a shared reference, mutable reference or an owned value.
///
/// So instead, we take the most powerful access we can have for `x[a]`, and convert it later.
pub(crate) enum Place {
    MutableReference {
        mut_ref: MutableValue,
    },
    SharedReference {
        shared_ref: SharedValue,
        reason_not_mutable: Option<syn::Error>,
    },
}

impl HasSpanRange for VariableReference {
    fn span_range(&self) -> SpanRange {
        self.variable_span_range
    }
}

pub(crate) type CapturedMut<T> = MutableSubPlace<T>;
pub(crate) type MutableValue = MutableSubPlace<ExpressionValue>;

/// A mutable reference to a value inside a variable; along with a span of the whole access.
/// For example, for `x.y[4]`, this captures both:
/// * The mutable reference to the location under `x`
/// * The lexical span of the tokens `x.y[4]`
pub(crate) struct MutableSubPlace<T: 'static> {
    mut_cell: MutSubRcRefCell<ExpressionValue, T>,
    span_range: SpanRange,
}

impl<T> MutableSubPlace<T> {
    pub(crate) fn into_shared(self) -> SharedSubPlace<T> {
        SharedSubPlace {
            shared_cell: self.mut_cell.into_shared(),
            span_range: self.span_range,
        }
    }

    #[allow(unused)]
    pub(crate) fn map<V>(
        self,
        value_map: impl for<'a> FnOnce(&'a mut T) -> &'a mut V,
    ) -> MutableSubPlace<V> {
        MutableSubPlace {
            mut_cell: self.mut_cell.map(value_map),
            span_range: self.span_range,
        }
    }

    pub(crate) fn try_map<V>(
        self,
        value_map: impl for<'a, 'b> FnOnce(&'a mut T, &'b SpanRange) -> ExecutionResult<&'a mut V>,
    ) -> ExecutionResult<MutableSubPlace<V>> {
        Ok(MutableSubPlace {
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

impl MutableSubPlace<ExpressionValue> {
    pub(crate) fn new_from_owned(value: ExpressionValue) -> Self {
        let span_range = value.span_range();
        Self {
            // Unwrap is safe because it's a new refcell
            mut_cell: MutSubRcRefCell::new(Rc::new(RefCell::new(value))).unwrap(),
            span_range,
        }
    }

    fn new_from_variable(reference: VariableReference) -> ExecutionResult<Self> {
        Ok(Self {
            mut_cell: MutSubRcRefCell::new(reference.data).map_err(|_| {
                reference.variable_span_range.execution_error(
                    "The variable cannot be modified as it is already being modified",
                )
            })?,
            span_range: reference.variable_span_range,
        })
    }

    pub(crate) fn into_stream(self) -> ExecutionResult<MutableSubPlace<OutputStream>> {
        self.try_map(|value, span_range| match value {
            ExpressionValue::Stream(stream) => Ok(&mut stream.value),
            _ => span_range.execution_err("The variable is not a stream"),
        })
    }

    pub(crate) fn resolve_indexed_with_autocreate(
        self,
        access: IndexAccess,
        index: ExpressionValue,
    ) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map(|value, _| value.index_mut_with_autocreate(access, index))
    }

    pub(crate) fn resolve_property(self, access: PropertyAccess) -> ExecutionResult<Self> {
        self.update_span_range(|span_range| SpanRange::new_between(span_range, access.span_range()))
            .try_map(|value, _| value.property_mut(&access))
    }

    pub(crate) fn set(&mut self, content: impl ToExpressionValue) {
        *self.mut_cell = content.to_value(self.span_range);
    }
}

impl<T> AsMut<T> for MutableSubPlace<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.mut_cell
    }
}

impl<T> AsRef<T> for MutableSubPlace<T> {
    fn as_ref(&self) -> &T {
        &self.mut_cell
    }
}

impl<T: 'static> HasSpanRange for MutableSubPlace<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl WithSpanExt for MutableSubPlace<ExpressionValue> {
    fn with_span(self, span: Span) -> Self {
        Self {
            mut_cell: self.mut_cell,
            span_range: SpanRange::new_single(span),
        }
    }
}

impl<T> Deref for MutableSubPlace<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.mut_cell
    }
}

impl<T> DerefMut for MutableSubPlace<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mut_cell
    }
}

pub(crate) type CapturedRef<T> = SharedSubPlace<T>;
pub(crate) type SharedValue = SharedSubPlace<ExpressionValue>;

/// A mutable reference to a value inside a variable; along with a span of the whole access.
/// For example, for `x.y[4]`, this captures both:
/// * The mutable reference to the location under `x`
/// * The lexical span of the tokens `x.y[4]`
pub(crate) struct SharedSubPlace<T: 'static> {
    shared_cell: SharedSubRcRefCell<ExpressionValue, T>,
    span_range: SpanRange,
}

impl<T> SharedSubPlace<T> {
    pub(crate) fn try_map<V>(
        self,
        value_map: impl for<'a, 'b> FnOnce(&'a T, &'b SpanRange) -> ExecutionResult<&'a V>,
    ) -> ExecutionResult<SharedSubPlace<V>> {
        Ok(SharedSubPlace {
            shared_cell: self
                .shared_cell
                .try_map(|value| value_map(value, &self.span_range))?,
            span_range: self.span_range,
        })
    }

    #[allow(unused)]
    pub(crate) fn map<V>(
        self,
        value_map: impl FnOnce(&T) -> &V,
    ) -> ExecutionResult<SharedSubPlace<V>> {
        Ok(SharedSubPlace {
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

impl SharedSubPlace<ExpressionValue> {
    pub(crate) fn new_from_owned(value: ExpressionValue) -> Self {
        let span_range = value.span_range();
        Self {
            // Unwrap is safe because it's a new refcell
            shared_cell: SharedSubRcRefCell::new(Rc::new(RefCell::new(value))).unwrap(),
            span_range,
        }
    }

    fn new_from_variable(reference: VariableReference) -> ExecutionResult<Self> {
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

    pub(crate) fn resolve_property(self, access: PropertyAccess) -> ExecutionResult<Self> {
        self.update_span_range(|old_span| SpanRange::new_between(old_span, access.span_range()))
            .try_map(|value, _| value.property_ref(&access))
    }
}

impl<T> AsRef<T> for SharedSubPlace<T> {
    fn as_ref(&self) -> &T {
        &self.shared_cell
    }
}

impl<T> Deref for SharedSubPlace<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.shared_cell
    }
}

impl<T: 'static> HasSpanRange for SharedSubPlace<T> {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl<T: 'static> WithSpanExt for SharedSubPlace<T> {
    fn with_span(self, span: Span) -> Self {
        Self {
            shared_cell: self.shared_cell,
            span_range: SpanRange::new_single(span),
        }
    }
}
