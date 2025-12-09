use super::*;

/// A [`ArgumentValue`] represents a value which has had its ownership concretely
/// resolved for use in a specific operation (e.g. method call, property access, etc).
///
/// It is typically paired with an [`ArgumentOwnership`] (maybe wrapped in a [`RequestedOwnership`])
/// which indicates what ownership type to resolve to.
pub(crate) enum ArgumentValue {
    Owned(OwnedValue),
    CopyOnWrite(CopyOnWriteValue),
    Mutable(MutableValue),
    Assignee(AssigneeValue),
    Shared(SharedValue),
}

impl ArgumentValue {
    pub(crate) fn expect_owned(self) -> OwnedValue {
        match self {
            ArgumentValue::Owned(value) => value,
            _ => panic!("expect_owned() called on a non-owned ArgumentValue"),
        }
    }

    pub(crate) fn expect_copy_on_write(self) -> CopyOnWriteValue {
        match self {
            ArgumentValue::CopyOnWrite(value) => value,
            _ => panic!("expect_copy_on_write() called on a non-copy-on-write ArgumentValue"),
        }
    }

    pub(crate) fn expect_mutable(self) -> MutableValue {
        match self {
            ArgumentValue::Mutable(value) => value,
            _ => panic!("expect_mutable() called on a non-mutable ArgumentValue"),
        }
    }

    pub(crate) fn expect_assignee(self) -> AssigneeValue {
        match self {
            ArgumentValue::Assignee(value) => value,
            _ => panic!("expect_assignee() called on a non-assignee ArgumentValue"),
        }
    }

    pub(crate) fn expect_shared(self) -> SharedValue {
        match self {
            ArgumentValue::Shared(value) => value,
            _ => panic!("expect_shared() called on a non-shared ArgumentValue"),
        }
    }

    /// SAFETY:
    /// * Must be paired with a call to `enable()` before any further use of the value.
    /// * Must not use the value while disabled.
    pub(crate) unsafe fn disable(&mut self) {
        match self {
            ArgumentValue::Owned(_) => {}
            ArgumentValue::CopyOnWrite(copy_on_write) => copy_on_write.disable(),
            ArgumentValue::Mutable(mutable) => mutable.disable(),
            ArgumentValue::Assignee(assignee) => assignee.0.disable(),
            ArgumentValue::Shared(shared) => shared.disable(),
        }
    }

    /// SAFETY:
    /// * Must only be used after a call to `disable()`.
    ///
    /// Returns an ownership error if re-enabling fails (e.g., due to conflicting borrows).
    pub(crate) unsafe fn enable(&mut self) -> ExecutionResult<()> {
        match self {
            ArgumentValue::Owned(_) => Ok(()),
            ArgumentValue::CopyOnWrite(copy_on_write) => copy_on_write.enable(),
            ArgumentValue::Mutable(mutable) => mutable.enable(),
            ArgumentValue::Assignee(assignee) => assignee.0.enable(),
            ArgumentValue::Shared(shared) => shared.enable(),
        }
    }
}

// Note: ArgumentValue no longer implements HasSpanRange or WithSpanRangeExt
// since the inner value types no longer carry spans internally.
// Spans should be tracked separately at a higher level if needed.

