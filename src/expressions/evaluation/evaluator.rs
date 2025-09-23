#![allow(unused)] // TODO[unused-clearup]
use super::*;

pub(in super::super) struct ExpressionEvaluator<'a, K: Expressionable> {
    nodes: &'a [ExpressionNode<K>],
    stack: EvaluationStack,
}

impl<'a> ExpressionEvaluator<'a, Source> {
    pub(in super::super) fn new(nodes: &'a [ExpressionNode<Source>]) -> Self {
        Self {
            nodes,
            stack: EvaluationStack::new(),
        }
    }

    pub(in super::super) fn evaluate(
        mut self,
        root: ExpressionNodeId,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<ExpressionValue> {
        let mut next_action =
            NextActionInner::ReadNodeAsValue(root, RequestedValueOwnership::Owned);

        loop {
            match self.step(next_action, interpreter)? {
                StepResult::Continue(continue_action) => {
                    next_action = continue_action.0;
                }
                StepResult::Return(value) => {
                    return Ok(value);
                }
            }
        }
    }

    fn step(
        &mut self,
        action: NextActionInner,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<StepResult> {
        Ok(StepResult::Continue(match action {
            NextActionInner::ReadNodeAsValue(node, ownership) => self.nodes[node.0]
                .handle_as_value(
                    interpreter,
                    Context {
                        request: ownership,
                        stack: &mut self.stack,
                    },
                )?,
            NextActionInner::ReadNodeAsAssignee(node, value) => self.nodes[node.0]
                .handle_as_assignee(
                    interpreter,
                    Context {
                        stack: &mut self.stack,
                        request: (),
                    },
                    self.nodes,
                    node,
                    value,
                )?,
            NextActionInner::ReadNodeAsPlace(node) => self.nodes[node.0]
                .handle_as_place(
                    interpreter,
                    Context {
                        stack: &mut self.stack,
                        request: (),
                    },
                )?,
            NextActionInner::HandleReturnedItem(item) => {
                let top_of_stack = match self.stack.handlers.pop() {
                    Some(top) => top,
                    None => {
                        // This aligns with the request for an owned value in evaluate
                        return Ok(StepResult::Return(item.expect_owned_value().into_inner()));
                    }
                };
                top_of_stack.handle_item(&mut self.stack, item)?
            }
        }))
    }
}

pub(super) struct EvaluationStack {
    /// The stack of operations which are waiting to handle an item / value / assignment completion to continue execution
    handlers: Vec<AnyEvaluationHandler>,
}

impl EvaluationStack {
    pub(super) fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

pub(super) enum StepResult {
    Continue(NextAction),
    Return(ExpressionValue),
}

pub(super) struct NextAction(NextActionInner);

impl NextAction {
    pub(super) fn return_owned(value: OwnedValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Owned(value)).into()
    }

    pub(super) fn return_mutable(mutable: MutableValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Mutable { mutable })
            .into()
    }

    pub(super) fn return_shared(
        shared: SharedValue,
        reason_not_mutable: Option<syn::Error>,
    ) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Shared {
            shared,
            reason_not_mutable,
        })
        .into()
    }
}

enum NextActionInner {
    /// Enters an expression node to output a value
    ReadNodeAsValue(ExpressionNodeId, RequestedValueOwnership),
    // Enters an expression node for assignment purposes
    // This covers atomic assignments (to places) and composite assignments
    // (similar to patterns but for existing values/reassignments)
    // let a = ["x", "y"]; let b; [a[1], .. b] = [1, 2, 3, 4]
    ReadNodeAsAssignee(ExpressionNodeId, ExpressionValue),
    // Enters an expression node to output a place (a source for an atomic assignment)
    // e.g. the a[1] in a[1] = "4"
    ReadNodeAsPlace(ExpressionNodeId),
    HandleReturnedItem(EvaluationItem),
}

impl From<NextActionInner> for NextAction {
    fn from(value: NextActionInner) -> Self {
        Self(value)
    }
}

pub(super) enum EvaluationItem {
    Owned(OwnedValue),
    Shared {
        shared: SharedValue,
        /// This is only populated if we request a "late bound" reference, and fail to resolve
        /// a mutable reference.
        reason_not_mutable: Option<syn::Error>,
    },
    Mutable {
        mutable: MutableValue,
    },
    AssignmentCompletion(AssignmentCompletion),
}

impl EvaluationItem {
    pub(super) fn expect_owned_value(self) -> OwnedValue {
        match self {
            EvaluationItem::Owned(value) => value,
            _ => panic!("expect_owned_value() called on a non-owned-value EvaluationItem"),
        }
    }

