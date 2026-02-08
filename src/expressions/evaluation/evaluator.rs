use super::*;

/// Trait for types that can be evaluated to produce a value with span information.
///
/// Types implementing this trait provide `evaluate()` which returns the raw value,
/// and get a default `evaluate_spanned()` implementation that wraps the result with the type's span.
pub(crate) trait Evaluate: HasSpanRange {
    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue>;

    fn evaluate_spanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        let value = self.evaluate(interpreter, ownership)?;
        Ok(Spanned(value, self.span_range()))
    }
}

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
        ownership: RequestedOwnership,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
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
            NextActionInner::HandleReturnedValue(item) => {
                let top_of_stack = match self.stack.handlers.pop() {
                    Some(top) => top,
                    None => {
                        return Ok(StepResult::Return(item));
                    }
                };
                top_of_stack.handle_next(interpreter, &mut self.stack, item)?
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
    Return(Spanned<RequestedValue>),
}

pub(super) struct NextAction(NextActionInner);

impl NextAction {
    fn return_requested(value: Spanned<RequestedValue>) -> Self {
        NextActionInner::HandleReturnedValue(value).into()
    }
}

enum NextActionInner {
    /// Enters an expression node to output a value
    ReadNodeAsValue(ExpressionNodeId, RequestedOwnership),
    // Enters an expression node for assignment purposes
    // This covers atomic assignments and composite assignments
    // (similar to patterns but for existing values/reassignments)
    // let a = ["x", "y"]; let b; [a[1], .. b] = [1, 2, 3, 4]
    ReadNodeAsAssignmentTarget(ExpressionNodeId, AnyValue),
    HandleReturnedValue(Spanned<RequestedValue>),
}

impl From<NextActionInner> for NextAction {
    fn from(value: NextActionInner) -> Self {
        Self(value)
    }
}

/// This value is the result of evaluating an expression, with the requested ownership applied
/// to the result. It should always be paired with a corresponding [`RequestedOwnership`] which
/// is used to produce it.
///
/// See [`RequestedOwnership`] and [`ArgumentOwnership`] for more details on these different types.
///
/// See also [`ReturnedValue`] which is used for returned values which don't necessarily yet
/// align with the requested ownership.
pub(crate) enum RequestedValue {
    // RequestedOwnership::Concrete(_)
    // -------------------------------
    Owned(AnyValueOwned),
    Shared(AnyValueShared),
    Mutable(AnyValueMutable),
    CopyOnWrite(CopyOnWriteValue),
    Assignee(AssigneeValue),

    // RequestedOwnership::LateBound
    // -------------------------------
    LateBound(LateBoundValue),

    // Marks completion of an assignment frame
    // ---------------------------------------
    AssignmentCompletion(AssignmentCompletion),
}

impl RequestedValue {
    pub(crate) fn expect_owned(self) -> AnyValueOwned {
        match self {
            RequestedValue::Owned(value) => value,
            _ => panic!("expect_owned() called on non-owned RequestedValue"),
        }
    }