impl Deref for ArgumentValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<Value> for ArgumentValue {
    fn as_ref(&self) -> &Value {
        match self {
            ArgumentValue::Owned(owned) => owned,
            ArgumentValue::Mutable(mutable) => mutable,
            ArgumentValue::Assignee(assignee) => &assignee.0,
            ArgumentValue::Shared(shared) => shared,
            ArgumentValue::CopyOnWrite(copy_on_write) => copy_on_write,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestedOwnership {
    /// Receives any of Owned, SharedReference or MutableReference, depending on what
    /// is available.
    /// This can then be used to resolve the value kind, and use the correct one.
    LateBound,
    /// A concrete value of the correct type.
    Concrete(ArgumentOwnership),
}

impl RequestedOwnership {
    pub(crate) fn owned() -> Self {
        RequestedOwnership::Concrete(ArgumentOwnership::Owned)
    }

    pub(crate) fn shared() -> Self {
        RequestedOwnership::Concrete(ArgumentOwnership::Shared)
    }

    pub(crate) fn replace_owned_with_copy_on_write(self) -> Self {
        match self {
            RequestedOwnership::Concrete(ArgumentOwnership::Owned) => {
                RequestedOwnership::Concrete(ArgumentOwnership::CopyOnWrite)
            }
            _ => self,
        }
    }

    pub(crate) fn requests_auto_create(&self) -> bool {
        match self {
            RequestedOwnership::Concrete(ArgumentOwnership::Assignee { auto_create }) => {
                *auto_create
            }
            _ => false,
        }
    }

    pub(crate) fn map_none(self) -> ExecutionResult<RequestedValue> {
        self.map_from_owned(().into_owned_value())
    }

    pub(crate) fn map_from_late_bound(
        &self,
        late_bound: LateBoundValue,
    ) -> ExecutionResult<RequestedValue> {
        Ok(match self {
            RequestedOwnership::LateBound => RequestedValue::LateBound(late_bound),
            RequestedOwnership::Concrete(_) => {
                panic!("Returning a late-bound reference when concrete ownership was requested")
            }
        })
    }

    pub(crate) fn map_from_argument(
        &self,
        value: ArgumentValue,
    ) -> ExecutionResult<RequestedValue> {
        match value {
            ArgumentValue::Owned(owned) => self.map_from_owned(owned),
            ArgumentValue::Mutable(mutable) => self.map_from_mutable(mutable),
            ArgumentValue::Assignee(assignee) => self.map_from_assignee(assignee),
            ArgumentValue::Shared(shared) => self.map_from_shared(shared),
            ArgumentValue::CopyOnWrite(copy_on_write) => self.map_from_copy_on_write(copy_on_write),
        }
    }

    pub(crate) fn map_from_returned(
        &self,
        value: ReturnedValue,
    ) -> ExecutionResult<RequestedValue> {
        match value {
            ReturnedValue::Owned(owned) => self.map_from_owned(owned),
            ReturnedValue::Mutable(mutable) => self.map_from_mutable(mutable),
            ReturnedValue::Shared(shared) => self.map_from_shared(shared),
            ReturnedValue::CopyOnWrite(copy_on_write) => self.map_from_copy_on_write(copy_on_write),
        }
    }

    /// This ensures the requested value's type aligns with the requested ownership.
    pub(crate) fn map_from_requested(
        &self,
        requested: RequestedValue,
    ) -> ExecutionResult<RequestedValue> {
        match requested {
            RequestedValue::Owned(owned) => self.map_from_owned(owned),
            RequestedValue::Shared(shared) => self.map_from_shared(shared),
            RequestedValue::Mutable(mutable) => self.map_from_mutable(mutable),
            RequestedValue::Assignee(assignee) => self.map_from_assignee(assignee),
            RequestedValue::LateBound(late_bound_value) => {
                self.map_from_late_bound(late_bound_value)
            }
            RequestedValue::CopyOnWrite(copy_on_write) => {
                self.map_from_copy_on_write(copy_on_write)
            }
            RequestedValue::AssignmentCompletion { .. } => {
                panic!("Returning a non-value item from a value context")
            }
        }
    }

    pub(crate) fn map_from_owned(&self, value: OwnedValue) -> ExecutionResult<RequestedValue> {
        match self {
            RequestedOwnership::LateBound => Ok(RequestedValue::LateBound(LateBoundValue::Owned(
                LateBoundOwnedValue {
                    owned: value,
                    // TODO: Get proper span - using call_site as fallback
                    span_range: Span::call_site().span_range(),
                    is_from_last_use: false,
                },
            ))),
            RequestedOwnership::Concrete(requested) => requested
                .map_from_owned(value)
                .map(Self::item_from_argument),
        }
    }

    pub(crate) fn map_from_copy_on_write(
        &self,
        cow: CopyOnWriteValue,
    ) -> ExecutionResult<RequestedValue> {
        match self {
            RequestedOwnership::LateBound => {
                Ok(RequestedValue::LateBound(LateBoundValue::CopyOnWrite(cow)))
            }
            RequestedOwnership::Concrete(requested) => requested
                .map_from_copy_on_write(cow)
                .map(Self::item_from_argument),
        }
    }

    pub(crate) fn map_from_mutable(
        &self,
        mutable: MutableValue,
    ) -> ExecutionResult<RequestedValue> {
        match self {
            RequestedOwnership::LateBound => {
                Ok(RequestedValue::LateBound(LateBoundValue::Mutable(mutable)))
            }
            RequestedOwnership::Concrete(requested) => requested
                .map_from_mutable(mutable)
                .map(Self::item_from_argument),
        }
    }

    pub(crate) fn map_from_assignee(
        &self,
        assignee: AssigneeValue,
    ) -> ExecutionResult<RequestedValue> {
        match self {
            RequestedOwnership::LateBound => Ok(RequestedValue::LateBound(
                LateBoundValue::Mutable(assignee.0),
            )),
            RequestedOwnership::Concrete(requested) => requested
                .map_from_assignee(assignee)
                .map(Self::item_from_argument),
        }
    }

    pub(crate) fn map_from_shared(&self, shared: SharedValue) -> ExecutionResult<RequestedValue> {
        match self {
            RequestedOwnership::LateBound => Ok(RequestedValue::LateBound(
                LateBoundValue::CopyOnWrite(CopyOnWrite::shared_in_place_of_shared(shared)),
            )),
            RequestedOwnership::Concrete(requested) => requested
                .map_from_shared(shared)
                .map(Self::item_from_argument),
        }
    }

    fn item_from_argument(value: ArgumentValue) -> RequestedValue {
        match value {
            ArgumentValue::Owned(owned) => RequestedValue::Owned(owned),
            ArgumentValue::Mutable(mutable) => RequestedValue::Mutable(mutable),
            ArgumentValue::Assignee(assignee) => RequestedValue::Assignee(assignee),
            ArgumentValue::Shared(shared) => RequestedValue::Shared(shared),
            ArgumentValue::CopyOnWrite(copy_on_write) => RequestedValue::CopyOnWrite(copy_on_write),
        }
    }
}

impl From<ArgumentOwnership> for RequestedOwnership {
    fn from(ownership: ArgumentOwnership) -> Self {
        RequestedOwnership::Concrete(ownership)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The ownership that a method might concretely request
pub(crate) enum ArgumentOwnership {
    Owned,
    Shared,
    Mutable,
    /// Approximately equivalent to Mutable, but explicitly for use as an assignee.
    /// (e.g. `x[0] = 1` or `obj.field = 2` or `x.swap(y)`)
    /// In rust, an assignee is a special type of place, and if you want to assign
    /// to a mutable reference, you have to explicitly dereference it with *.
    /// (Under Niko's Overwrite proposal, this would require that the value is also
    /// Overwrite).
    ///
    /// The distinction from mutable allows for different handling in ownership conversion.
    /// For example, whilst an owned value can be freely converted to a mutable reference
    /// for use in a method, it cannot be freely converted to an assignee.
    /// This prevents (1 = 2) = 3 style issues, where it would be insane to allow assignment
    /// to a floating owned value.
    ///
    /// The auto_create flag indicates whether the assignee should create missing entries,
    /// for example, it enables `obj.new_field = value` to work by auto-creating `new_field` in `obj`.
    Assignee {
        auto_create: bool,
    },
    /// Approximately equivalent to Owned, but more flexible to avoid cloning large values unnecessarily
    /// e.g. array indexing operations should take CopyOnWrite instead of Owned
    /// If a method needs to create an owned value, that method can transparently or infallibly copy it,
    /// as per the method's own requirements/expectations.
    CopyOnWrite,
    /// A niche resolved value which passes through the value uncoerced, for
    /// handling in the method itself - notably this is used in the `.as_mut()` method.
    ///
    /// Currently this is handled as a ArgumentValue, but it might be better to handle
    /// it as LateBound (so that we don't drop the shared-conversion error reason).
    AsIs,
}

impl ArgumentOwnership {
    pub(crate) fn map_from_late_bound(
        &self,
        late_bound: LateBoundValue,
    ) -> ExecutionResult<ArgumentValue> {
        match late_bound {
            LateBoundValue::Owned(owned) => {
                self.map_from_owned_with_is_last_use(owned.owned, owned.is_from_last_use)
            }
            LateBoundValue::CopyOnWrite(copy_on_write) => {
                self.map_from_copy_on_write(copy_on_write)
            }
            LateBoundValue::Mutable(mutable) => self.map_from_mutable_inner(mutable, true),
            LateBoundValue::Shared(late_bound_shared) => self
                .map_from_shared_with_error_reason(late_bound_shared.shared, |_| {
                    ExecutionInterrupt::ownership_error(late_bound_shared.reason_not_mutable)
                }),
        }
    }

    pub(crate) fn map_from_copy_on_write(
        &self,
        copy_on_write: CopyOnWriteValue,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned => {
                Ok(ArgumentValue::Owned(copy_on_write.into_owned_infallible()))
            }
            ArgumentOwnership::Shared => Ok(ArgumentValue::Shared(copy_on_write.into_shared())),
            ArgumentOwnership::Mutable => {
                if copy_on_write.acts_as_shared_reference() {
                    // TODO: Get proper span for error
                    Span::call_site().ownership_err("A mutable reference is required, but a shared reference was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.clone()` to get a mutable reference to a cloned value.")
                } else {
                    Ok(ArgumentValue::Mutable(Mutable::new_from_owned(
                        copy_on_write.into_owned_infallible().into_inner(),
                    )))
                }
            }
            ArgumentOwnership::Assignee { .. } => {
                // TODO: Get proper span for error
                if copy_on_write.acts_as_shared_reference() {
                    Span::call_site().ownership_err("A shared reference cannot be assigned to.")
                } else {
                    Span::call_site().ownership_err("An owned value cannot be assigned to.")
                }
            }
            ArgumentOwnership::CopyOnWrite | ArgumentOwnership::AsIs => {
                Ok(ArgumentValue::CopyOnWrite(copy_on_write))
            }
        }
    }

    pub(crate) fn map_from_shared(&self, shared: SharedValue) -> ExecutionResult<ArgumentValue> {
        self.map_from_shared_with_error_reason(
            shared,
            // TODO: Get proper span for error
            |_shared| Span::call_site().ownership_error("A mutable reference is required, but a shared reference was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.clone().as_mut()` to get a mutable reference."),
        )
    }

    fn map_from_shared_with_error_reason(
        &self,
        shared: SharedValue,
        mutable_error: impl FnOnce(SharedValue) -> ExecutionInterrupt,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned => Ok(ArgumentValue::Owned(shared.infallible_clone())),
            ArgumentOwnership::CopyOnWrite => Ok(ArgumentValue::CopyOnWrite(
                CopyOnWrite::shared_in_place_of_shared(shared),
            )),
            ArgumentOwnership::Assignee { .. } => Err(mutable_error(shared)),
            ArgumentOwnership::Mutable => Err(mutable_error(shared)),
            ArgumentOwnership::Shared | ArgumentOwnership::AsIs => {
                Ok(ArgumentValue::Shared(shared))
            }
        }
    }

    pub(crate) fn map_from_mutable(&self, mutable: MutableValue) -> ExecutionResult<ArgumentValue> {
        self.map_from_mutable_inner(mutable, false)
    }

    pub(crate) fn map_from_assignee(
        &self,
        assignee: AssigneeValue,
    ) -> ExecutionResult<ArgumentValue> {
        self.map_from_mutable_inner(assignee.0, false)
    }

    fn map_from_mutable_inner(
        &self,
        mutable: MutableValue,
        is_late_bound: bool,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned => {
                if is_late_bound {
                    // Use infallible clone for late-bound values
                    Ok(ArgumentValue::Owned(Owned(mutable.as_ref().clone())))
                } else {
                    // TODO: Get proper span for error
                    Span::call_site().ownership_err("An owned value is required, but a mutable reference was received. This indicates a possible bug. If this was intended, use `.clone()` to get an owned value.")
                }
            }
            ArgumentOwnership::CopyOnWrite => Ok(ArgumentValue::CopyOnWrite(
                CopyOnWrite::shared_in_place_of_shared(mutable.into_shared()),
            )),
            ArgumentOwnership::Mutable | ArgumentOwnership::AsIs => {
                Ok(ArgumentValue::Mutable(mutable))
            }
            ArgumentOwnership::Assignee { .. } => Ok(ArgumentValue::Assignee(Assignee(mutable))),
            ArgumentOwnership::Shared => Ok(ArgumentValue::Shared(mutable.into_shared())),
        }
    }

    pub(crate) fn map_from_owned(&self, owned: OwnedValue) -> ExecutionResult<ArgumentValue> {
        self.map_from_owned_with_is_last_use(owned, false)
    }

    fn map_from_owned_with_is_last_use(
        &self,
        owned: OwnedValue,
        is_from_last_use: bool,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned | ArgumentOwnership::AsIs => Ok(ArgumentValue::Owned(owned)),
            ArgumentOwnership::CopyOnWrite => {
                Ok(ArgumentValue::CopyOnWrite(CopyOnWrite::owned(owned)))
            }
            ArgumentOwnership::Mutable => Ok(ArgumentValue::Mutable(Mutable::new_from_owned(
                owned.into_inner(),
            ))),
            ArgumentOwnership::Assignee { .. } => {
                // TODO: Get proper span for error
                if is_from_last_use {
                    Span::call_site().ownership_err("The final usage of a variable cannot be assigned to. You can use `let _ = ..` to discard a value.")
                } else {
                    Span::call_site().ownership_err("An owned value cannot be assigned to.")
                }
            }
            ArgumentOwnership::Shared => Ok(ArgumentValue::Shared(Shared::new_from_owned(
                owned.into_inner(),
            ))),
        }
    }
}

/// Handlers which return a Value
pub(super) enum AnyValueFrame {
    Group(GroupBuilder),
    Array(ArrayBuilder),
    Object(Box<ObjectBuilder>),
    UnaryOperation(UnaryOperationBuilder),
    BinaryOperation(BinaryOperationBuilder),
    PropertyAccess(ValuePropertyAccessBuilder),
    IndexAccess(ValueIndexAccessBuilder),
    Range(RangeBuilder),
    Assignment(AssignmentBuilder),
    MethodCall(MethodCallBuilder),
}

impl AnyValueFrame {
    pub(super) fn handle_next(
        self,
        context: Context<ValueType>,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        match self {
            AnyValueFrame::Group(frame) => frame.handle_next(context, value),
            AnyValueFrame::Array(frame) => frame.handle_next(context, value),
            AnyValueFrame::Object(frame) => frame.handle_next(context, value),
            AnyValueFrame::UnaryOperation(frame) => frame.handle_next(context, value),
            AnyValueFrame::BinaryOperation(frame) => frame.handle_next(context, value),
            AnyValueFrame::PropertyAccess(frame) => frame.handle_next(context, value),
            AnyValueFrame::IndexAccess(frame) => frame.handle_next(context, value),
            AnyValueFrame::Range(frame) => frame.handle_next(context, value),
            AnyValueFrame::Assignment(frame) => frame.handle_next(context, value),
            AnyValueFrame::MethodCall(frame) => frame.handle_next(context, value),
        }
    }
}

pub(super) struct GroupBuilder {
    span: Span,
}

impl GroupBuilder {
    pub(super) fn start(
        context: ValueContext,
        delim_span: &DelimSpan,
        inner: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            span: delim_span.join(),
        };
        context.request_owned(frame, inner)
    }
}

