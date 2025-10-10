#![allow(unused)] // TODO[unused-clearup]
use super::*;

/// A [`ResolvedValue`] represents a value which has had its ownership concretely
/// resolved for use in a specific operation (e.g. method call, property access, etc)
pub(crate) enum ResolvedValue {
    Owned(OwnedValue),
    CopyOnWrite(CopyOnWriteValue),
    Mutable(MutableValue),
    Shared(SharedValue),
}

impl ResolvedValue {
    pub(crate) fn expect_owned(self) -> OwnedValue {
        match self {
            ResolvedValue::Owned(value) => value,
            _ => panic!("expect_owned() called on a non-owned ResolvedValue"),
        }
    }

    pub(crate) fn expect_copy_on_write(self) -> CopyOnWriteValue {
        match self {
            ResolvedValue::CopyOnWrite(value) => value,
            _ => panic!("expect_copy_on_write() called on a non-copy-on-write ResolvedValue"),
        }
    }

    pub(crate) fn expect_mutable(self) -> MutableValue {
        match self {
            ResolvedValue::Mutable(value) => value,
            _ => panic!("expect_mutable() called on a non-mutable ResolvedValue"),
        }
    }

    pub(crate) fn expect_shared(self) -> SharedValue {
        match self {
            ResolvedValue::Shared(value) => value,
            _ => panic!("expect_shared() called on a non-shared ResolvedValue"),
        }
    }

    pub(crate) fn as_value_ref(&self) -> &ExpressionValue {
        self.as_ref()
    }
}

impl HasSpanRange for ResolvedValue {
    fn span_range(&self) -> SpanRange {
        match self {
            ResolvedValue::Owned(owned) => owned.span_range(),
            ResolvedValue::CopyOnWrite(copy_on_write) => copy_on_write.span_range(),
            ResolvedValue::Mutable(mutable) => mutable.span_range(),
            ResolvedValue::Shared(shared) => shared.span_range(),
        }
    }
}

impl WithSpanRangeExt for ResolvedValue {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        match self {
            ResolvedValue::Owned(value) => ResolvedValue::Owned(value.with_span_range(span_range)),
            ResolvedValue::Mutable(reference) => {
                ResolvedValue::Mutable(reference.with_span_range(span_range))
            }
            ResolvedValue::Shared(shared) => {
                ResolvedValue::Shared(shared.with_span_range(span_range))
            }
            ResolvedValue::CopyOnWrite(copy_on_write) => {
                ResolvedValue::CopyOnWrite(copy_on_write.with_span_range(span_range))
            }
        }
    }
}

