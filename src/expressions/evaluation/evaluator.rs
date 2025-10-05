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
    ) -> ExecutionResult<OwnedValue> {
        let mut next_action = NextActionInner::ReadNodeAsValue(
            root,
            RequestedValueOwnership::Concrete(ResolvedValueOwnership::Owned),
        );

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
                .handle_as_value(Context {
                    request: ownership,
                    interpreter,
                    stack: &mut self.stack,
                })?,
            NextActionInner::ReadNodeAsAssignee(node, value) => self.nodes[node.0]
                .handle_as_assignee(
                    Context {
                        stack: &mut self.stack,
                        interpreter,
                        request: (),
                    },
                    self.nodes,
                    node,
                    value,
                )?,
            NextActionInner::ReadNodeAsPlace(node) => {
                self.nodes[node.0].handle_as_place(Context {
                    stack: &mut self.stack,
                    interpreter,
                    request: (),
                })?
            }
            NextActionInner::HandleReturnedItem(item) => {
                let top_of_stack = match self.stack.handlers.pop() {
                    Some(top) => top,
                    None => {
                        // This aligns with the request for an owned value in evaluate
                        return Ok(StepResult::Return(item.expect_owned()));
                    }
                };
                top_of_stack.handle_item(interpreter, &mut self.stack, item)?
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
    Return(OwnedValue),
}

pub(super) struct NextAction(NextActionInner);

impl NextAction {
    pub(super) fn return_owned(value: OwnedValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Owned(value)).into()
    }

    pub(super) fn return_mutable(mutable: MutableValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Mutable(mutable)).into()
    }

    pub(super) fn return_shared(shared: SharedValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Shared(shared)).into()
    }

    pub(super) fn return_copy_on_write(copy_on_write: CopyOnWriteValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::CopyOnWrite(copy_on_write)).into()
    }

    pub(super) fn return_resolved_value(resolved: ResolvedValue) -> Self {
        match resolved {
            ResolvedValue::Owned(owned) => Self::return_owned(owned),
            ResolvedValue::Mutable(mutable) => Self::return_mutable(mutable),
            ResolvedValue::Shared(shared) => Self::return_shared(shared),
            ResolvedValue::CopyOnWrite(copy_on_write) => Self::return_copy_on_write(copy_on_write),
        }
    }

    pub(super) fn return_late_bound(late_bound: LateBoundValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::LateBound(late_bound)).into()
    }

    pub(super) fn return_place(place: MutableValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Place(place)).into()
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
    // Value items - these mirror RequestedValueOwnership exactly
    LateBound(LateBoundValue),
    Owned(OwnedValue),
    Shared(SharedValue),
    Mutable(MutableValue), // Mutable reference to a value
    CopyOnWrite(CopyOnWriteValue),

    // Place items (for assignment targets)
    // Note that places are handled subtly differently than a mutable value,
    // for example with a place, x["a"] creates an entry if it doesn't exist,
    // whereas with a mutable value it would return None without creating the entry.
    Place(MutableValue),

    // Assignment items
    AssignmentCompletion(AssignmentCompletion),
}

impl EvaluationItem {
    pub(super) fn expect_owned(self) -> OwnedValue {
        match self {
            EvaluationItem::Owned(value) => value,
            _ => panic!("expect_owned() called on non-owned EvaluationItem"),
        }
    }

    pub(super) fn expect_shared(self) -> SharedValue {
        match self {
            EvaluationItem::Shared(shared) => shared,
            _ => panic!("expect_shared() called on non-shared EvaluationItem"),
        }
    }

    pub(super) fn expect_mutable(self) -> MutableValue {
        match self {
            EvaluationItem::Mutable(mutable) => mutable,
            _ => panic!("expect_mutable() called on non-mutable EvaluationItem"),
        }
    }

    pub(super) fn expect_late_bound(self) -> LateBoundValue {
        match self {
            EvaluationItem::LateBound(late_bound) => late_bound,
            _ => panic!("expect_late_bound() called on non-late-bound EvaluationItem"),
        }
    }

