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
            NextActionInner::ReadNodeAsPlace(node, ownership) => self.nodes[node.0]
                .handle_as_place(
                    interpreter,
                    Context {
                        stack: &mut self.stack,
                        request: ownership,
                    },
                )?,
            NextActionInner::HandleReturnedItem(item) => {
                let top_of_stack = match self.stack.handlers.pop() {
                    Some(top) => top,
                    None => {
                        // This aligns with the request for an owned value in evaluate
                        return Ok(StepResult::Return(item.expect_owned_value()));
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
    pub(super) fn return_owned(value: ExpressionValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::OwnedValue(value)).into()
    }

    pub(super) fn return_mutable(mut_ref: MutableValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::MutableReference { mut_ref }).into()
    }

    pub(super) fn return_shared(
        shared_ref: SharedValue,
        reason_not_mutable: Option<syn::Error>,
    ) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::SharedReference {
            shared_ref,
            reason_not_mutable,
        })
        .into()
    }
}

enum NextActionInner {
    /// Enters an expression node to output a value
    ReadNodeAsValue(ExpressionNodeId, RequestedValueOwnership),
    // Enters an expression node for assignment purposes
    ReadNodeAsAssignee(ExpressionNodeId, ExpressionValue),
    // Enters an expression node to output a place (a location in memory)
    // A place can be thought of as a mutable reference for e.g. a += operation
    ReadNodeAsPlace(ExpressionNodeId, RequestedPlaceOwnership),
    HandleReturnedItem(EvaluationItem),
}

impl From<NextActionInner> for NextAction {
    fn from(value: NextActionInner) -> Self {
        Self(value)
    }
}

pub(super) enum EvaluationItem {
    OwnedValue(ExpressionValue),
    SharedReference {
        shared_ref: SharedValue,
        /// This is only populated if we request a "late bound" reference, and fail to resolve
        /// a mutable reference.
        reason_not_mutable: Option<syn::Error>,
    },
    MutableReference {
        mut_ref: MutableValue,
    },
    AssignmentCompletion(AssignmentCompletion),
}

impl EvaluationItem {
    pub(super) fn expect_owned_value(self) -> ExpressionValue {
        match self {
            EvaluationItem::OwnedValue(value) => value,
            _ => panic!("expect_owned_value() called on a non-owned-value EvaluationItem"),
        }
    }

    pub(super) fn expect_any_value(self) -> ResolvedValue {
        match self {
            EvaluationItem::OwnedValue(value) => ResolvedValue::Owned(value),
            EvaluationItem::MutableReference { mut_ref } => ResolvedValue::Mutable(mut_ref),
            EvaluationItem::SharedReference { shared_ref, .. } => ResolvedValue::Shared {
                shared_ref,
                reason_not_mutable: None,
            },
            _ => panic!("expect_any_value() called on a non-value EvaluationItem"),
        }
    }

    pub(super) fn expect_any_place(self) -> Place {
        match self {
            EvaluationItem::MutableReference { mut_ref } => Place::MutableReference { mut_ref },
            EvaluationItem::SharedReference {
                shared_ref,
                reason_not_mutable,
            } => Place::SharedReference {
                shared_ref,
                reason_not_mutable,
            },
            _ => panic!("expect_any_place() called on a non-place EvaluationItem"),
        }
    }

    pub(super) fn expect_shared_ref(self) -> SharedValue {
        match self {
            EvaluationItem::SharedReference { shared_ref, .. } => shared_ref,
            _ => {
                panic!("expect_shared_reference() called on a non-shared-reference EvaluationItem")
            }
        }
    }