impl EvaluationFrame for GroupBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Group(self)
    }

    fn handle_next(
        self,
        mut context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        let inner = value.expect_owned();
        context.set_output_span(self.span.span_range());
        context.return_value(inner)
    }
}

pub(super) struct ArrayBuilder {
    span: Span,
    unevaluated_items: Vec<ExpressionNodeId>,
    evaluated_items: Vec<Value>,
}

impl ArrayBuilder {
    pub(super) fn start(
        context: ValueContext,
        brackets: &Brackets,
        items: &[ExpressionNodeId],
    ) -> ExecutionResult<NextAction> {
        Self {
            span: brackets.join(),
            unevaluated_items: items.to_vec(),
            evaluated_items: Vec::with_capacity(items.len()),
        }
        .next(context)
    }

    pub(super) fn next(self, mut context: ValueContext) -> ExecutionResult<NextAction> {
        Ok(
            match self
                .unevaluated_items
                .get(self.evaluated_items.len())
                .cloned()
            {
                Some(next) => context.request_owned(self, next),
                None => {
                    context.set_output_span(self.span.span_range());
                    context.return_value(self.evaluated_items)?
                }
            },
        )
    }
}

impl EvaluationFrame for ArrayBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Array(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        let value = value.expect_owned();
        self.evaluated_items.push(value.into_inner());
        self.next(context)
    }
}

