#![allow(unused)] use crate::expressions::evaluation::type_resolution::{ResolvedMethod, ResolvedTypeDetails};

// TODO: Remove when values are properly late-bound
use super::*;

/// # Late Binding
///
/// Sometimes, a value which can be accessed, but we don't yet know *how* we need to access it.
/// In this case, we attempt to load it as a Mutable place, and failing that, as a Shared place.
///
/// ## Example of requirement
/// For example, if we have `x.y(z)`, we first need to resolve the type of `x` to know
/// whether `y` takes a shared reference, mutable reference or an owned value.
///
/// So instead, we take the most powerful access we can have, and convert it later.
pub(crate) enum ResolvedValue {
    /// This has been requested as an owned value.
    Owned(ExpressionValue),
    /// This has been requested as a mutable reference.
    Mutable(MutableValue),
    /// This has been requested as a shared reference.
    Shared {
        shared_ref: SharedValue,
        reason_not_mutable: Option<syn::Error>,
    },
}

impl ResolvedValue {
    /// The requirement is just for error messages, and should be "A <xyz>", and will be filled like:
    /// "{usage} expects an owned value, but receives a reference..."
    pub(crate) fn into_owned_value(self, usage: &str) -> ExecutionResult<ExpressionValue> {
        let reference = match self {
            ResolvedValue::Owned(value) => return Ok(value),
            ResolvedValue::Shared { .. } | ResolvedValue::Mutable { .. } => self.as_ref(),
        };
        if !reference.kind().supports_transparent_cloning() {
            return reference.execution_err(format!(
                "{} expects an owned value, but receives a reference and {} does not support transparent cloning. You may wish to use .clone() explicitly.",
                usage,
                reference.articled_value_type(),
            ));
        }
        Ok(reference.clone())
    }

    /// The requirement is just for error messages, and should be "A <xyz>", and will be filled like:
    /// "{usage} expects an owned value, but receives a reference..."
    pub(crate) fn into_mutable_reference(self, usage: &str) -> ExecutionResult<MutableValue> {
        Ok(match self {
            // It might be a bug if a user calls a mutable method on a floating owned value,
            // but it might be annoying to force them to do `.as_mut()` everywhere.
            // Let's leave this as-is for now.
            ResolvedValue::Owned(value) => MutableValue::new_from_owned(value),
            ResolvedValue::Mutable(reference) => reference,
            ResolvedValue::Shared { shared_ref, reason_not_mutable: Some(reason_not_mutable), } => return shared_ref.execution_err(format!(
                "{} expects a mutable reference, but only receives a shared reference because {reason_not_mutable}.",
                usage
            )),
            ResolvedValue::Shared { shared_ref, reason_not_mutable: None, } => return shared_ref.execution_err(format!(
                "{} expects a mutable reference, but only receives a shared reference.",
                usage
            )),
        })
    }

    pub(crate) fn into_shared_reference(self) -> SharedValue {
        match self {
            ResolvedValue::Owned(value) => SharedValue::new_from_owned(value),
            ResolvedValue::Mutable(reference) => reference.to_shared(),
            ResolvedValue::Shared { shared_ref, .. } => shared_ref,
        }
    }

    pub(crate) fn kind(&self) -> ValueKind {
        self.as_value_ref().kind()
    }

    pub(crate) fn as_value_ref(&self) -> &ExpressionValue {
        match self {
            ResolvedValue::Owned(value) => value,
            ResolvedValue::Mutable(reference) => reference.as_ref(),
            ResolvedValue::Shared { shared_ref, .. } => shared_ref.as_ref(),
        }
    }

