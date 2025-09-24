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

    pub(crate) fn kind(&self) -> ValueKind {
        self.as_value_ref().kind()
    }

    pub(crate) fn as_value_ref(&self) -> &ExpressionValue {
        match self {
            ResolvedValue::Owned(owned) => owned.as_ref(),
            ResolvedValue::Mutable(mutable) => mutable.as_ref(),
            ResolvedValue::Shared(shared) => shared.as_ref(),
            ResolvedValue::CopyOnWrite(copy_on_write) => copy_on_write.as_ref(),
        }
    }
}

impl HasSpanRange for ResolvedValue {
    fn span_range(&self) -> SpanRange {
        self.as_value_ref().span_range()
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

impl AsRef<ExpressionValue> for ResolvedValue {
    fn as_ref(&self) -> &ExpressionValue {
        self.as_value_ref()
    }
}

pub(crate) use crate::interpretation::CopyOnWriteValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RequestedValueOwnership {
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
            LateBoundValue::Mutable(mutable) => self.map_from_mutable(mutable),
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
            ResolvedValueOwnership::Owned => Ok(ResolvedValue::Owned(copy_on_write.into_owned_transparently()?)),
            ResolvedValueOwnership::Shared => Ok(ResolvedValue::Shared(copy_on_write.into_shared())),
            ResolvedValueOwnership::Mutable => {
                if copy_on_write.acts_as_shared_reference() {
                    copy_on_write.execution_err("A mutable reference is required, but a shared reference was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.clone().as_mut()` to get a mutable reference.")
                } else {
                    copy_on_write.execution_err("A mutable reference is required, but an owned value was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.as_mut()` to get a mutable reference.")
                }
            },
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
        match self {
            ResolvedValueOwnership::Owned => mutable.execution_err("An owned value is required, but a mutable reference was received. This indicates a possible bug. If this was intended, use `.take()` or `.clone()` to get an owned value."),
            ResolvedValueOwnership::CopyOnWrite => Ok(ResolvedValue::CopyOnWrite(CopyOnWrite::shared_in_place_of_shared(mutable.into_shared()))),
            ResolvedValueOwnership::Mutable => Ok(ResolvedValue::Mutable(mutable)),
            ResolvedValueOwnership::Shared => Ok(ResolvedValue::Shared(mutable.into_shared())),
        }
    }

    pub(crate) fn map_from_owned(&self, owned: OwnedValue) -> ExecutionResult<ResolvedValue> {
        match self {
            ResolvedValueOwnership::Owned => Ok(ResolvedValue::Owned(owned)),
            ResolvedValueOwnership::CopyOnWrite => Ok(ResolvedValue::CopyOnWrite(CopyOnWrite::owned(owned))),
            ResolvedValueOwnership::Mutable => owned.execution_err("A mutable reference is required, but an owned value was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.as_mut()` to get a mutable reference."),
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
                None => context.return_owned(ExpressionValue::Array(ExpressionArray {
                    items: self.evaluated_items,
                    span_range: self.span.span_range(),
                }))?,
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
                None => {
                    context.return_owned(self.evaluated_entries.to_value(self.span.span_range()))?
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

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let pending = self.pending.take();
        Ok(match pending {
            Some(PendingEntryPath::OnIndexKeyBranch { access, value_node }) => {
                let value = item.expect_owned();
                let key = value.into_inner().expect_string("An object key")?.value;
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
        // TODO[operation-refactor]: Change to be LateBound to resolve what it actually should be
        context.handle_node_as_owned(frame, input)
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
        let value = item.expect_owned().into_inner();
        context.return_owned(self.operation.evaluate(value)?)
    }
}

pub(super) struct BinaryOperationBuilder {
    operation: BinaryOperation,
    state: BinaryPath,
}

enum BinaryPath {
    OnLeftBranch { right: ExpressionNodeId },
    OnRightBranch { left: ExpressionValue },
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
        // TODO[operation-refactor]: Change to be LateBound to resolve what it actually should be
        context.handle_node_as_owned(frame, left)
    }
}

impl EvaluationFrame for BinaryOperationBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::BinaryOperation(self)
    }

    fn handle_item(
        mut self,
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            BinaryPath::OnLeftBranch { right } => {
                let value = item.expect_owned().into_inner();
                if let Some(result) = self.operation.lazy_evaluate(&value)? {
                    context.return_owned(result)?
                } else {
                    self.state = BinaryPath::OnRightBranch { left: value };
                    // TODO[operation-refactor]: Change to late bound as the operation may dictate what ownership it needs for its right operand
                    context.handle_node_as_owned(self, right)
                }
            }
            BinaryPath::OnRightBranch { left } => {
                let value = item.expect_owned().into_inner();
                context.return_owned(self.operation.evaluate(left, value)?)?
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
                    |shared| shared.resolve_indexed(self.access, index.as_ref()),
                    |mutable| mutable.resolve_indexed(self.access, index.as_ref(), false),
                    |owned| owned.resolve_indexed(self.access, index.as_ref()),
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
                    context.return_owned(inner.to_value(token.span_range()))?
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
                context.return_owned(inner.to_value(token.span_range()))?
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
                context.return_owned(inner.to_value(token.span_range()))?
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::Closed(token)) => {
                let inner = ExpressionRangeInner::RangeInclusive {
                    start_inclusive: left,
                    token,
                    end_inclusive: value,
                };
                context.return_owned(inner.to_value(token.span_range()))?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = ExpressionRangeInner::RangeTo {
                    token,
                    end_exclusive: value,
                };
                context.return_owned(inner.to_value(token.span_range()))?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::Closed(token)) => {
                let inner = ExpressionRangeInner::RangeToInclusive {
                    token,
                    end_inclusive: value,
                };
                context.return_owned(inner.to_value(token.span_range()))?
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
                context.return_owned(ExpressionValue::None(span_range))?
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
    OnTargetBranch { value: ExpressionValue },
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
                let value = item.expect_owned().into_inner();
                self.state = CompoundAssignmentPath::OnTargetBranch { value };
                // TODO[compound-assignment-refactor]: Resolve as LateBound, and then convert to what is needed based on the operation
                context.handle_node_as_mutable(self, target)
            }
            CompoundAssignmentPath::OnTargetBranch { value } => {
                let mut mutable = item.expect_mutable();
                let span_range = SpanRange::new_between(mutable.span_range(), value.span_range());
                mutable
                    .as_mut()
                    .handle_compound_assignment(&self.operation, value, span_range)?;
                context.return_owned(ExpressionValue::None(span_range))?
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
        context: ValueContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        // Handle expected item based on current state
        match self.state {
            MethodCallPath::CallerPath => {
                let caller = item.expect_late_bound();
                let method = caller
                    .as_ref()
                    .kind()
                    .resolve_method(&self.method, self.unevaluated_parameters_stack.len())?;
                assert!(
                    method.ownerships().len() == 1 + self.unevaluated_parameters_stack.len(),
                    "The method resolution should ensure the argument count is correct"
                );
                let caller = method.ownerships()[0].map_from_late_bound(caller)?;

                // We skip 1 to ignore the caller
                let argument_ownerships_stack = method.ownerships().iter().skip(1).rev();
                for ((_, requested_ownership), ownership) in self
                    .unevaluated_parameters_stack
                    .iter_mut()
                    .zip(argument_ownerships_stack)
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
                let output = method.execute(arguments, self.method.span_range())?;
                context.return_resolved_value(output)?
            }
        })
    }
}