pub(super) struct ObjectBuilder {
    span: Span,
    pending: Option<PendingEntryPath>,
    unevaluated_entries: Vec<(ObjectKey, ExpressionNodeId)>,
    evaluated_entries: BTreeMap<String, ObjectEntry>,
}

enum PendingEntryPath {
    OnIndexKeyBranch {
        access: IndexAccess,
        value_node: ExpressionNodeId,
    },
    OnValueBranch {
        key: String,
        key_span: Span,
    },
}

impl ObjectBuilder {
    pub(super) fn start(
        context: ValueContext,
        braces: &Braces,
        entries: &[(ObjectKey, ExpressionNodeId)],
    ) -> ExecutionResult<NextAction> {
        Box::new(Self {
            span: braces.join(),
            unevaluated_entries: entries.to_vec(),
            evaluated_entries: BTreeMap::new(),
            pending: None,
        })
        .next(context)
    }

    fn next(mut self: Box<Self>, mut context: ValueContext) -> ExecutionResult<NextAction> {
        Ok(
            match self
                .unevaluated_entries
                .get(self.evaluated_entries.len())
                .cloned()
            {
                Some((ObjectKey::Identifier(ident), value_node)) => {
                    let key = ident.to_string();
                    if self.evaluated_entries.contains_key(&key) {
                        return ident.syntax_err(format!("The key {} has already been set", key));
                    }
                    self.pending = Some(PendingEntryPath::OnValueBranch {
                        key,
                        key_span: ident.span(),
                    });
                    context.request_owned(self, value_node)
                }
                Some((ObjectKey::Indexed { access, index }, value_node)) => {
                    self.pending = Some(PendingEntryPath::OnIndexKeyBranch { access, value_node });
                    context.request_owned(self, index)
                }
                None => {
                    context.set_output_span(self.span.span_range());
                    context.return_value(self.evaluated_entries)?
                }
            },
        )
    }
}