    pub(crate) fn as_value_mut(&mut self) -> ExecutionResult<&mut ExpressionValue> {
        match self {
            ResolvedValue::Owned(value) => Ok(value),
            ResolvedValue::Mutable(reference) => Ok(reference.as_mut()),
            ResolvedValue::Shared {
                shared_ref,
                reason_not_mutable: Some(reason_not_mutable),
            } => shared_ref.execution_err(format!(
                "Cannot get a mutable reference: {}",
                reason_not_mutable
            )),
            ResolvedValue::Shared {
                shared_ref,
                reason_not_mutable: None,
            } => shared_ref.execution_err("Cannot get a mutable reference: Unknown reason"),
        }
    }
}

impl HasSpanRange for ResolvedValue {
    fn span_range(&self) -> SpanRange {
        self.as_value_ref().span_range()
    }
}

impl WithSpanExt for ResolvedValue {
    fn with_span(self, span: Span) -> Self {
        match self {
            ResolvedValue::Owned(value) => ResolvedValue::Owned(value.with_span(span)),
            ResolvedValue::Mutable(reference) => ResolvedValue::Mutable(reference.with_span(span)),
            ResolvedValue::Shared {
                shared_ref,
                reason_not_mutable,
            } => ResolvedValue::Shared {
                shared_ref: shared_ref.with_span(span),
                reason_not_mutable,
            },
        }
    }
}

impl AsRef<ExpressionValue> for ResolvedValue {
    fn as_ref(&self) -> &ExpressionValue {
        self.as_value_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RequestedValueOwnership {
    LateBound,
    Owned,
    SharedReference,
    MutableReference,
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
        context.handle_node_as_value(frame, inner, RequestedValueOwnership::Owned)
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
        let inner = item.expect_owned_value();
        Ok(context.return_owned_value(inner.with_span(self.span)))
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
    ) -> NextAction {
        Self {
            span: brackets.join(),
            unevaluated_items: items.to_vec(),
            evaluated_items: Vec::with_capacity(items.len()),
        }
        .next(context)
    }

    pub(super) fn next(self, context: ValueContext) -> NextAction {
        match self
            .unevaluated_items
            .get(self.evaluated_items.len())
            .cloned()
        {
            Some(next) => context.handle_node_as_value(self, next, RequestedValueOwnership::Owned),
            None => context.return_owned_value(ExpressionValue::Array(ExpressionArray {
                items: self.evaluated_items,
                span_range: self.span.span_range(),
            })),
        }
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
        let value = item.expect_owned_value();
        self.evaluated_items.push(value);
        Ok(self.next(context))
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
                    context.handle_node_as_value(self, value_node, RequestedValueOwnership::Owned)
                }
                Some((ObjectKey::Indexed { access, index }, value_node)) => {
                    self.pending = Some(PendingEntryPath::OnIndexKeyBranch { access, value_node });
                    context.handle_node_as_value(self, index, RequestedValueOwnership::Owned)
                }
                None => context
                    .return_owned_value(self.evaluated_entries.to_value(self.span.span_range())),
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
                let value = item.expect_owned_value();
                let key = value.expect_string("An object key")?.value;
                if self.evaluated_entries.contains_key(&key) {
                    return access.execution_err(format!("The key {} has already been set", key));
                }
                self.pending = Some(PendingEntryPath::OnValueBranch {
                    key,
                    key_span: access.span(),
                });
                context.handle_node_as_value(self, value_node, RequestedValueOwnership::Owned)
            }
            Some(PendingEntryPath::OnValueBranch { key, key_span }) => {
                let value = item.expect_owned_value();
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
        // TODO: Change to be LateBound to resolve what it actually should be
        context.handle_node_as_value(frame, input, RequestedValueOwnership::Owned)
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
        let value = item.expect_owned_value();
        Ok(context.return_owned_value(self.operation.evaluate(value)?))
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
        // TODO: Change to be LateBound to resolve what it actually should be
        context.handle_node_as_value(frame, left, RequestedValueOwnership::Owned)
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
                let value = item.expect_owned_value();
                if let Some(result) = self.operation.lazy_evaluate(&value)? {
                    context.return_owned_value(result)
                } else {
                    self.state = BinaryPath::OnRightBranch { left: value };
                    context.handle_node_as_value(
                        self,
                        right,
                        // TODO: Change to late bound as the operation may dictate what ownership it needs for its right operand
                        RequestedValueOwnership::Owned,
                    )
                }
            }
            BinaryPath::OnRightBranch { left } => {
                let value = item.expect_owned_value();
                context.return_owned_value(self.operation.evaluate(left, value)?)
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
        // TODO: Change to propagate context from value, and not always clone!
        let ownership = RequestedValueOwnership::Owned;
        context.handle_node_as_value(frame, node, ownership)
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
        let value = item.expect_owned_value();
        Ok(context.return_owned_value(value.into_property(&self.access)?))
    }
}