impl Deref for ResolvedValue {
    type Target = ExpressionValue;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<ExpressionValue> for ResolvedValue {
    fn as_ref(&self) -> &ExpressionValue {
        match self {
            ResolvedValue::Owned(owned) => owned.as_ref(),
            ResolvedValue::Mutable(mutable) => mutable.as_ref(),
            ResolvedValue::Shared(shared) => shared.as_ref(),
            ResolvedValue::CopyOnWrite(copy_on_write) => copy_on_write.as_ref(),
        }
    }
}

pub(crate) use crate::interpretation::CopyOnWriteValue;
use crate::stream_interface::method_definitions::assert;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestedValueOwnership {
    /// Receives any of Owned, SharedReference or MutableReference, depending on what
    /// is available.
    /// This can then be used to resolve the value kind, and use the correct one.
    LateBound,
    /// A concrete value of the correct type.
    Concrete(ResolvedValueOwnership),
}

impl RequestedValueOwnership {
    pub(super) fn replace_owned_with_copy_on_write(self) -> Self {
        match self {
            RequestedValueOwnership::Concrete(ResolvedValueOwnership::Owned) => {
                RequestedValueOwnership::Concrete(ResolvedValueOwnership::CopyOnWrite)
            }
            _ => self,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The ownership that a method might concretely request
pub(crate) enum ResolvedValueOwnership {
    Owned,
    Shared,
    Mutable,
    /// Approximately equivalent to Owned, but more flexible to avoid cloning large values unnecessarily
    /// e.g. array indexing operations should take CopyOnWrite instead of Owned
    /// If a method needs to create an owned value, that method can transparently or infallibly copy it,
    /// as per the method's own requirements/expectations.
    CopyOnWrite,
}

impl ResolvedValueOwnership {
    pub(crate) fn map_from_late_bound(
        &self,
        late_bound: LateBoundValue,
    ) -> ExecutionResult<ResolvedValue> {
        match late_bound {
            LateBoundValue::Owned(owned) => self.map_from_owned(owned),
            LateBoundValue::CopyOnWrite(copy_on_write) => {
                self.map_from_copy_on_write(copy_on_write)
            }
            LateBoundValue::Mutable(mutable) => self.map_from_mutable_inner(mutable, true),
            LateBoundValue::Shared(late_bound_shared) => self
                .map_from_shared_with_error_reason(late_bound_shared.shared, |_| {
                    late_bound_shared.reason_not_mutable.into()
                }),
        }
    }

    pub(crate) fn map_from_copy_on_write(
        &self,
        copy_on_write: CopyOnWriteValue,
    ) -> ExecutionResult<ResolvedValue> {
        match self {
            ResolvedValueOwnership::Owned => Ok(ResolvedValue::Owned(
                copy_on_write.into_owned_transparently()?,
            )),
            ResolvedValueOwnership::Shared => {
                Ok(ResolvedValue::Shared(copy_on_write.into_shared()))
            }
            ResolvedValueOwnership::Mutable => {
                if copy_on_write.acts_as_shared_reference() {
                    copy_on_write.execution_err("A mutable reference is required, but a shared reference was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.clone().as_mut()` to get a mutable reference.")
                } else {
                    copy_on_write.execution_err("A mutable reference is required, but an owned value was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.as_mut()` to get a mutable reference.")
                }
            }
            ResolvedValueOwnership::CopyOnWrite => Ok(ResolvedValue::CopyOnWrite(copy_on_write)),
        }
    }

    pub(crate) fn map_from_shared(&self, shared: SharedValue) -> ExecutionResult<ResolvedValue> {
        self.map_from_shared_with_error_reason(
            shared,
            |shared| shared.execution_error("A mutable reference is required, but a shared reference was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.clone().as_mut()` to get a mutable reference."),
        )
    }

    fn map_from_shared_with_error_reason(
        &self,
        shared: SharedValue,
        mutable_error: impl FnOnce(SharedValue) -> ExecutionInterrupt,
    ) -> ExecutionResult<ResolvedValue> {
        match self {
            ResolvedValueOwnership::Owned => Ok(ResolvedValue::Owned(shared.transparent_clone()?)),
            ResolvedValueOwnership::CopyOnWrite => Ok(ResolvedValue::CopyOnWrite(
                CopyOnWrite::shared_in_place_of_shared(shared),
            )),
            ResolvedValueOwnership::Mutable => Err(mutable_error(shared)),
            ResolvedValueOwnership::Shared => Ok(ResolvedValue::Shared(shared)),
        }
    }

    pub(crate) fn map_from_mutable(&self, mutable: MutableValue) -> ExecutionResult<ResolvedValue> {
        self.map_from_mutable_inner(mutable, false)
    }

    fn map_from_mutable_inner(
        &self,
        mutable: MutableValue,
        is_late_bound: bool,
    ) -> ExecutionResult<ResolvedValue> {
        match self {
            ResolvedValueOwnership::Owned => {
                if is_late_bound {
                    Ok(ResolvedValue::Owned(mutable.transparent_clone()?))
                } else {
                    mutable.execution_err("An owned value is required, but a mutable reference was received. This indicates a possible bug. If this was intended, use `.take()` or `.clone()` to get an owned value.")
                }
            }
            ResolvedValueOwnership::CopyOnWrite => Ok(ResolvedValue::CopyOnWrite(
                CopyOnWrite::shared_in_place_of_shared(mutable.into_shared()),
            )),
            ResolvedValueOwnership::Mutable => Ok(ResolvedValue::Mutable(mutable)),
            ResolvedValueOwnership::Shared => Ok(ResolvedValue::Shared(mutable.into_shared())),
        }
    }

    pub(crate) fn map_from_owned(&self, owned: OwnedValue) -> ExecutionResult<ResolvedValue> {
        match self {
            ResolvedValueOwnership::Owned => Ok(ResolvedValue::Owned(owned)),
            ResolvedValueOwnership::CopyOnWrite => Ok(ResolvedValue::CopyOnWrite(CopyOnWrite::owned(owned))),
            ResolvedValueOwnership::Mutable => owned.execution_err("A mutable reference is required, but an owned value was received (possibly from a last variable use), this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.as_mut()` to get a mutable reference."),
            ResolvedValueOwnership::Shared => Ok(ResolvedValue::Shared(Shared::new_from_owned(owned))),
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
    CompoundAssignment(CompoundAssignmentBuilder),
    MethodCall(MethodCallBuilder),
}

impl AnyValueFrame {
    pub(super) fn handle_item(
        self,
        context: Context<ValueType>,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        match self {
            AnyValueFrame::Group(frame) => frame.handle_item(context, item),
            AnyValueFrame::Array(frame) => frame.handle_item(context, item),
            AnyValueFrame::Object(frame) => frame.handle_item(context, item),
            AnyValueFrame::UnaryOperation(frame) => frame.handle_item(context, item),
            AnyValueFrame::BinaryOperation(frame) => frame.handle_item(context, item),
            AnyValueFrame::PropertyAccess(frame) => frame.handle_item(context, item),
            AnyValueFrame::IndexAccess(frame) => frame.handle_item(context, item),
            AnyValueFrame::Range(frame) => frame.handle_item(context, item),
            AnyValueFrame::Assignment(frame) => frame.handle_item(context, item),
            AnyValueFrame::CompoundAssignment(frame) => frame.handle_item(context, item),
            AnyValueFrame::MethodCall(frame) => frame.handle_item(context, item),
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
        context.handle_node_as_owned(frame, inner)
    }
}

impl EvaluationFrame for GroupBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Group(self)
    }

    fn handle_item(
        self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let inner = item.expect_owned();
        context.return_owned(inner.with_span(self.span))
    }
}

pub(super) struct ArrayBuilder {
    span: Span,
    unevaluated_items: Vec<ExpressionNodeId>,
    evaluated_items: Vec<ExpressionValue>,
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

    pub(super) fn next(self, context: ValueContext) -> ExecutionResult<NextAction> {
        Ok(
            match self
                .unevaluated_items
                .get(self.evaluated_items.len())
                .cloned()
            {
                Some(next) => context.handle_node_as_owned(self, next),
                None => context.return_owned(
                    ExpressionValue::Array(ExpressionArray {
                        items: self.evaluated_items,
                    })
                    .into_owned(self.span),
                )?,
            },
        )
    }
}

impl EvaluationFrame for ArrayBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Array(self)
    }

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let value = item.expect_owned();
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

    fn next(mut self: Box<Self>, context: ValueContext) -> ExecutionResult<NextAction> {
        Ok(
            match self
                .unevaluated_entries
                .get(self.evaluated_entries.len())
                .cloned()
            {
                Some((ObjectKey::Identifier(ident), value_node)) => {
                    let key = ident.to_string();
                    if self.evaluated_entries.contains_key(&key) {
                        return ident
                            .execution_err(format!("The key {} has already been set", key));
                    }
                    self.pending = Some(PendingEntryPath::OnValueBranch {
                        key,
                        key_span: ident.span(),
                    });
                    context.handle_node_as_owned(self, value_node)
                }
                Some((ObjectKey::Indexed { access, index }, value_node)) => {
                    self.pending = Some(PendingEntryPath::OnIndexKeyBranch { access, value_node });
                    context.handle_node_as_owned(self, index)
                }
                None => context
                    .return_owned(self.evaluated_entries.into_value().into_owned(self.span))?,
            },
        )
    }
}

impl EvaluationFrame for Box<ObjectBuilder> {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Object(self)
    }

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let pending = self.pending.take();
        Ok(match pending {
            Some(PendingEntryPath::OnIndexKeyBranch { access, value_node }) => {
                let value = item.expect_owned();
                let key: String = value.resolve_as("An object key")?;
                if self.evaluated_entries.contains_key(&key) {
                    return access.execution_err(format!("The key {} has already been set", key));
                }
                self.pending = Some(PendingEntryPath::OnValueBranch {
                    key,
                    key_span: access.span(),
                });
                context.handle_node_as_owned(self, value_node)
            }
            Some(PendingEntryPath::OnValueBranch { key, key_span }) => {
                let value = item.expect_owned().into_inner();
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
        context.handle_node_as_late_bound(frame, input)
    }
}

impl EvaluationFrame for UnaryOperationBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::UnaryOperation(self)
    }

    fn handle_item(
        self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let late_bound_value = item.expect_late_bound();
        let operand_kind = late_bound_value.kind();

        // Try method resolution first
        if let Some(interface) = operand_kind.resolve_unary_operation(&self.operation) {
            let resolved_value = late_bound_value.resolve(interface.argument_ownership())?;
            let result = interface.execute(resolved_value, &self.operation)?;
            return context.return_resolved_value(result);
        }
        self.operation.execution_err(format!(
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
        left: ResolvedValue,
        method: Option<MethodInterface>,
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
        context.handle_node_as_late_bound(frame, left)
    }
}

impl EvaluationFrame for BinaryOperationBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::BinaryOperation(self)
    }

    fn handle_item(
        mut self,
        mut context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            BinaryPath::OnLeftBranch { right } => {
                let left_late_bound = item.expect_late_bound();

                // Check for lazy evaluation first (short-circuit operators)
                let left_value = left_late_bound
                    .as_ref()
                    .spanned(left_late_bound.span_range());
                if let Some(result) = self.operation.lazy_evaluate(left_value)? {
                    context.return_owned(result)?
                } else {
                    // Try method resolution based on left operand's kind and resolve left operand immediately
                    let method = left_late_bound
                        .as_ref()
                        .kind()
                        .resolve_binary_operation(&self.operation);

                    let (left_ownership, right_ownership) = if let Some(method) = &method {
                        let (ownerships, min_required) = method.argument_ownerships();
                        assert!(
                            ownerships.len() == 2 && min_required == 2,
                            "Binary operation methods must have exactly two ownerships"
                        );
                        (ownerships[0], ownerships[1])
                    } else {
                        // Fallback to legacy system - use owned values for legacy evaluation
                        (ResolvedValueOwnership::Owned, ResolvedValueOwnership::Owned)
                    };
                    let left = left_ownership.map_from_late_bound(left_late_bound)?;

                    self.state = BinaryPath::OnRightBranch { left, method };
                    context.handle_node_as_any_value(
                        self,
                        right,
                        RequestedValueOwnership::Concrete(right_ownership),
                    )
                }
            }
            BinaryPath::OnRightBranch { left, method } => {
                let right = item.expect_resolved_value();
                let right_kind = right.as_ref().kind();

                // Try method resolution first (we already determined this during left evaluation)
                if let Some(method) = method {
                    // TODO[operation-refactor]: Use proper span range from operation
                    let span_range =
                        SpanRange::new_between(left.span_range().start(), right.span_range().end());

                    let mut call_context = MethodCallContext {
                        output_span_range: span_range,
                        interpreter: context.interpreter(),
                    };
                    let result = method.execute(vec![left, right], &mut call_context)?;
                    return context.return_resolved_value(result);
                }

                let left = left.expect_owned();
                let right = right.expect_owned();
                context.return_owned(self.operation.evaluate(left, right)?)?
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
        context.handle_node_as_any_value(frame, node, ownership_request)
    }
}

impl EvaluationFrame for ValuePropertyAccessBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::PropertyAccess(self)
    }