    pub(super) fn expect_copy_on_write(self) -> CopyOnWriteValue {
        match self {
            EvaluationItem::CopyOnWrite(cow) => cow,
            _ => panic!("expect_copy_on_write() called on non-copy-on-write EvaluationItem"),
        }
    }

    pub(super) fn expect_assignment_completion(self) -> AssignmentCompletion {
        match self {
            EvaluationItem::AssignmentCompletion(completion) => completion,
            _ => panic!(
                "expect_assignment_completion() called on non-assignment-completion EvaluationItem"
            ),
        }
    }

    pub(super) fn expect_resolved_value(self) -> ResolvedValue {
        match self {
            EvaluationItem::Owned(value) => ResolvedValue::Owned(value),
            EvaluationItem::Mutable(mutable) => ResolvedValue::Mutable(mutable),
            EvaluationItem::Shared(shared) => ResolvedValue::Shared(shared),
            EvaluationItem::CopyOnWrite(copy_on_write) => ResolvedValue::CopyOnWrite(copy_on_write),
            _ => panic!("expect_resolved_value() called on non-value EvaluationItem"),
        }
    }

    pub(super) fn expect_place(self) -> MutableValue {
        match self {
            EvaluationItem::Place(place) => place,
            _ => panic!("expect_place() called on non-place EvaluationItem"),
        }
    }

    pub(super) fn expect_any_value_and_map(
        self,
        map_shared: impl FnOnce(SharedValue) -> ExecutionResult<SharedValue>,
        map_mutable: impl FnOnce(MutableValue) -> ExecutionResult<MutableValue>,
        map_owned: impl FnOnce(OwnedValue) -> ExecutionResult<OwnedValue>,
    ) -> ExecutionResult<EvaluationItem> {
        Ok(match self {
            EvaluationItem::LateBound(late_bound) => {
                EvaluationItem::LateBound(late_bound.map_any(map_shared, map_mutable, map_owned)?)
            }
            EvaluationItem::Owned(value) => EvaluationItem::Owned(map_owned(value)?),
            EvaluationItem::Mutable(mutable) => EvaluationItem::Mutable(map_mutable(mutable)?),
            EvaluationItem::Shared(shared) => EvaluationItem::Shared(map_shared(shared)?),
            EvaluationItem::CopyOnWrite(cow) => {
                EvaluationItem::CopyOnWrite(cow.map_any(map_shared, map_owned)?)
            }
            _ => panic!("expect_any_value_and_map() called on non-value EvaluationItem"),
        })
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
        interpreter: &mut Interpreter,
        stack: &mut EvaluationStack,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        match self {
            AnyEvaluationHandler::Value(handler, ownership) => handler.handle_item(
                Context {
                    interpreter,
                    stack,
                    request: ownership,
                },
                item,
            ),
            AnyEvaluationHandler::Place(handler) => handler.handle_item(
                Context {
                    interpreter,
                    stack,
                    request: (),
                },
                item,
            ),
            AnyEvaluationHandler::Assignment(handler) => handler.handle_item(
                Context {
                    interpreter,
                    stack,
                    request: (),
                },
                item,
            ),
        }
    }
}

pub(super) struct Context<'a, T: EvaluationItemType> {
    interpreter: &'a mut Interpreter,
    stack: &'a mut EvaluationStack,
    request: T::RequestConstraints,
}

