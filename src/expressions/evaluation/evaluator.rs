#![allow(unused)] // TODO[unused-clearup]
use super::*;

pub(in crate::expressions) struct ExpressionEvaluator<'a> {
    nodes: &'a Arena<ExpressionNodeId, ExpressionNode>,
    stack: EvaluationStack,
}

impl<'a> ExpressionEvaluator<'a> {
    pub(in super::super) fn new(nodes: &'a Arena<ExpressionNodeId, ExpressionNode>) -> Self {
        Self {
            nodes,
            stack: EvaluationStack::new(),
        }
    }

    pub(in super::super) fn evaluate(
        mut self,
        root: ExpressionNodeId,
        interpreter: &mut Interpreter,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        let mut next_action = NextActionInner::ReadNodeAsValue(root, ownership);

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
            NextActionInner::ReadNodeAsValue(node, ownership) => {
                self.nodes.get(node).handle_as_value(Context {
                    request: ownership,
                    interpreter,
                    stack: &mut self.stack,
                })?
            }
            NextActionInner::ReadNodeAsAssignmentTarget(node, value) => {
                self.nodes.get(node).handle_as_assignment_target(
                    Context {
                        stack: &mut self.stack,
                        interpreter,
                        request: (),
                    },
                    self.nodes,
                    node,
                    value,
                )?
            }
            NextActionInner::HandleReturnedItem(item) => {
                let top_of_stack = match self.stack.handlers.pop() {
                    Some(top) => top,
                    None => {
                        return Ok(StepResult::Return(item));
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
    Return(EvaluationItem),
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
            ResolvedValue::Assignee(assignee) => Self::return_assignee(assignee),
            ResolvedValue::Shared(shared) => Self::return_shared(shared),
            ResolvedValue::CopyOnWrite(copy_on_write) => Self::return_copy_on_write(copy_on_write),
        }
    }

    pub(super) fn return_late_bound(late_bound: LateBoundValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::LateBound(late_bound)).into()
    }

    pub(super) fn return_assignee(assignee: AssigneeValue) -> Self {
        NextActionInner::HandleReturnedItem(EvaluationItem::Assignee(assignee)).into()
    }

    fn return_item(item: EvaluationItem) -> Self {
        NextActionInner::HandleReturnedItem(item).into()
    }
}

enum NextActionInner {
    /// Enters an expression node to output a value
    ReadNodeAsValue(ExpressionNodeId, RequestedValueOwnership),
    // Enters an expression node for assignment purposes
    // This covers atomic assignments and composite assignments
    // (similar to patterns but for existing values/reassignments)
    // let a = ["x", "y"]; let b; [a[1], .. b] = [1, 2, 3, 4]
    ReadNodeAsAssignmentTarget(ExpressionNodeId, ExpressionValue),
    HandleReturnedItem(EvaluationItem),
}

impl From<NextActionInner> for NextAction {
    fn from(value: NextActionInner) -> Self {
        Self(value)
    }
}

pub(crate) enum EvaluationItem {
    // Value items - these mirror RequestedValueOwnership exactly
    LateBound(LateBoundValue),
    Owned(OwnedValue),
    Shared(SharedValue),
    Mutable(MutableValue), // Mutable reference to a value
    CopyOnWrite(CopyOnWriteValue),
    /// Note that assignees are handled subtly differently than a mutable value,
    /// for example with an assignee, x["a"] creates an entry if it doesn't exist,
    /// whereas with a mutable value it would return None without creating the entry.
    Assignee(AssigneeValue),

    // Assignment items
    AssignmentCompletion(AssignmentCompletion),
}

impl EvaluationItem {
    pub(crate) fn expect_owned(self) -> OwnedValue {
        match self {
            EvaluationItem::Owned(value) => value,
            _ => panic!("expect_owned() called on non-owned EvaluationItem"),
        }
    }

    pub(crate) fn expect_shared(self) -> SharedValue {
        match self {
            EvaluationItem::Shared(shared) => shared,
            _ => panic!("expect_shared() called on non-shared EvaluationItem"),
        }
    }

    pub(crate) fn expect_mutable(self) -> MutableValue {
        match self {
            EvaluationItem::Mutable(mutable) => mutable,
            _ => panic!("expect_mutable() called on non-mutable EvaluationItem"),
        }
    }

    pub(super) fn expect_assignee(self) -> AssigneeValue {
        match self {
            EvaluationItem::Assignee(assignee) => assignee,
            _ => panic!("expect_assignee() called on non-assignee EvaluationItem"),
        }
    }

    pub(crate) fn expect_late_bound(self) -> LateBoundValue {
        match self {
            EvaluationItem::LateBound(late_bound) => late_bound,
            _ => panic!("expect_late_bound() called on non-late-bound EvaluationItem"),
        }
    }