    fn handle_item(
        self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let mapped = item.expect_any_value_and_map(
            |shared| shared.resolve_property(&self.access),
            |mutable| mutable.resolve_property(&self.access, false),
            |owned| owned.resolve_property(&self.access),
        )?;
        context.return_item(mapped)
    }
}

pub(super) struct ValueIndexAccessBuilder {
    access: IndexAccess,
    state: IndexPath,
}

enum IndexPath {
    OnSourceBranch { index: ExpressionNodeId },
    OnIndexBranch { source: EvaluationItem },
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
        context.handle_node_as_any_value(frame, source, ownership_request)
    }
}

impl EvaluationFrame for ValueIndexAccessBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::IndexAccess(self)
    }

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            IndexPath::OnSourceBranch { index } => {
                self.state = IndexPath::OnIndexBranch { source: item };
                // This is a value, so we are _accessing it_ and can't create values
                // (that's only possible in a place!) - therefore we don't need an owned key,
                // and can use &index for reading values from our array
                context.handle_node_as_shared(self, index)
            }
            IndexPath::OnIndexBranch { source } => {
                let index = item.expect_shared();
                let is_range = matches!(index.kind(), ValueKind::Range);

                context.return_item(source.expect_any_value_and_map(
                    |shared| shared.resolve_indexed(self.access, index.as_spanned()),
                    |mutable| mutable.resolve_indexed(self.access, index.as_spanned(), false),
                    |owned| owned.resolve_indexed(self.access, index.as_spanned()),
                )?)?
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
    OnRightBranch { left: Option<ExpressionValue> },
}