impl<'a, T: EvaluationItemType> Context<'a, T> {
    pub(super) fn handle_node_as_owned<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.handle_node_as_any_value(
            handler,
            node,
            RequestedValueOwnership::Concrete(ResolvedValueOwnership::Owned),
        )
    }

    pub(super) fn handle_node_as_copy_on_write<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.handle_node_as_any_value(
            handler,
            node,
            RequestedValueOwnership::Concrete(ResolvedValueOwnership::CopyOnWrite),
        )
    }

    pub(super) fn handle_node_as_shared<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.handle_node_as_any_value(
            handler,
            node,
            RequestedValueOwnership::Concrete(ResolvedValueOwnership::Shared),
        )
    }

    pub(super) fn handle_node_as_mutable<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.handle_node_as_any_value(
            handler,
            node,
            RequestedValueOwnership::Concrete(ResolvedValueOwnership::Mutable),
        )
    }

    pub(super) fn handle_node_as_late_bound<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.handle_node_as_any_value(handler, node, RequestedValueOwnership::LateBound)
    }

    pub(super) fn handle_node_as_any_value<H: EvaluationFrame<ReturnType = T>>(
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

    pub(super) fn interpreter(&mut self) -> &mut Interpreter {
        self.interpreter
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

    pub(super) fn return_late_bound(
        self,
        late_bound: LateBoundValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.request {
            RequestedValueOwnership::LateBound => NextAction::return_late_bound(late_bound),
            RequestedValueOwnership::Concrete(_) => {
                panic!("Returning a late-bound reference when concrete ownership was requested")
            }
        })
    }

    pub(super) fn return_resolved_value(self, value: ResolvedValue) -> ExecutionResult<NextAction> {
        Ok(match value {
            ResolvedValue::Owned(owned) => self.return_owned(owned)?,
            ResolvedValue::Mutable(mutable) => self.return_mutable(mutable)?,
            ResolvedValue::Shared(shared) => self.return_shared(shared)?,
            ResolvedValue::CopyOnWrite(copy_on_write) => {
                self.return_copy_on_write(copy_on_write)?
            }
        })
    }

    pub(super) fn return_item(self, value: EvaluationItem) -> ExecutionResult<NextAction> {
        match value {
            EvaluationItem::Owned(owned) => self.return_owned(owned),
            EvaluationItem::Shared(shared) => self.return_shared(shared),
            EvaluationItem::Mutable(mutable) => self.return_mutable(mutable),
            EvaluationItem::LateBound(late_bound_value) => self.return_late_bound(late_bound_value),
            EvaluationItem::CopyOnWrite(copy_on_write) => self.return_copy_on_write(copy_on_write),
            EvaluationItem::Place { .. } | EvaluationItem::AssignmentCompletion { .. } => {
                panic!("Returning a non-value item from a value context")
            }
        }
    }

    /// This method doesn't panic. It's always safe to return an owned value, as it can be converted to any other ownership type.
    pub(super) fn return_owned(self, value: impl Into<OwnedValue>) -> ExecutionResult<NextAction> {
        let value = value.into();
        Ok(match self.request {
            RequestedValueOwnership::LateBound => {
                NextAction::return_late_bound(LateBoundValue::Owned(value))
            }
            RequestedValueOwnership::Concrete(requested) => {
                NextAction::return_resolved_value(requested.map_from_owned(value)?)
            }
        })
    }

    pub(super) fn return_copy_on_write(self, cow: CopyOnWriteValue) -> ExecutionResult<NextAction> {
        Ok(match self.request {
            RequestedValueOwnership::LateBound => {
                NextAction::return_late_bound(LateBoundValue::CopyOnWrite(cow))
            }
            RequestedValueOwnership::Concrete(requested) => {
                NextAction::return_resolved_value(requested.map_from_copy_on_write(cow)?)
            }
        })
    }

    pub(super) fn return_mutable(self, mutable: MutableValue) -> ExecutionResult<NextAction> {
        Ok(match self.request {
            RequestedValueOwnership::LateBound => {
                NextAction::return_late_bound(LateBoundValue::Mutable(mutable))
            }
            RequestedValueOwnership::Concrete(requested) => {
                NextAction::return_resolved_value(requested.map_from_mutable(mutable)?)
            }
        })
    }

    pub(super) fn return_shared(self, shared: SharedValue) -> ExecutionResult<NextAction> {
        Ok(match self.request {
            RequestedValueOwnership::LateBound => {
                panic!("Returning a shared reference when late-bound was requested")
            }
            RequestedValueOwnership::Concrete(requested) => {
                NextAction::return_resolved_value(requested.map_from_shared(shared)?)
            }
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
        NextAction::return_place(place)
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