pub(super) struct ValueIndexAccessBuilder {
    access: IndexAccess,
    state: IndexPath,
}

enum IndexPath {
    OnSourceBranch { index: ExpressionNodeId },
    OnIndexBranch { object: ExpressionValue },
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
        // TODO: Change to propagate context from value, and not always clone
        context.handle_node_as_value(frame, source, RequestedValueOwnership::Owned)
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
                let value = item.expect_owned_value();
                self.state = IndexPath::OnIndexBranch { object: value };
                context.handle_node_as_value(
                    self,
                    index,
                    // We can use a &index for reading values from our array
                    RequestedValueOwnership::SharedReference,
                )
            }
            IndexPath::OnIndexBranch { object } => {
                let value = item.expect_shared_ref();
                context.return_owned_value(object.into_indexed(self.access, value.as_ref())?)
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
    ) -> NextAction {
        match (left, right) {
            (None, None) => match range_limits {
                syn::RangeLimits::HalfOpen(token) => {
                    let inner = ExpressionRangeInner::RangeFull { token: *token };
                    context.return_owned_value(inner.to_value(token.span_range()))
                }
                syn::RangeLimits::Closed(_) => {
                    unreachable!(
                        "A closed range should have been given a right in continue_range(..)"
                    )
                }
            },
            (None, Some(right)) => context.handle_node_as_value(
                Self {
                    range_limits: *range_limits,
                    state: RangePath::OnRightBranch { left: None },
                },
                *right,
                RequestedValueOwnership::Owned,
            ),
            (Some(left), right) => context.handle_node_as_value(
                Self {
                    range_limits: *range_limits,
                    state: RangePath::OnLeftBranch { right: *right },
                },
                *left,
                RequestedValueOwnership::Owned,
            ),
        }
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
        // TODO: Change to not always clone the value
        let value = item.expect_owned_value();
        Ok(match (self.state, self.range_limits) {
            (RangePath::OnLeftBranch { right: Some(right) }, _) => {
                self.state = RangePath::OnRightBranch { left: Some(value) };
                context.handle_node_as_value(self, right, RequestedValueOwnership::Owned)
            }
            (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = ExpressionRangeInner::RangeFrom {
                    start_inclusive: value,
                    token,
                };
                context.return_owned_value(inner.to_value(token.span_range()))
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
                context.return_owned_value(inner.to_value(token.span_range()))
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::Closed(token)) => {
                let inner = ExpressionRangeInner::RangeInclusive {
                    start_inclusive: left,
                    token,
                    end_inclusive: value,
                };
                context.return_owned_value(inner.to_value(token.span_range()))
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = ExpressionRangeInner::RangeTo {
                    token,
                    end_exclusive: value,
                };
                context.return_owned_value(inner.to_value(token.span_range()))
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::Closed(token)) => {
                let inner = ExpressionRangeInner::RangeToInclusive {
                    token,
                    end_inclusive: value,
                };
                context.return_owned_value(inner.to_value(token.span_range()))
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
        context.handle_node_as_value(frame, value, RequestedValueOwnership::Owned)
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
                let value = item.expect_owned_value();
                self.state = AssignmentPath::OnAwaitingAssignment;
                context.handle_node_as_assignment(self, assignee, value)
            }
            AssignmentPath::OnAwaitingAssignment => {
                let AssignmentCompletion { span_range } = item.expect_assignment_complete();
                context.return_owned_value(ExpressionValue::None(span_range))
            }
        })
    }
}