    pub(crate) fn expect_shared(self) -> AnyValueShared {
        match self {
            RequestedValue::Shared(shared) => shared,
            _ => panic!("expect_shared() called on non-shared RequestedValue"),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn expect_copy_on_write(self) -> CopyOnWriteValue {
        match self {
            RequestedValue::CopyOnWrite(copy_on_write) => copy_on_write,
            _ => panic!("expect_copy_on_write() called on non-copy-on-write RequestedValue"),
        }
    }

    pub(super) fn expect_assignee(self) -> AnyValueAssignee {
        match self {
            RequestedValue::Assignee(assignee) => assignee,
            _ => panic!("expect_assignee() called on non-assignee RequestedValue"),
        }
    }

    pub(crate) fn expect_late_bound(self) -> LateBoundValue {
        match self {
            RequestedValue::LateBound(late_bound) => late_bound,
            _ => panic!("expect_late_bound() called on non-late-bound RequestedValue"),
        }
    }

    pub(super) fn expect_assignment_completion(self) -> AssignmentCompletion {
        match self {
            RequestedValue::AssignmentCompletion(completion) => completion,
            _ => panic!(
                "expect_assignment_completion() called on non-assignment-completion RequestedValue"
            ),
        }
    }

    pub(crate) fn expect_argument_value(self) -> ArgumentValue {
        match self {
            RequestedValue::Owned(value) => ArgumentValue::Owned(value),
            RequestedValue::Mutable(mutable) => ArgumentValue::Mutable(mutable),
            RequestedValue::Assignee(assignee) => ArgumentValue::Assignee(assignee),
            RequestedValue::Shared(shared) => ArgumentValue::Shared(shared),
            RequestedValue::CopyOnWrite(copy_on_write) => ArgumentValue::CopyOnWrite(copy_on_write),
            RequestedValue::LateBound(_) | RequestedValue::AssignmentCompletion(_) => {
                panic!("expect_resolved_value() called on non-value RequestedValue")
            }
        }
    }

    /// Returns the leaf kind of the underlying value, for type resolution purposes.
    pub(crate) fn value_kind(&self) -> AnyValueLeafKind {
        match self {
            RequestedValue::Owned(value) => value.value_kind(),
            RequestedValue::Shared(shared) => shared.value_kind(),
            RequestedValue::Mutable(mutable) => mutable.value_kind(),
            RequestedValue::CopyOnWrite(cow) => cow.value_kind(),
            RequestedValue::Assignee(assignee) => assignee.value_kind(),
            RequestedValue::LateBound(late_bound) => late_bound.value_kind(),
            RequestedValue::AssignmentCompletion(_) => {
                panic!("value_kind() called on AssignmentCompletion")
            }
        }
    }

    pub(crate) fn expect_any_value_and_map(
        self,
        map_shared: impl FnOnce(AnyValueShared) -> FunctionResult<AnyValueShared>,
        map_mutable: impl FnOnce(
            AnyValueMutable,
        )
            -> Result<AnyValueMutable, (FunctionError, AnyValueMutable)>,
        map_owned: impl FnOnce(AnyValueOwned) -> FunctionResult<AnyValueOwned>,
    ) -> FunctionResult<RequestedValue> {
        Ok(match self {
            RequestedValue::LateBound(late_bound) => {
                RequestedValue::LateBound(late_bound.map_any(map_shared, map_mutable, map_owned)?)
            }
            RequestedValue::Owned(value) => RequestedValue::Owned(map_owned(value)?),
            RequestedValue::Assignee(assignee) => {
                // Assignee doesn't support fallback - propagate error directly
                let mapped = map_mutable(assignee.0).map_err(|(e, _)| e)?;
                RequestedValue::Assignee(Assignee(mapped))
            }
            RequestedValue::Mutable(mutable) => {
                // Non-late-bound mutable doesn't support fallback - propagate error directly
                let mapped = map_mutable(mutable).map_err(|(e, _)| e)?;
                RequestedValue::Mutable(mapped)
            }
            RequestedValue::Shared(shared) => RequestedValue::Shared(map_shared(shared)?),
            RequestedValue::CopyOnWrite(cow) => {
                RequestedValue::CopyOnWrite(cow.map(map_shared, map_owned)?)
            }
            RequestedValue::AssignmentCompletion(_) => {
                panic!("expect_any_value_and_map() called on non-value RequestedValue")
            }
        })
    }
}

#[allow(unused)]
impl Spanned<RequestedValue> {
    #[inline]
    pub(crate) fn expect_owned(self) -> Spanned<AnyValueOwned> {
        self.map(|v| v.expect_owned())
    }

    #[inline]
    pub(crate) fn expect_shared(self) -> Spanned<AnyValueShared> {
        self.map(|v| v.expect_shared())
    }

    #[inline]
    pub(crate) fn expect_assignee(self) -> Spanned<AnyValueAssignee> {
        self.map(|v| v.expect_assignee())
    }

    #[inline]
    pub(crate) fn expect_late_bound(self) -> Spanned<LateBoundValue> {
        self.map(|v| v.expect_late_bound())
    }

    #[inline]
    pub(crate) fn expect_argument_value(self) -> Spanned<ArgumentValue> {
        self.map(|v| v.expect_argument_value())
    }

    #[inline]
    pub(crate) fn expect_assignment_completion(self) -> Spanned<AssignmentCompletion> {
        self.map(|v| v.expect_assignment_completion())
    }
}

/// See the [rust reference] for a good description of assignee vs place.
///
/// [rust reference]: https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
pub(super) enum AnyEvaluationHandler {
    Value(AnyValueFrame, RequestedOwnership),
    Assignment(AnyAssignmentFrame),
}

impl AnyEvaluationHandler {
    fn handle_next(
        self,
        interpreter: &mut Interpreter,
        stack: &mut EvaluationStack,
        value: Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        match self {
            AnyEvaluationHandler::Value(handler, ownership) => handler.handle_next(
                Context {
                    interpreter,
                    stack,
                    request: ownership,
                },
                value,
            ),
            AnyEvaluationHandler::Assignment(handler) => handler.handle_next(
                Context {
                    interpreter,
                    stack,
                    request: (),
                },
                value,
            ),
        }
    }
}

pub(super) struct Context<'a, T: RequestedValueType> {
    interpreter: &'a mut Interpreter,
    stack: &'a mut EvaluationStack,
    request: T::RequestConstraints,
}

impl<'a, T: RequestedValueType> Context<'a, T> {
    pub(super) fn request_owned<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.request_argument_value(handler, node, ArgumentOwnership::Owned)
    }