impl EvaluationFrame for Box<ObjectBuilder> {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Object(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        let pending = self.pending.take();
        Ok(match pending {
            Some(PendingEntryPath::OnIndexKeyBranch { access, value_node }) => {
                let value = value.expect_owned();
                let key: String = value.resolve_as("An object key")?;
                if self.evaluated_entries.contains_key(&key) {
                    return access.syntax_err(format!("The key {} has already been set", key));
                }
                self.pending = Some(PendingEntryPath::OnValueBranch {
                    key,
                    key_span: access.span(),
                });
                context.request_owned(self, value_node)
            }
            Some(PendingEntryPath::OnValueBranch { key, key_span }) => {
                let value = value.expect_owned().into_inner();
                let entry = ObjectEntry { key_span, value };
                self.evaluated_entries.insert(key, entry);
                self.next(context)?
            }
            None => {
                unreachable!("Should not receive a value without a pending handler set")
            }
        })
    }
}

pub(super) struct UnaryOperationBuilder {
    operation: UnaryOperation,
}

impl UnaryOperationBuilder {
    pub(super) fn start(
        context: ValueContext,
        operation: UnaryOperation,
        input: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self { operation };
        // Use late-bound evaluation to allow method resolution to determine ownership requirements
        context.request_late_bound(frame, input)
    }
}

impl EvaluationFrame for UnaryOperationBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::UnaryOperation(self)
    }

    fn handle_next(
        self,
        context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        let late_bound_value = value.expect_late_bound();
        let operand_kind = late_bound_value.kind();

        // Try method resolution first
        if let Some(interface) = operand_kind.resolve_unary_operation(&self.operation) {
            let resolved_value = late_bound_value.resolve(interface.argument_ownership())?;
            let result = interface.execute(resolved_value, &self.operation)?;
            return context.return_returned_value(result);
        }
        self.operation.type_err(format!(
            "The {} operator is not supported for {} values",
            self.operation.symbolic_description(),
            late_bound_value.value_type(),
        ))
    }
}