impl RangeBuilder {
    pub(super) fn start(
        context: ValueContext,
        left: &Option<ExpressionNodeId>,
        range_limits: &syn::RangeLimits,
        right: &Option<ExpressionNodeId>,
    ) -> ExecutionResult<NextAction> {
        Ok(match (left, right) {
            (None, None) => match range_limits {
                syn::RangeLimits::HalfOpen(token) => {
                    let inner = ExpressionRangeInner::RangeFull { token: *token };
                    context.return_owned(inner.into_value().into_owned(token.span_range()))?
                }
                syn::RangeLimits::Closed(_) => {
                    unreachable!(
                        "A closed range should have been given a right in continue_range(..)"
                    )
                }
            },
            (None, Some(right)) => context.handle_node_as_owned(
                Self {
                    range_limits: *range_limits,
                    state: RangePath::OnRightBranch { left: None },
                },
                *right,
            ),
            (Some(left), right) => context.handle_node_as_owned(
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

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        // TODO[range-refactor]: Change to not always clone the value
        let value = item.expect_owned().into_inner();
        Ok(match (self.state, self.range_limits) {
            (RangePath::OnLeftBranch { right: Some(right) }, _) => {
                self.state = RangePath::OnRightBranch { left: Some(value) };
                context.handle_node_as_owned(self, right)
            }
            (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = ExpressionRangeInner::RangeFrom {
                    start_inclusive: value,
                    token,
                };
                context.return_owned(inner.into_owned_value(token))?
            }
            (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::Closed(_)) => {
                unreachable!("A closed range should have been given a right in continue_range(..)")
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = ExpressionRangeInner::Range {
                    start_inclusive: left,
                    token,
                    end_exclusive: value,
                };
                context.return_owned(inner.into_owned_value(token))?
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::Closed(token)) => {
                let inner = ExpressionRangeInner::RangeInclusive {
                    start_inclusive: left,
                    token,
                    end_inclusive: value,
                };
                context.return_owned(inner.into_owned_value(token))?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = ExpressionRangeInner::RangeTo {
                    token,
                    end_exclusive: value,
                };
                context.return_owned(inner.into_owned_value(token))?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::Closed(token)) => {
                let inner = ExpressionRangeInner::RangeToInclusive {
                    token,
                    end_inclusive: value,
                };
                context.return_owned(inner.into_owned_value(token.span_range()))?
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
        context.handle_node_as_owned(frame, value)
    }
}

impl EvaluationFrame for AssignmentBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Assignment(self)
    }

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            AssignmentPath::OnValueBranch { assignee } => {
                let value = item.expect_owned().into_inner();
                self.state = AssignmentPath::OnAwaitingAssignment;
                context.handle_node_as_assignment(self, assignee, value)
            }
            AssignmentPath::OnAwaitingAssignment => {
                let AssignmentCompletion { span_range } = item.expect_assignment_completion();
                context.return_owned(ExpressionValue::None.into_owned(span_range))?
            }
        })
    }
}

pub(super) struct CompoundAssignmentBuilder {
    operation: CompoundAssignmentOperation,
    state: CompoundAssignmentPath,
}

enum CompoundAssignmentPath {
    OnValueBranch { target: ExpressionNodeId },
    OnTargetBranch { value: OwnedValue },
}

impl CompoundAssignmentBuilder {
    pub(super) fn start(
        context: ValueContext,
        target: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
        value: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            operation,
            state: CompoundAssignmentPath::OnValueBranch { target },
        };
        context.handle_node_as_owned(frame, value)
    }
}

