use super::*;

/// Trait for types that can be evaluated to produce a value with span information.
///
/// Types implementing this trait provide `evaluate_unspanned()` which returns the raw value,
/// and get a default `evaluate()` implementation that wraps the result with the type's span.
pub(crate) trait Evaluate: HasSpanRange {
    fn evaluate_unspanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue>;

    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        let value = self.evaluate_unspanned(interpreter, ownership)?;
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
    ) -> ExecutionResult<SpannedRequestedValue> {
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
                let expression_node = self.nodes.get(node);
                let output_span_range = expression_node.span_range();
                expression_node.handle_as_value(Context {
                    request: ownership,
                    interpreter,
                    stack: &mut self.stack,
                    output_span_range,
                })?
            }
            NextActionInner::ReadNodeAsAssignmentTarget(node, value) => {
                let expression_node = self.nodes.get(node);
                let output_span_range = expression_node.span_range();
                expression_node.handle_as_assignment_target(
                    Context {
                        stack: &mut self.stack,
                        interpreter,
                        request: (),
                        output_span_range,
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

/// A `RequestedValue` with its associated span.
pub(crate) type SpannedRequestedValue = Spanned<RequestedValue>;

pub(super) enum StepResult {
    Continue(NextAction),
    Return(SpannedRequestedValue),
}

pub(super) struct NextAction(NextActionInner);

impl NextAction {
    fn return_requested(value: SpannedRequestedValue) -> Self {
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
    ReadNodeAsAssignmentTarget(ExpressionNodeId, Value),
    HandleReturnedValue(SpannedRequestedValue),
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
    Owned(OwnedValue),
    Shared(SharedValue),
    Mutable(MutableValue),
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
    pub(crate) fn expect_owned(self) -> OwnedValue {
        match self {
            RequestedValue::Owned(value) => value,
            _ => panic!("expect_owned() called on non-owned RequestedValue"),
        }
    }

    pub(crate) fn expect_shared(self) -> SharedValue {
        match self {
            RequestedValue::Shared(shared) => shared,
            _ => panic!("expect_shared() called on non-shared RequestedValue"),
        }
    }

    pub(super) fn expect_assignee(self) -> AssigneeValue {
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

    pub(crate) fn expect_any_value_and_map(
        self,
        map_shared: impl FnOnce(SharedValue) -> ExecutionResult<SharedValue>,
        map_mutable: impl FnOnce(MutableValue) -> ExecutionResult<MutableValue>,
        map_owned: impl FnOnce(OwnedValue) -> ExecutionResult<OwnedValue>,
    ) -> ExecutionResult<RequestedValue> {
        Ok(match self {
            RequestedValue::LateBound(late_bound) => {
                RequestedValue::LateBound(late_bound.map_any(map_shared, map_mutable, map_owned)?)
            }
            RequestedValue::Owned(value) => RequestedValue::Owned(map_owned(value)?),
            RequestedValue::Assignee(assignee) => {
                // Assignee is Assignee(Mutable<Value>)
                // Extract the inner Mutable, map it, then rewrap in Assignee
                let mapped = map_mutable(assignee.0)?;
                RequestedValue::Assignee(Assignee(mapped))
            }
            RequestedValue::Mutable(mutable) => RequestedValue::Mutable(map_mutable(mutable)?),
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

// Note: RequestedValue no longer implements WithSpanRangeExt since the inner
// value types (Owned, Mutable, Shared, etc.) no longer carry spans internally.
// Spans should be tracked separately at a higher level if needed.

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
        spanned_value: SpannedRequestedValue,
    ) -> ExecutionResult<NextAction> {
        let Spanned(value, output_span_range) = spanned_value;
        match self {
            AnyEvaluationHandler::Value(handler, ownership) => handler.handle_next(
                Context {
                    interpreter,
                    stack,
                    request: ownership,
                    output_span_range,
                },
                value,
            ),
            AnyEvaluationHandler::Assignment(handler) => handler.handle_next(
                Context {
                    interpreter,
                    stack,
                    request: (),
                    output_span_range,
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
    pub(super) output_span_range: SpanRange,
}

impl<'a, T: RequestedValueType> Context<'a, T> {
    /// Updates the output span range. Call this before returning if you need to
    /// override the span with a different one (e.g., for grouped expressions where
    /// the span should cover the entire grouping, not just the inner expression).
    pub(super) fn set_output_span(&mut self, span_range: SpanRange) {
        self.output_span_range = span_range;
    }

    pub(super) fn request_owned<H: EvaluationFrame<ReturnType = T>>(
        self,
        handler: H,
        node: ExpressionNodeId,
    ) -> NextAction {
        self.request_argument_value(handler, node, ArgumentOwnership::Owned)
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
        value: Value,
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
        value: RequestedValue,
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

pub(super) struct ValueType;
pub(super) type ValueContext<'a> = Context<'a, ValueType>;

impl RequestedValueType for ValueType {
    type RequestConstraints = RequestedOwnership;
    type AnyHandler = AnyValueFrame;

    fn into_unkinded_handler(
        handler: Self::AnyHandler,
        request: Self::RequestConstraints,
    ) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Value(handler, request)
    }
}

impl<'a> Context<'a, ValueType> {
    pub(super) fn requested_ownership(&self) -> RequestedOwnership {
        self.request
    }

    pub(super) fn evaluate(
        self,
        f: impl FnOnce(&mut Interpreter, RequestedOwnership) -> ExecutionResult<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let value = f(self.interpreter, self.request)?;
        self.return_not_necessarily_matching_requested(value)
    }

    pub(super) fn return_late_bound(
        self,
        late_bound: LateBoundValue,
    ) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_late_bound(late_bound)?;
        Ok(NextAction::return_requested(Spanned(
            value,
            self.output_span_range,
        )))
    }

    pub(super) fn return_argument_value(self, value: ArgumentValue) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_argument(value)?;
        Ok(NextAction::return_requested(Spanned(
            value,
            self.output_span_range,
        )))
    }

    pub(super) fn return_returned_value(self, value: ReturnedValue) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_returned(value)?;
        Ok(NextAction::return_requested(Spanned(
            value,
            self.output_span_range,
        )))
    }

    /// Note: This doesn't assume that the requested ownership matches the value's ownership.
    ///
    /// This allows the value to come from `requested.replace_owned_with_copy_on_write()`,
    /// and this resolver then maps back to the requested ownership.
    /// See e.g. [`ValuePropertyAccessBuilder`].
    pub(super) fn return_not_necessarily_matching_requested(
        self,
        value: RequestedValue,
    ) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_requested(value)?;
        Ok(NextAction::return_requested(Spanned(
            value,
            self.output_span_range,
        )))
    }

    pub(super) fn return_value(self, value: impl IsReturnable) -> ExecutionResult<NextAction> {
        let value = self.request.map_from_returned(value.to_returned_value()?)?;
        Ok(NextAction::return_requested(Spanned(
            value,
            self.output_span_range,
        )))
    }
}

pub(super) struct AssignmentType;

pub(super) type AssignmentContext<'a> = Context<'a, AssignmentType>;

impl RequestedValueType for AssignmentType {
    type RequestConstraints = ();
    type AnyHandler = AnyAssignmentFrame;

    fn into_unkinded_handler(handler: Self::AnyHandler, _request: ()) -> AnyEvaluationHandler {
        AnyEvaluationHandler::Assignment(handler)
    }
}

impl<'a> Context<'a, AssignmentType> {
    pub(super) fn return_assignment_completion(self, span_range: SpanRange) -> NextAction {
        NextActionInner::HandleReturnedValue(Spanned(
            RequestedValue::AssignmentCompletion(AssignmentCompletion { span_range }),
            span_range,
        ))
        .into()
    }
}