    pub(super) fn expect_cow_value(self) -> CowValue {
        match self {
            EvaluationItem::Owned(value) => CowValue::Owned(value),
            EvaluationItem::Shared { shared, .. } => CowValue::Shared(shared),
            _ => panic!("expect_cow_value() called on a non-cow-value EvaluationItem"),
        }
    }

    pub(super) fn expect_any_value(self) -> ResolvedValue {
        match self {
            EvaluationItem::Owned(value) => ResolvedValue::Owned(value),
            EvaluationItem::Mutable { mutable } => ResolvedValue::Mutable(mutable),
            EvaluationItem::Shared { shared, .. } => ResolvedValue::Shared {
                shared,
                reason_not_mutable: None,
            },
            _ => panic!("expect_any_value() called on a non-value EvaluationItem"),
        }
    }

    // pub(super) fn expect_any_late_bound_value(self) -> LateBoundValue {
    //     match self {
    //         EvaluationItem::Mutable { mutable } => LateBoundValue::Mutable { mutable },
    //         EvaluationItem::Shared {
    //             shared,
    //             reason_not_mutable,
    //         } => LateBoundValue::Shared {
    //             shared,
    //             reason_not_mutable,
    //         },
    //         _ => panic!("expect_any_place() called on a non-place EvaluationItem"),
    //     }
    // }

    pub(super) fn expect_shared(self) -> SharedValue {
        match self {
            EvaluationItem::Shared { shared, .. } => shared,
            _ => {
                panic!("expect_shared() called on a non-shared-reference EvaluationItem")
            }
        }
    }

    pub(super) fn expect_mutable(self) -> MutableValue {
        match self {
            EvaluationItem::Mutable { mutable } => mutable,
            _ => panic!("expect_mutable() called on a non-mutable-place EvaluationItem"),
        }
    }

    pub(super) fn expect_place(self) -> MutableValue {
        self.expect_mutable()
    }

    pub(super) fn expect_assignment_complete(self) -> AssignmentCompletion {
        match self {
            EvaluationItem::AssignmentCompletion(completion) => completion,
            _ => panic!(
                "expect_assignment_complete() called on a non-assignment-completion EvaluationItem"
            ),
        }
    }
}

/// See the [rust reference] for a good description of assignee vs place.
///
/// [rust reference]: https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
pub(super) enum AnyEvaluationHandler {
    Value(AnyValueFrame, RequestedValueOwnership),
    Place(AnyPlaceFrame),
    Assignment(AnyAssignmentFrame),
}

impl AnyEvaluationHandler {
    fn handle_item(
        self,
        stack: &mut EvaluationStack,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        match self {
            AnyEvaluationHandler::Value(handler, ownership) => handler.handle_item(
                Context {
                    stack,
                    request: ownership,
                },
                item,
            ),
            AnyEvaluationHandler::Place(handler) => handler.handle_item(
                Context {
                    stack,
                    request: (),
                },
                item,
            ),
            AnyEvaluationHandler::Assignment(handler) => {
                handler.handle_item(Context { stack, request: () }, item)
            }
        }
    }
}

pub(super) struct Context<'a, T: EvaluationItemType> {
    stack: &'a mut EvaluationStack,
    request: T::RequestConstraints,
}

impl<'a, T: EvaluationItemType> Context<'a, T> {
    pub(super) fn handle_node_as_value<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        requested_ownership: RequestedValueOwnership,
    ) -> NextAction {
        self.stack
            .handlers
            .push(T::into_unkinded_handler(handler.into_any(), self.request));
        NextActionInner::ReadNodeAsValue(node, requested_ownership).into()
    }

    pub(super) fn handle_node_as_place<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.stack
            .handlers
            .push(T::into_unkinded_handler(handler.into_any(), self.request));
        NextActionInner::ReadNodeAsPlace(node).into()
    }

    pub(super) fn handle_node_as_assignment<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        value: ExpressionValue,
    ) -> NextAction {
        self.stack
            .handlers
            .push(T::into_unkinded_handler(handler.into_any(), self.request));
        NextActionInner::ReadNodeAsAssignee(node, value).into()
    }
}

pub(super) trait EvaluationFrame: Sized {
    type ReturnType: EvaluationItemType;

    fn into_any(self) -> <Self::ReturnType as EvaluationItemType>::AnyHandler;