pub(super) struct BinaryOperationBuilder {
    operation: BinaryOperation,
    state: BinaryPath,
}

enum BinaryPath {
    OnLeftBranch {
        right: ExpressionNodeId,
    },
    OnRightBranch {
        left: ArgumentValue,
        interface: BinaryOperationInterface,
    },
}

impl BinaryOperationBuilder {
    pub(super) fn start(
        context: ValueContext,
        operation: BinaryOperation,
        left: ExpressionNodeId,
        right: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            operation,
            state: BinaryPath::OnLeftBranch { right },
        };
        // Use late-bound evaluation to allow method resolution to determine ownership requirements
        context.request_late_bound(frame, left)
    }
}

impl EvaluationFrame for BinaryOperationBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::BinaryOperation(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            BinaryPath::OnLeftBranch { right } => {
                let left_late_bound = value.expect_late_bound();

                // Check for lazy evaluation first (short-circuit operators)
                // TODO: Get proper span for left value
                let left_value = left_late_bound
                    .as_ref()
                    .spanned(self.operation.span_range());
                if let Some(result) = self.operation.lazy_evaluate(left_value)? {
                    context.return_returned_value(ReturnedValue::Owned(result))?
                } else {
                    // Try method resolution based on left operand's kind and resolve left operand immediately
                    let interface = left_late_bound
                        .as_ref()
                        .kind()
                        .resolve_binary_operation(&self.operation);

                    match interface {
                        Some(interface) => {
                            let rhs_ownership = interface.rhs_ownership();
                            let mut left = interface
                                .lhs_ownership()
                                .map_from_late_bound(left_late_bound)?;

                            unsafe {
                                // SAFETY: We re-enable it below and don't use it while disabled
                                left.disable();
                            }

                            self.state = BinaryPath::OnRightBranch { left, interface };
                            context.request_argument_value(self, right, rhs_ownership)
                        }
                        None => {
                            return self.operation.type_err(format!(
                                "The {} operator is not supported for {} operand",
                                self.operation.symbolic_description(),
                                left_late_bound.articled_value_type(),
                            ));
                        }
                    }
                }
            }
            BinaryPath::OnRightBranch {
                mut left,
                interface,
            } => {
                let mut right = value.expect_argument_value();

                // NOTE:
                // - This disable/enable flow allows us to do x += x without issues
                // - Read https://rust-lang.github.io/rfcs/2025-nested-method-calls.html for more details
                // - We enable left-to-right for more intuitive error messages:
                //   If left and right clash, then the error message should be on the right, not the left
                unsafe {
                    // SAFETY: We disabled left above
                    right.disable();
                    // SAFETY: enable() may fail if left and right reference the same variable
                    left.enable()?;
                    right.enable()?;
                }
                let result = interface.execute(left, right, &self.operation)?;
                return context.return_returned_value(result);
            }
        })
    }
}

pub(super) struct ValuePropertyAccessBuilder {
    access: PropertyAccess,
}

impl ValuePropertyAccessBuilder {
    pub(super) fn start(
        context: ValueContext,
        access: PropertyAccess,
        node: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self { access };
        // If we need an owned value, we can try resolving a reference and
        // clone the outputted value if needed - which can be much cheaper.
        // e.g. `let x = arr[0]` only copies `arr[0]` instead of the whole array.
        let ownership_request = context
            .requested_ownership()
            .replace_owned_with_copy_on_write();
        context.request_any_value(frame, node, ownership_request)
    }
}

impl EvaluationFrame for ValuePropertyAccessBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::PropertyAccess(self)
    }

    fn handle_next(
        self,
        context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        let auto_create = context.requested_ownership().requests_auto_create();
        let mapped = value.expect_any_value_and_map(
            |shared| shared.resolve_property(&self.access),
            |mutable| mutable.resolve_property(&self.access, auto_create),
            |owned| owned.resolve_property(&self.access),
        )?;
        context.return_not_necessarily_matching_requested(mapped)
    }
}

pub(super) struct ValueIndexAccessBuilder {
    access: IndexAccess,
    state: IndexPath,
}

enum IndexPath {
    OnSourceBranch { index: ExpressionNodeId },
    OnIndexBranch { source: RequestedValue },
}

impl ValueIndexAccessBuilder {
    pub(super) fn start(
        context: ValueContext,
        source: ExpressionNodeId,
        access: IndexAccess,
        index: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            access,
            state: IndexPath::OnSourceBranch { index },
        };
        // If we need an owned value, we can try resolving a reference and
        // clone the outputted value if needed - which can be much cheaper.
        // e.g. `let x = obj.xyz` only copies `obj.xyz` instead of the whole object.
        let ownership_request = context
            .requested_ownership()
            .replace_owned_with_copy_on_write();
        context.request_any_value(frame, source, ownership_request)
    }
}