    pub(super) fn expect_mutable_ref(self) -> MutableValue {
        match self {
            EvaluationItem::MutableReference { mut_ref } => mut_ref,
            _ => panic!("expect_mutable_place() called on a non-mutable-place EvaluationItem"),
        }
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
    Place(AnyPlaceFrame, RequestedPlaceOwnership),
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
            AnyEvaluationHandler::Place(handler, ownership) => handler.handle_item(
                Context {
                    stack,
                    request: ownership,
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
        requested_ownership: RequestedPlaceOwnership,
    ) -> NextAction {
        self.stack
            .handlers
            .push(T::into_unkinded_handler(handler.into_any(), self.request));
        NextActionInner::ReadNodeAsPlace(node, requested_ownership).into()
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

    pub(super) fn return_any_value(self, value: ResolvedValue) -> NextAction {
        match value {
            ResolvedValue::Owned(value) => self.return_owned_value(value),
            ResolvedValue::Mutable(mut_ref) => self.return_mut_ref(mut_ref),
            ResolvedValue::Shared {
                shared_ref,
                reason_not_mutable,
            } => self.return_ref(shared_ref, reason_not_mutable),
        }
    }

    pub(super) fn return_any_place(self, value: Place) -> NextAction {
        match value {
            Place::MutableReference { mut_ref } => self.return_mut_ref(mut_ref),
            Place::SharedReference {
                shared_ref,
                reason_not_mutable,
            } => self.return_ref(shared_ref, reason_not_mutable),
        }
    }

    pub(super) fn return_owned_value(self, value: ExpressionValue) -> NextAction {
        match self.request {
            RequestedValueOwnership::LateBound | RequestedValueOwnership::Owned => {
                NextAction::return_owned(value)
            }
            RequestedValueOwnership::SharedReference => {
                NextAction::return_shared(SharedSubPlace::new_from_owned(value), None)
            }
            RequestedValueOwnership::MutableReference => {
                NextAction::return_mutable(MutableSubPlace::new_from_owned(value))
            }
        }
    }

    pub(super) fn return_mut_ref(self, mut_ref: MutableValue) -> NextAction {
        match self.request {
            RequestedValueOwnership::LateBound
            | RequestedValueOwnership::MutableReference => NextAction::return_mutable(mut_ref),
            RequestedValueOwnership::Owned
            | RequestedValueOwnership::SharedReference => panic!("Returning a mutable reference should only be used when the requested ownership is mutable or late-bound"),
        }
    }

    pub(super) fn return_ref(
        self,
        shared_ref: SharedValue,
        reason_not_mutable: Option<syn::Error>,
    ) -> NextAction {
        match self.request {
            RequestedValueOwnership::LateBound
            | RequestedValueOwnership::SharedReference => NextAction::return_shared(shared_ref, reason_not_mutable),
            RequestedValueOwnership::Owned
            | RequestedValueOwnership::MutableReference => panic!("Returning a reference should only be used when the requested ownership is shared or late-bound"),
        }
    }
}

pub(super) struct PlaceType;

pub(super) type PlaceContext<'a> = Context<'a, PlaceType>;

impl EvaluationItemType for PlaceType {
    type RequestConstraints = RequestedPlaceOwnership;
    type AnyHandler = AnyPlaceFrame;

    fn into_unkinded_handler(
        handler: Self::AnyHandler,
        request: Self::RequestConstraints,
    ) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Place(handler, request)
    }
}

impl<'a> Context<'a, PlaceType> {
    pub(super) fn requested_ownership(&self) -> RequestedPlaceOwnership {
        self.request
    }

    pub(super) fn return_any(self, place: Place) -> NextAction {
        match place {
            Place::MutableReference { mut_ref } => self.return_mutable(mut_ref),
            Place::SharedReference {
                shared_ref,
                reason_not_mutable,
            } => self.return_shared(shared_ref, reason_not_mutable),
        }
    }

    pub(super) fn return_mutable(self, mut_ref: MutableValue) -> NextAction {
        match self.request {
            RequestedPlaceOwnership::LateBound
            | RequestedPlaceOwnership::MutableReference => NextAction::return_mutable(mut_ref),
            RequestedPlaceOwnership::SharedReference => panic!("Returning a mutable reference should only be used when the requested ownership is mutable or late-bound"),
        }
    }

    pub(super) fn return_shared(
        self,
        shared_ref: SharedValue,
        reason_not_mutable: Option<syn::Error>,
    ) -> NextAction {
        match self.request {
            RequestedPlaceOwnership::LateBound
            | RequestedPlaceOwnership::SharedReference => NextAction::return_shared(shared_ref, reason_not_mutable),
            RequestedPlaceOwnership::MutableReference => panic!("Returning a shared reference should only be used when the requested ownership is shared or late-bound"),
        }
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