    fn handle_item(
        self,
        context: Context<Self::ReturnType>,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction>;
}

pub(super) trait EvaluationItemType {
    type RequestConstraints;
    type AnyHandler;
    fn into_unkinded_handler(
        handler: Self::AnyHandler,
        request: Self::RequestConstraints,
    ) -> AnyEvaluationHandler;
}

pub(super) struct ValueType;
pub(super) type ValueContext<'a> = Context<'a, ValueType>;

impl EvaluationItemType for ValueType {
    type RequestConstraints = RequestedValueOwnership;
    type AnyHandler = AnyValueFrame;

    fn into_unkinded_handler(
        handler: Self::AnyHandler,
        request: Self::RequestConstraints,
    ) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Value(handler, request)
    }
}

impl<'a> Context<'a, ValueType> {
    pub(super) fn requested_ownership(&self) -> RequestedValueOwnership {
        self.request
    }

    pub(super) fn return_any_value(self, value: ResolvedValue) -> ExecutionResult<NextAction> {
        Ok(match value {
            ResolvedValue::Owned(owned) => self.return_owned(owned)?,
            ResolvedValue::Mutable(mutable) => self.return_mutable(mutable),
            ResolvedValue::Shared {
                shared,
                reason_not_mutable,
            } => self.return_shared(shared, reason_not_mutable)?,
        })
    }

    pub(super) fn return_late_bound(self, value: LateBoundValue) -> ExecutionResult<NextAction> {
        Ok(match value {
            LateBoundValue::Mutable { mutable } => self.return_mutable(mutable),
            LateBoundValue::Shared {
                shared,
                reason_not_mutable,
            } => self.return_shared(shared, reason_not_mutable)?,
        })
    }

    pub(super) fn return_owned(self, value: impl Into<OwnedValue>) -> ExecutionResult<NextAction> {
        let value = value.into();
        Ok(match self.request {
            RequestedValueOwnership::LateBound
            | RequestedValueOwnership::CopyOnWrite
            | RequestedValueOwnership::Owned => NextAction::return_owned(value),
            RequestedValueOwnership::Shared => {
                NextAction::return_shared(Shared::new_from_owned(value), None)
            }
            RequestedValueOwnership::Mutable => {
                // This aligns with ResolveValue::into_mutable
                return value.execution_err("A mutable reference is required, but an owned value was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.as_mut()` to get a mutable reference.")
            }
        })
    }

    pub(super) fn return_mutable(self, mutable: MutableValue) -> NextAction {
        match self.request {
            RequestedValueOwnership::LateBound
            | RequestedValueOwnership::Mutable => NextAction::return_mutable(mutable),
            RequestedValueOwnership::Owned
            | RequestedValueOwnership::CopyOnWrite
            | RequestedValueOwnership::Shared => panic!("Returning a mutable reference should only be used when the requested ownership is mutable or late-bound"),
        }
    }

    pub(super) fn return_shared(
        self,
        shared: SharedValue,
        reason_not_mutable: Option<syn::Error>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.request {
            RequestedValueOwnership::LateBound
            | RequestedValueOwnership::CopyOnWrite
            | RequestedValueOwnership::Shared => NextAction::return_shared(shared, reason_not_mutable),
            RequestedValueOwnership::Owned => {
                // This can happen if a requested of "Owned" was mapped to "CopyOnWrite"
                NextAction::return_owned(shared.transparent_clone()?)
            }
            RequestedValueOwnership::Mutable => panic!("Returning a reference should only be used when the requested ownership is shared or late-bound"),
        })
    }
}

pub(super) struct PlaceType;

pub(super) type PlaceContext<'a> = Context<'a, PlaceType>;

impl EvaluationItemType for PlaceType {
    type RequestConstraints = ();
    type AnyHandler = AnyPlaceFrame;

    fn into_unkinded_handler(
        handler: Self::AnyHandler,
        (): Self::RequestConstraints,
    ) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Place(handler)
    }
}

impl<'a> Context<'a, PlaceType> {
    pub(super) fn return_place(self, place: MutableValue) -> NextAction {
        NextAction::return_mutable(place)
    }
}

pub(super) struct AssignmentType;

pub(super) type AssignmentContext<'a> = Context<'a, AssignmentType>;

impl EvaluationItemType for AssignmentType {
    type RequestConstraints = ();
    type AnyHandler = AnyAssignmentFrame;

    fn into_unkinded_handler(handler: Self::AnyHandler, _request: ()) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Assignment(handler)
    }
}

impl<'a> Context<'a, AssignmentType> {
    pub(super) fn return_assignment_completion(self, span_range: SpanRange) -> NextAction {
        NextActionInner::HandleReturnedItem(EvaluationItem::AssignmentCompletion(
            AssignmentCompletion { span_range },
        ))
        .into()
    }
}