impl EvaluationFrame for ValueIndexAccessBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::IndexAccess(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            IndexPath::OnSourceBranch { index } => {
                self.state = IndexPath::OnIndexBranch { source: value };
                // This is a value, so we are _accessing it_ and can't create values
                // (that's only possible in a place!) - therefore we don't need an owned key,
                // and can use &index for reading values from our array
                context.request_shared(self, index)
            }
            IndexPath::OnIndexBranch { source } => {
                let index = value.expect_shared();
                // Wrap index value in Spanned using access span
                let index_spanned = index.as_ref().spanned(self.access.span_range());

                let auto_create = context.requested_ownership().requests_auto_create();
                context.return_not_necessarily_matching_requested(
                    source.expect_any_value_and_map(
                        |shared| shared.resolve_indexed(self.access, index_spanned),
                        |mutable| mutable.resolve_indexed(self.access, index_spanned, auto_create),
                        |owned| owned.resolve_indexed(self.access, index_spanned),
                    )?,
                )?
            }
        })
    }
}

pub(super) struct RangeBuilder {
    range_limits: syn::RangeLimits,
    state: RangePath,
}

enum RangePath {
    OnLeftBranch { right: Option<ExpressionNodeId> },
    OnRightBranch { left: Option<Value> },
}

impl RangeBuilder {
    pub(super) fn start(
        mut context: ValueContext,
        left: &Option<ExpressionNodeId>,
        range_limits: &syn::RangeLimits,
        right: &Option<ExpressionNodeId>,
    ) -> ExecutionResult<NextAction> {
        Ok(match (left, right) {
            (None, None) => match range_limits {
                syn::RangeLimits::HalfOpen(token) => {
                    let inner = RangeValueInner::RangeFull { token: *token };
                    context.set_output_span(token.span_range());
                    context.return_value(inner)?
                }
                syn::RangeLimits::Closed(_) => {
                    unreachable!(
                        "A closed range should have been given a right in continue_range(..)"
                    )
                }
            },
            (None, Some(right)) => context.request_owned(
                Self {
                    range_limits: *range_limits,
                    state: RangePath::OnRightBranch { left: None },
                },
                *right,
            ),
            (Some(left), right) => context.request_owned(
                Self {
                    range_limits: *range_limits,
                    state: RangePath::OnLeftBranch { right: *right },
                },
                *left,
            ),
        })
    }
}

impl EvaluationFrame for RangeBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Range(self)
    }

    fn handle_next(
        mut self,
        mut context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        // TODO[range-refactor]: Change to not always clone the value
        let value = value.expect_owned().into_inner();
        Ok(match (self.state, self.range_limits) {
            (RangePath::OnLeftBranch { right: Some(right) }, _) => {
                self.state = RangePath::OnRightBranch { left: Some(value) };
                context.request_owned(self, right)
            }
            (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = RangeValueInner::RangeFrom {
                    start_inclusive: value,
                    token,
                };
                context.set_output_span(token.span_range());
                context.return_value(inner)?
            }
            (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::Closed(_)) => {
                unreachable!("A closed range should have been given a right in continue_range(..)")
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = RangeValueInner::Range {
                    start_inclusive: left,
                    token,
                    end_exclusive: value,
                };
                context.set_output_span(token.span_range());
                context.return_value(inner)?
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::Closed(token)) => {
                let inner = RangeValueInner::RangeInclusive {
                    start_inclusive: left,
                    token,
                    end_inclusive: value,
                };
                context.set_output_span(token.span_range());
                context.return_value(inner)?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = RangeValueInner::RangeTo {
                    token,
                    end_exclusive: value,
                };
                context.set_output_span(token.span_range());
                context.return_value(inner)?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::Closed(token)) => {
                let inner = RangeValueInner::RangeToInclusive {
                    token,
                    end_inclusive: value,
                };
                context.set_output_span(token.span_range());
                context.return_value(inner)?
            }
        })
    }
}

pub(super) struct AssignmentBuilder {
    #[allow(unused)]
    equals_token: Token![=],
    state: AssignmentPath,
}

enum AssignmentPath {
    OnValueBranch { assignee: ExpressionNodeId },
    OnAwaitingAssignment,
}

impl AssignmentBuilder {
    pub(super) fn start(
        context: ValueContext,
        assignee: ExpressionNodeId,
        equals_token: Token![=],
        value: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            equals_token,
            state: AssignmentPath::OnValueBranch { assignee },
        };
        context.request_owned(frame, value)
    }
}

impl EvaluationFrame for AssignmentBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Assignment(self)
    }

    fn handle_next(
        mut self,
        mut context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            AssignmentPath::OnValueBranch { assignee } => {
                let value = value.expect_owned().into_inner();
                self.state = AssignmentPath::OnAwaitingAssignment;
                context.request_assignment(self, assignee, value)
            }
            AssignmentPath::OnAwaitingAssignment => {
                let AssignmentCompletion { span_range } = value.expect_assignment_completion();
                context.set_output_span(span_range);
                context.return_value(())?
            }
        })
    }
}

pub(super) struct MethodCallBuilder {
    method: MethodAccess,
    unevaluated_parameters_stack: Vec<(ExpressionNodeId, ArgumentOwnership)>,
    state: MethodCallPath,
}