    pub(crate) fn expect_copy_on_write(self) -> CopyOnWriteValue {
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

    pub(crate) fn expect_resolved_value(self) -> ResolvedValue {
        match self {
            EvaluationItem::Owned(value) => ResolvedValue::Owned(value),
            EvaluationItem::Mutable(mutable) => ResolvedValue::Mutable(mutable),
            EvaluationItem::Assignee(assignee) => ResolvedValue::Assignee(assignee),
            EvaluationItem::Shared(shared) => ResolvedValue::Shared(shared),
            EvaluationItem::CopyOnWrite(copy_on_write) => ResolvedValue::CopyOnWrite(copy_on_write),
            EvaluationItem::LateBound(_) | EvaluationItem::AssignmentCompletion(_) => {
                panic!("expect_resolved_value() called on non-value EvaluationItem")
            }
        }
    }

    pub(crate) fn expect_any_value_and_map(
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
            EvaluationItem::Assignee(assignee) => {
                EvaluationItem::Assignee(Assignee(map_mutable(assignee.0)?))
            }
            EvaluationItem::Mutable(mutable) => EvaluationItem::Mutable(map_mutable(mutable)?),
            EvaluationItem::Shared(shared) => EvaluationItem::Shared(map_shared(shared)?),
            EvaluationItem::CopyOnWrite(cow) => {
                EvaluationItem::CopyOnWrite(cow.map(map_shared, map_owned)?)
            }
            EvaluationItem::AssignmentCompletion(_) => {
                panic!("expect_any_value_and_map() called on non-value EvaluationItem")
            }
        })
    }
}

impl WithSpanRangeExt for EvaluationItem {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        match self {
            EvaluationItem::LateBound(late_bound) => {
                EvaluationItem::LateBound(late_bound.with_span_range(span_range))
            }
            EvaluationItem::Owned(value) => {
                EvaluationItem::Owned(value.with_span_range(span_range))
            }
            EvaluationItem::Mutable(mutable) => {
                EvaluationItem::Mutable(mutable.with_span_range(span_range))
            }
            EvaluationItem::Shared(shared) => {
                EvaluationItem::Shared(shared.with_span_range(span_range))
            }
            EvaluationItem::CopyOnWrite(cow) => {
                EvaluationItem::CopyOnWrite(cow.with_span_range(span_range))
            }
            EvaluationItem::Assignee(assignee) => {
                EvaluationItem::Assignee(assignee.with_span_range(span_range))
            }
            EvaluationItem::AssignmentCompletion(assignment_completion) => {
                EvaluationItem::AssignmentCompletion(
                    assignment_completion.with_span_range(span_range),
                )
            }
        }
    }
}

/// See the [rust reference] for a good description of assignee vs place.
///
/// [rust reference]: https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
pub(super) enum AnyEvaluationHandler {
    Value(AnyValueFrame, RequestedValueOwnership),
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

    pub(super) fn handle_node_as_assignee<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        auto_create: bool,
    ) -> NextAction {
        self.handle_node_as_any_value(
            handler,
            node,
            RequestedValueOwnership::Concrete(ResolvedValueOwnership::Assignee { auto_create }),
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

    pub(super) fn handle_node_as_assignment<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        value: ExpressionValue,
    ) -> NextAction {
        self.stack
            .handlers
            .push(T::into_unkinded_handler(handler.into_any(), self.request));
        NextActionInner::ReadNodeAsAssignmentTarget(node, value).into()
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
        Ok(NextAction::return_item(
            self.request.map_from_late_bound(late_bound)?,
        ))
    }

    pub(super) fn return_resolved_value(self, value: ResolvedValue) -> ExecutionResult<NextAction> {
        Ok(NextAction::return_item(
            self.request.map_from_resolved(value)?,
        ))
    }

    pub(super) fn return_item(self, value: EvaluationItem) -> ExecutionResult<NextAction> {
        Ok(NextAction::return_item(self.request.map_from_item(value)?))
    }

    pub(super) fn return_owned(self, value: OwnedValue) -> ExecutionResult<NextAction> {
        Ok(NextAction::return_item(self.request.map_from_owned(value)?))
    }

    pub(super) fn return_copy_on_write(self, cow: CopyOnWriteValue) -> ExecutionResult<NextAction> {
        Ok(NextAction::return_item(
            self.request.map_from_copy_on_write(cow)?,
        ))
    }

    pub(super) fn return_mutable(self, mutable: MutableValue) -> ExecutionResult<NextAction> {
        Ok(NextAction::return_item(
            self.request.map_from_mutable(mutable)?,
        ))
    }

    pub(super) fn return_shared(self, shared: SharedValue) -> ExecutionResult<NextAction> {
        Ok(NextAction::return_item(
            self.request.map_from_shared(shared)?,
        ))
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