impl EvaluationFrame for CompoundAssignmentBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::CompoundAssignment(self)
    }

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            CompoundAssignmentPath::OnValueBranch { target } => {
                let value = item.expect_owned();
                self.state = CompoundAssignmentPath::OnTargetBranch { value };
                // TODO[compound-assignment-refactor]: Resolve as LateBound, and then convert to what is needed based on the operation
                context.handle_node_as_mutable(self, target)
            }
            CompoundAssignmentPath::OnTargetBranch { value } => {
                let mut mutable = item.expect_mutable();
                let span_range = SpanRange::new_between(mutable.span_range(), value.span_range());
                SpannedAnyRefMut::from(mutable)
                    .handle_compound_assignment(&self.operation, value)?;
                context.return_owned(ExpressionValue::None.into_owned(span_range))?
            }
        })
    }
}

pub(super) struct MethodCallBuilder {
    method: MethodAccess,
    unevaluated_parameters_stack: Vec<(ExpressionNodeId, ResolvedValueOwnership)>,
    state: MethodCallPath,
}

enum MethodCallPath {
    CallerPath,
    ArgumentsPath {
        method: MethodInterface,
        evaluated_arguments_including_caller: Vec<ResolvedValue>,
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
                    (*x, ResolvedValueOwnership::Owned)
                })
                .collect(),
            state: MethodCallPath::CallerPath,
        };
        context.handle_node_as_late_bound(frame, caller)
    }
}