pub(super) struct CompoundAssignmentBuilder {
    operation: CompoundAssignmentOperation,
    state: CompoundAssignmentPath,
}

enum CompoundAssignmentPath {
    OnValueBranch { place: ExpressionNodeId },
    OnPlaceBranch { value: ExpressionValue },
}

impl CompoundAssignmentBuilder {
    pub(super) fn start(
        context: ValueContext,
        place: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
        value: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            operation,
            state: CompoundAssignmentPath::OnValueBranch { place },
        };
        context.handle_node_as_value(frame, value, RequestedValueOwnership::Owned)
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
            CompoundAssignmentPath::OnValueBranch { place } => {
                let value = item.expect_owned_value();
                self.state = CompoundAssignmentPath::OnPlaceBranch { value };
                // TODO: Resolve as LateBound, and then convert to what is needed based on the operation
                context.handle_node_as_place(self, place, RequestedPlaceOwnership::MutableReference)
            }
            CompoundAssignmentPath::OnPlaceBranch { value } => {
                let mut mutable_reference = item.expect_mutable_ref();
                let span_range =
                    SpanRange::new_between(mutable_reference.span_range(), value.span_range());
                mutable_reference.as_mut().handle_compound_assignment(
                    &self.operation,
                    value,
                    span_range,
                )?;
                context.return_owned_value(ExpressionValue::None(span_range))
            }
        })
    }
}

pub(super) struct MethodCallBuilder {
    method: MethodAccess,
    unevaluated_parameters_stack: Vec<(ExpressionNodeId, RequestedValueOwnership)>,
    evaluated_parameters: Vec<ResolvedValue>,
    state: MethodCallPath,
}

enum MethodCallPath {
    CallerPath,
    ArgumentsPath {
        caller: ResolvedValue,
        method: ResolvedMethod,
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
            unevaluated_parameters_stack: parameters.iter().rev()
                .map(|x| {
                    // This is just a placeholder - we'll fix it up shortly
                    (*x, RequestedValueOwnership::LateBound)
                }).collect(),
            evaluated_parameters: Vec::with_capacity(parameters.len()),
            state: MethodCallPath::CallerPath,
        };
        context.handle_node_as_value(frame, caller, RequestedValueOwnership::LateBound)
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
                let caller = item.expect_any_value();
                let method = caller.kind().resolve_method(&self.method, self.unevaluated_parameters_stack.len())?;
                assert!(method.ownerships().len() == self.unevaluated_parameters_stack.len(), "The method resolution should ensure the argument count is correct");
                let argument_ownerships_stack = method.ownerships().iter().rev();
                for ((_, requested_ownership), ownership) in self.unevaluated_parameters_stack.iter_mut().zip(argument_ownerships_stack) {
                    *requested_ownership = *ownership;
                }
                self.state = MethodCallPath::ArgumentsPath { caller, method, };
            }
            MethodCallPath::ArgumentsPath { .. } => {
                let argument = item.expect_any_value();
                self.evaluated_parameters.push(argument);
            }
        };
        // Now plan the next action
        Ok(match self.unevaluated_parameters_stack.pop() {
            Some((parameter, ownership)) => {
                context.handle_node_as_value(self, parameter, ownership)
            }
            None => {
                let (caller, method) = match self.state {
                    MethodCallPath::CallerPath => unreachable!("Already updated above"),
                    MethodCallPath::ArgumentsPath { caller, method } => (caller, method),
                };
                let output = method.execute(caller, self.evaluated_parameters, self.method.span_range())?;
                context.return_any_value(output)
            }
        })
    }
}