enum MethodCallPath {
    CallerPath,
    ArgumentsPath {
        method: MethodInterface,
        disabled_evaluated_arguments_including_caller: Vec<ArgumentValue>,
    },
}

impl MethodCallBuilder {
    pub(super) fn start(
        context: ValueContext,
        caller: ExpressionNodeId,
        method: MethodAccess,
        parameters: &[ExpressionNodeId],
    ) -> NextAction {
        let frame = Self {
            method,
            unevaluated_parameters_stack: parameters
                .iter()
                .rev()
                .map(|x| {
                    // This is just a placeholder - we'll fix it up shortly
                    (*x, ArgumentOwnership::Owned)
                })
                .collect(),
            state: MethodCallPath::CallerPath,
        };
        context.request_late_bound(frame, caller)
    }
}

impl EvaluationFrame for MethodCallBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::MethodCall(self)
    }

    fn handle_next(
        mut self,
        mut context: ValueContext,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        // Handle expected item based on current state
        match self.state {
            MethodCallPath::CallerPath => {
                let caller = value.expect_late_bound();
                let method = caller
                    .as_ref()
                    .kind()
                    .resolve_method(self.method.method.to_string().as_str());
                let method = match method {
                    Some(m) => m,
                    None => {
                        return self.method.method.type_err(format!(
                            "The method {} does not exist on {}",
                            self.method.method,
                            caller.as_ref().articled_value_type(),
                        ))
                    }
                };
                let non_caller_arguments = self.unevaluated_parameters_stack.len();
                let (argument_ownerships, min_arguments) = method.argument_ownerships();
                let max_arguments = argument_ownerships.len();
                assert!(
                    max_arguments >= 1 && min_arguments >= 1,
                    "Method calls must have at least one argument (the caller)"
                );
                let non_caller_min_arguments = min_arguments - 1;
                let non_caller_max_arguments = max_arguments - 1;

                if non_caller_arguments < non_caller_min_arguments
                    || non_caller_arguments > non_caller_max_arguments
                {
                    return self.method.method.type_err(format!(
                        "The method {} expects {} non-self argument/s, but {} were provided",
                        self.method.method,
                        if non_caller_min_arguments == non_caller_max_arguments {
                            (non_caller_min_arguments).to_string()
                        } else {
                            format!(
                                "{} to {}",
                                (non_caller_min_arguments),
                                (non_caller_max_arguments)
                            )
                        },
                        non_caller_arguments,
                    ));
                }
                let mut caller = argument_ownerships[0].map_from_late_bound(caller)?;

                // We skip 1 to ignore the caller
                let non_self_argument_ownerships: iter::Skip<
                    std::slice::Iter<'_, ArgumentOwnership>,
                > = argument_ownerships.iter().skip(1);
                for ((_, requested_ownership), ownership) in self
                    .unevaluated_parameters_stack
                    .iter_mut()
                    .rev() // Swap the stack back to the normal order
                    .zip(non_self_argument_ownerships)
                {
                    *requested_ownership = *ownership;
                }

                self.state = MethodCallPath::ArgumentsPath {
                    disabled_evaluated_arguments_including_caller: {
                        let mut params =
                            Vec::with_capacity(1 + self.unevaluated_parameters_stack.len());
                        unsafe {
                            // SAFETY: We enable it again before use
                            caller.disable();
                        }
                        params.push(caller);
                        params
                    },
                    method,
                };
            }
            MethodCallPath::ArgumentsPath {
                ref mut disabled_evaluated_arguments_including_caller,
                ..
            } => {
                let mut argument = value.expect_argument_value();
                unsafe {
                    // SAFETY: We enable it again before use
                    argument.disable();
                }
                disabled_evaluated_arguments_including_caller.push(argument);
            }
        };
        // Now plan the next action
        Ok(match self.unevaluated_parameters_stack.pop() {
            Some((parameter, ownership)) => {
                context.request_any_value(self, parameter, RequestedOwnership::Concrete(ownership))
            }
            None => {
                let (arguments, method) = match self.state {
                    MethodCallPath::CallerPath => unreachable!("Already updated above"),
                    MethodCallPath::ArgumentsPath {
                        disabled_evaluated_arguments_including_caller: mut arguments,
                        method,
                    } => {
                        // NOTE:
                        // - This disable/enable flow allows us to do things like vec.push(vec.len())
                        // - Read https://rust-lang.github.io/rfcs/2025-nested-method-calls.html for more details
                        // - We enable left-to-right for intuitive error messages: later borrows will report errors
                        unsafe {
                            for argument in &mut arguments {
                                // SAFETY: enable() may fail if arguments conflict (e.g., same variable)
                                argument.enable()?;
                            }
                        }
                        (arguments, method)
                    }
                };
                let mut call_context = MethodCallContext {
                    output_span_range: self.method.span_range(),
                    interpreter: context.interpreter(),
                };
                let output = method.execute(arguments, &mut call_context)?;
                context.return_returned_value(output)?
            }
        })
    }
}