impl EvaluationFrame for MethodCallBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::MethodCall(self)
    }

    fn handle_item(
        mut self,
        mut context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        // Handle expected item based on current state
        match self.state {
            MethodCallPath::CallerPath => {
                let caller = item.expect_late_bound();
                let method = caller
                    .as_ref()
                    .kind()
                    .resolve_method(self.method.method.to_string().as_str());
                let method = match method {
                    Some(m) => m,
                    None => {
                        return self.method.method.execution_err(format!(
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
                    return self.method.method.execution_err(format!(
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
                let caller = argument_ownerships[0].map_from_late_bound(caller)?;

                // We skip 1 to ignore the caller
                let non_self_argument_ownerships = argument_ownerships.iter().skip(1);
                for ((_, requested_ownership), ownership) in self
                    .unevaluated_parameters_stack
                    .iter_mut()
                    .rev() // Swap the stack back to the normal order
                    .zip(non_self_argument_ownerships)
                {
                    *requested_ownership = *ownership;
                }

                self.state = MethodCallPath::ArgumentsPath {
                    evaluated_arguments_including_caller: {
                        let mut params =
                            Vec::with_capacity(1 + self.unevaluated_parameters_stack.len());
                        params.push(caller);
                        params
                    },
                    method,
                };
            }
            MethodCallPath::ArgumentsPath {
                evaluated_arguments_including_caller: ref mut evaluated_parameters_including_caller,
                ..
            } => {
                let argument = item.expect_resolved_value();
                evaluated_parameters_including_caller.push(argument);
            }
        };
        // Now plan the next action
        Ok(match self.unevaluated_parameters_stack.pop() {
            Some((parameter, ownership)) => context.handle_node_as_any_value(
                self,
                parameter,
                RequestedValueOwnership::Concrete(ownership),
            ),
            None => {
                let (arguments, method) = match self.state {
                    MethodCallPath::CallerPath => unreachable!("Already updated above"),
                    MethodCallPath::ArgumentsPath {
                        evaluated_arguments_including_caller,
                        method,
                    } => (evaluated_arguments_including_caller, method),
                };
                let mut call_context = MethodCallContext {
                    output_span_range: self.method.span_range(),
                    interpreter: context.interpreter(),
                };
                let output = method.execute(arguments, &mut call_context)?;
                context.return_resolved_value(output)?
            }
        })
    }
}