    #[allow(dead_code)]
    pub(super) fn request_copy_on_write<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.request_argument_value(handler, node, ArgumentOwnership::CopyOnWrite)
    }

    pub(super) fn request_shared<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.request_argument_value(handler, node, ArgumentOwnership::Shared)
    }

    pub(super) fn request_assignee<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        auto_create: bool,
    ) -> NextAction {
        self.request_argument_value(handler, node, ArgumentOwnership::Assignee { auto_create })
    }

    pub(super) fn request_late_bound<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.request_any_value(handler, node, RequestedOwnership::LateBound)
    }

    pub(super) fn request_any_value<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        requested_ownership: RequestedOwnership,
    ) -> NextAction {
        self.stack
            .handlers
            .push(T::into_unkinded_handler(handler.into_any(), self.request));
        NextActionInner::ReadNodeAsValue(node, requested_ownership).into()
    }

    pub(super) fn request_argument_value<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        argument_ownership: ArgumentOwnership,
    ) -> NextAction {
        self.request_any_value(handler, node, argument_ownership.into())
    }

    pub(super) fn request_assignment<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
        value: AnyValue,
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
    type ReturnType: RequestedValueType;

    fn into_any(self) -> <Self::ReturnType as RequestedValueType>::AnyHandler;

    fn handle_next(
        self,
        context: Context<Self::ReturnType>,
        value: Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction>;
}

pub(super) trait RequestedValueType {
    type RequestConstraints;
    type AnyHandler;
    fn into_unkinded_handler(
        handler: Self::AnyHandler,
        request: Self::RequestConstraints,
    ) -> AnyEvaluationHandler;
}

pub(super) struct ReturnsValue;
pub(super) type ValueContext<'a> = Context<'a, ReturnsValue>;

impl RequestedValueType for ReturnsValue {
    type RequestConstraints = RequestedOwnership;
    type AnyHandler = AnyValueFrame;

    fn into_unkinded_handler(
        handler: Self::AnyHandler,
        request: Self::RequestConstraints,
    ) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Value(handler, request)
    }
}

impl<'a> Context<'a, ReturnsValue> {
    pub(super) fn requested_ownership(&self) -> RequestedOwnership {
        self.request
    }

    pub(super) fn evaluate(
        self,
        f: impl FnOnce(&mut Interpreter, RequestedOwnership) -> ExecutionResult<Spanned<RequestedValue>>,
    ) -> ExecutionResult<NextAction> {
        let value = f(self.interpreter, self.request)?;
        self.return_not_necessarily_matching_requested(value)
    }

    pub(super) fn return_late_bound(
        self,
        late_bound: Spanned<LateBoundValue>,
    ) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_late_bound(late_bound)?;
        Ok(NextAction::return_requested(value))
    }

    pub(super) fn return_argument_value(
        self,
        value: Spanned<ArgumentValue>,
    ) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_argument(value)?;
        Ok(NextAction::return_requested(value))
    }

    pub(super) fn return_returned_value(
        self,
        value: Spanned<ReturnedValue>,
    ) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_returned(value)?;
        Ok(NextAction::return_requested(value))
    }

    /// Note: This doesn't assume that the requested ownership matches the value's ownership.
    ///
    /// This allows the value to come from `requested.replace_owned_with_copy_on_write()`,
    /// and this resolver then maps back to the requested ownership.
    /// See e.g. [`ValuePropertyAccessBuilder`].
    pub(super) fn return_not_necessarily_matching_requested(
        self,
        value: Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_requested(value)?;
        Ok(NextAction::return_requested(value))
    }

    pub(super) fn return_value<T: IsReturnable>(
        self,
        value: Spanned<T>,
    ) -> ExecutionResult<NextAction> {
        let value = self
            .request
            .map_from_returned(value.try_map(|v| v.to_returned_value())?)?;
        Ok(NextAction::return_requested(value))
    }
}

pub(super) struct ReturnsAssignmentCompletion;

pub(super) type AssignmentContext<'a> = Context<'a, ReturnsAssignmentCompletion>;

impl RequestedValueType for ReturnsAssignmentCompletion {
    type RequestConstraints = ();
    type AnyHandler = AnyAssignmentFrame;

    fn into_unkinded_handler(handler: Self::AnyHandler, _request: ()) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Assignment(handler)
    }
}

impl<'a> Context<'a, ReturnsAssignmentCompletion> {
    pub(super) fn return_assignment_completion(self, span_range: SpanRange) -> NextAction {
        NextActionInner::HandleReturnedValue(Spanned(
            RequestedValue::AssignmentCompletion(AssignmentCompletion),
            span_range,
        ))
        .into()
    }
}
