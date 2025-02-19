use super::*;

pub(super) use inner::ExpressionEvaluator;

/// This is to hide implementation details to protect the abstraction and make it harder to make mistakes.
mod inner {
    use super::*;

    pub(in super::super) struct ExpressionEvaluator<'a, K: Expressionable> {
        nodes: &'a [ExpressionNode<K>],
        stacks: Stacks,
    }

    impl<'a> ExpressionEvaluator<'a, Source> {
        pub(in super::super) fn new(nodes: &'a [ExpressionNode<Source>]) -> Self {
            Self {
                nodes,
                stacks: Stacks::new(),
            }
        }

        pub(in super::super) fn evaluate(
            mut self,
            root: ExpressionNodeId,
            interpreter: &mut Interpreter,
        ) -> ExecutionResult<ExpressionValue> {
            let mut next_action = NextActionInner::ReadNodeAsValue(root);

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
                NextActionInner::ReadNodeAsValue(node) => self.nodes[node.0]
                    .handle_as_value(interpreter, self.stacks.creator(ReturnMode::Value))?,
                NextActionInner::HandleReturnedValue(value) => {
                    let top_of_stack = match self.stacks.value_stack.pop() {
                        Some(top) => top,
                        None => {
                            debug_assert!(self.stacks.assignment_stack.is_empty(), "Evaluation completed with none-empty assignment stack - there's some bug in the ExpressionEvaluator");
                            debug_assert!(self.stacks.place_stack.is_empty(), "Evaluation completed with none-empty place stack - there's some bug in the ExpressionEvaluator");
                            return Ok(StepResult::Return(value));
                        }
                    };
                    let next_creator = self.stacks.creator(top_of_stack.ultimate_return_mode());
                    top_of_stack.handle_value(value, next_creator)?
                }
                NextActionInner::ReadNodeAsAssignee(node, value) => self.nodes[node.0]
                    .handle_as_assignee(
                        self.nodes,
                        node,
                        value,
                        interpreter,
                        self.stacks.creator(ReturnMode::AssignmentCompletion),
                    )?,
                NextActionInner::HandleAssignmentComplete(assignment_complete) => {
                    let top_of_stack = match self.stacks.assignment_stack.pop() {
                        Some(top) => top,
                        None => unreachable!("Received AssignmentComplete without any assignment stack frames - there's some bug in the ExpressionEvaluator"),
                    };
                    let next_creator = self.stacks.creator(top_of_stack.ultimate_return_mode());
                    top_of_stack.handle_assignment_complete(assignment_complete, next_creator)?
                }
                NextActionInner::ReadNodeAsPlace(node) => self.nodes[node.0]
                    .handle_as_place(interpreter, self.stacks.creator(ReturnMode::Place))?,
                NextActionInner::HandleReturnedPlace(place) => {
                    let top_of_stack = match self.stacks.place_stack.pop() {
                        Some(top) => top,
                        None => unreachable!("Received Place without any place stack frames - there's some bug in the ExpressionEvaluator"),
                    };
                    let next_creator = self.stacks.creator(top_of_stack.ultimate_return_mode());
                    top_of_stack.handle_place(place, next_creator)?
                }
            }))
        }
    }

    /// See the [rust reference] for a good description of assignee vs place.
    ///
    /// [rust reference]: https://doc.rust-lang.org/reference/expressions/assignment-expressions.html#assignee-vs-place
    pub(super) struct Stacks {
        /// The stack of operations which will output a value
        value_stack: Vec<ValueStackFrame>,
        /// The stack of operations which will handle an assignment completion
        assignment_stack: Vec<AssignmentStackFrame>,
        /// The stack of operations which will output a place
        place_stack: Vec<PlaceStackFrame>,
    }

    impl Stacks {
        pub(super) fn new() -> Self {
            Self {
                value_stack: Vec::new(),
                assignment_stack: Vec::new(),
                place_stack: Vec::new(),
            }
        }

        fn creator(&mut self, return_mode: ReturnMode) -> ActionCreator {
            ActionCreator {
                stacks: self,
                return_mode,
            }
        }
    }

    pub(super) enum StepResult {
        Continue(NextAction),
        Return(ExpressionValue),
    }

    pub(super) struct NextAction(NextActionInner);

    enum NextActionInner {
        /// Enters an expression node to output a value
        ReadNodeAsValue(ExpressionNodeId),
        HandleReturnedValue(ExpressionValue),
        // Enters an expression node for assignment purposes
        ReadNodeAsAssignee(ExpressionNodeId, ExpressionValue),
        HandleAssignmentComplete(AssignmentCompletion),
        // Enters an expression node to output a place
        ReadNodeAsPlace(ExpressionNodeId),
        HandleReturnedPlace(Place),
    }

    impl From<NextActionInner> for NextAction {
        fn from(value: NextActionInner) -> Self {
            Self(value)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) enum ReturnMode {
        Value,
        AssignmentCompletion,
        Place,
    }

    pub(super) struct ActionCreator<'a> {
        stacks: &'a mut Stacks,
        return_mode: ReturnMode,
    }

    impl ActionCreator<'_> {
        pub(super) fn return_value(self, value: ExpressionValue) -> NextAction {
            debug_assert_eq!(
                self.return_mode,
                ReturnMode::Value,
                "This handler claimed to ultimately return {:?}, but it returned a value",
                self.return_mode
            );
            NextActionInner::HandleReturnedValue(value).into()
        }

        pub(super) fn read_value_with_handler(
            self,
            node: ExpressionNodeId,
            handler: ValueStackFrame,
        ) -> NextAction {
            debug_assert_eq!(
                handler.ultimate_return_mode(),
                self.return_mode,
                "Handler is expected to return {:?}, but claims to ultimately return {:?}",
                self.return_mode,
                handler.ultimate_return_mode()
            );
            self.stacks.value_stack.push(handler);
            NextActionInner::ReadNodeAsValue(node).into()
        }

        pub(super) fn return_place(self, place: Place) -> NextAction {
            debug_assert_eq!(
                self.return_mode,
                ReturnMode::Place,
                "This handler claimed to ultimately return {:?}, but it returned a place",
                self.return_mode
            );
            NextActionInner::HandleReturnedPlace(place).into()
        }

        pub(super) fn read_place_with_handler(
            self,
            node: ExpressionNodeId,
            handler: PlaceStackFrame,
        ) -> NextAction {
            debug_assert_eq!(
                handler.ultimate_return_mode(),
                self.return_mode,
                "Handler is expected to return {:?}, but claims to ultimately return {:?}",
                self.return_mode,
                handler.ultimate_return_mode()
            );
            self.stacks.place_stack.push(handler);
            NextActionInner::ReadNodeAsPlace(node).into()
        }

        /// The span range should cover from the start of the assignee to the end of the value
        pub(super) fn return_assignment_completion(self, span_range: SpanRange) -> NextAction {
            debug_assert_eq!(self.return_mode, ReturnMode::AssignmentCompletion, "This handler claimed to ultimately return {:?}, but it returned an assignment completion", self.return_mode);
            NextActionInner::HandleAssignmentComplete(AssignmentCompletion { span_range }).into()
        }

        pub(super) fn handle_assignment_and_return_to(
            self,
            node: ExpressionNodeId,
            value: ExpressionValue,
            handler: AssignmentStackFrame,
        ) -> NextAction {
            debug_assert_eq!(
                handler.ultimate_return_mode(),
                self.return_mode,
                "Handler is expected to return {:?}, but claims to ultimately return {:?}",
                self.return_mode,
                handler.ultimate_return_mode()
            );
            self.stacks.assignment_stack.push(handler);
            NextActionInner::ReadNodeAsAssignee(node, value).into()
        }
    }
}

use inner::*;

impl ExpressionNode<Source> {
    fn handle_as_value(
        &self,
        interpreter: &mut Interpreter,
        next: ActionCreator,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(leaf) => {
                next.return_value(Source::evaluate_leaf(leaf, interpreter)?)
            }
            ExpressionNode::Grouped { delim_span, inner } => next.read_value_with_handler(
                *inner,
                ValueStackFrame::Group {
                    span: delim_span.join(),
                },
            ),
            ExpressionNode::Array { brackets, items } => ArrayValueStackFrame {
                span: brackets.join(),
                unevaluated_items: items.clone(),
                evaluated_items: Vec::with_capacity(items.len()),
            }
            .next(next),
            ExpressionNode::Object { braces, entries } => ObjectValueStackFrame {
                span: braces.join(),
                unevaluated_entries: entries.clone(),
                evaluated_entries: BTreeMap::new(),
                pending: None,
            }
            .next(next)?,
            ExpressionNode::UnaryOperation { operation, input } => next.read_value_with_handler(
                *input,
                ValueStackFrame::UnaryOperation {
                    operation: operation.clone(),
                },
            ),
            ExpressionNode::BinaryOperation {
                operation,
                left_input,
                right_input,
            } => next.read_value_with_handler(
                *left_input,
                ValueStackFrame::BinaryOperation {
                    operation: operation.clone(),
                    state: BinaryPath::OnLeftBranch {
                        right: *right_input,
                    },
                },
            ),
            ExpressionNode::Property { node, access } => next.read_value_with_handler(
                *node,
                ValueStackFrame::Property {
                    access: access.clone(),
                },
            ),
            ExpressionNode::Index {
                node,
                access,
                index,
            } => next.read_value_with_handler(
                *node,
                ValueStackFrame::Index {
                    access: *access,
                    state: IndexPath::OnObjectBranch { index: *index },
                },
            ),
            ExpressionNode::Range {
                left,
                range_limits,
                right,
            } => {
                match (left, right) {
                    (None, None) => match range_limits {
                        syn::RangeLimits::HalfOpen(token) => {
                            let inner = ExpressionRangeInner::RangeFull { token: *token };
                            next.return_value(inner.to_value(token.span_range()))
                        }
                        syn::RangeLimits::Closed(_) => {
                            unreachable!("A closed range should have been given a right in continue_range(..)")
                        }
                    },
                    (None, Some(right)) => next.read_value_with_handler(
                        *right,
                        ValueStackFrame::Range {
                            range_limits: *range_limits,
                            state: RangePath::OnRightBranch { left: None },
                        },
                    ),
                    (Some(left), right) => next.read_value_with_handler(
                        *left,
                        ValueStackFrame::Range {
                            range_limits: *range_limits,
                            state: RangePath::OnLeftBranch { right: *right },
                        },
                    ),
                }
            }
            ExpressionNode::Assignment {
                assignee,
                equals_token,
                value,
            } => next.read_value_with_handler(
                *value,
                ValueStackFrame::HandleValueForAssignment {
                    assignee: *assignee,
                    equals_token: *equals_token,
                },
            ),
            ExpressionNode::CompoundAssignment {
                place,
                operation,
                value,
            } => next.read_value_with_handler(
                *value,
                ValueStackFrame::HandleValueForCompoundAssignment {
                    place: *place,
                    operation: *operation,
                },
            ),
        })
    }

    fn handle_as_assignee(
        &self,
        nodes: &[ExpressionNode<Source>],
        self_node_id: ExpressionNodeId,
        value: ExpressionValue,
        _: &mut Interpreter,
        next: ActionCreator,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(SourceExpressionLeaf::Variable(_))
            | ExpressionNode::Leaf(SourceExpressionLeaf::Discarded(_))
            | ExpressionNode::Index { .. }
            | ExpressionNode::Property { .. } => {
                next.read_place_with_handler(self_node_id, PlaceStackFrame::Assignment { value })
            }
            ExpressionNode::Array {
                brackets,
                items: assignee_item_node_ids,
            } => {
                ArrayAssigneeStackFrame::new(nodes, brackets.join(), assignee_item_node_ids, value)?
                    .handle_next(next)
            }
            ExpressionNode::Grouped { inner, .. } => {
                next.handle_assignment_and_return_to(*inner, value, AssignmentStackFrame::Grouped)
            }
            other => {
                return other
                    .operator_span_range()
                    .execution_err("This type of expression is not supported as an assignee");
            }
        })
    }

    fn handle_as_place(
        &self,
        interpreter: &mut Interpreter,
        next: ActionCreator,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(SourceExpressionLeaf::Variable(variable)) => next.return_place(
                Place::MutableReference(variable.reference(interpreter)?.into_mut()?),
            ),
            ExpressionNode::Leaf(SourceExpressionLeaf::Discarded(token)) => {
                next.return_place(Place::Discarded(*token))
            }
            ExpressionNode::Index {
                node,
                access,
                index,
            } => next.read_place_with_handler(
                *node,
                PlaceStackFrame::Indexed {
                    access: *access,
                    index: *index,
                },
            ),
            ExpressionNode::Property { node, access, .. } => next.read_place_with_handler(
                *node,
                PlaceStackFrame::PropertyAccess {
                    access: access.clone(),
                },
            ),
            ExpressionNode::Grouped { inner, .. } => {
                next.read_place_with_handler(*inner, PlaceStackFrame::Grouped)
            }
            other => {
                return other
                    .operator_span_range()
                    .execution_err("This type of expression is not supported as an assignee");
            }
        })
    }
}

/// Stack frames which need to receive a value to continue their execution.
enum ValueStackFrame {
    Group {
        span: Span,
    },
    Array(ArrayValueStackFrame),
    Object(ObjectValueStackFrame),
    UnaryOperation {
        operation: UnaryOperation,
    },
    BinaryOperation {
        operation: BinaryOperation,
        state: BinaryPath,
    },
    Property {
        access: PropertyAccess,
    },
    Index {
        access: IndexAccess,
        state: IndexPath,
    },
    Range {
        range_limits: syn::RangeLimits,
        state: RangePath,
    },
    HandleValueForAssignment {
        assignee: ExpressionNodeId,
        equals_token: Token![=],
    },
    HandleValueForCompoundAssignment {
        place: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
    },
    ResolveIndexedPlace {
        place: Place,
        access: IndexAccess,
    },
}

impl ValueStackFrame {
    fn ultimate_return_mode(&self) -> ReturnMode {
        match self {
            ValueStackFrame::Group { .. } => ReturnMode::Value,
            ValueStackFrame::Array(_) => ReturnMode::Value,
            ValueStackFrame::Object(_) => ReturnMode::Value,
            ValueStackFrame::UnaryOperation { .. } => ReturnMode::Value,
            ValueStackFrame::BinaryOperation { .. } => ReturnMode::Value,
            ValueStackFrame::Property { .. } => ReturnMode::Value,
            ValueStackFrame::Index { .. } => ReturnMode::Value,
            ValueStackFrame::Range { .. } => ReturnMode::Value,
            ValueStackFrame::HandleValueForAssignment { .. } => ReturnMode::Value,
            ValueStackFrame::HandleValueForCompoundAssignment { .. } => ReturnMode::Value,
            ValueStackFrame::ResolveIndexedPlace { .. } => ReturnMode::Place,
        }
    }

    fn handle_value(
        self,
        value: ExpressionValue,
        next: ActionCreator,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ValueStackFrame::Group { span } => next.return_value(value.with_span(span)),
            ValueStackFrame::UnaryOperation { operation } => {
                next.return_value(operation.evaluate(value)?)
            }
            ValueStackFrame::Array(mut array) => {
                array.evaluated_items.push(value);
                array.next(next)
            }
            ValueStackFrame::Object(object) => object.handle_value(value, next)?,
            ValueStackFrame::BinaryOperation { operation, state } => match state {
                BinaryPath::OnLeftBranch { right } => {
                    if let Some(result) = operation.lazy_evaluate(&value)? {
                        next.return_value(result)
                    } else {
                        next.read_value_with_handler(
                            right,
                            ValueStackFrame::BinaryOperation {
                                operation,
                                state: BinaryPath::OnRightBranch { left: value },
                            },
                        )
                    }
                }
                BinaryPath::OnRightBranch { left } => {
                    next.return_value(operation.evaluate(left, value)?)
                }
            },
            ValueStackFrame::Property { access } => next.return_value(value.into_property(access)?),
            ValueStackFrame::Index { access, state } => match state {
                IndexPath::OnObjectBranch { index } => next.read_value_with_handler(
                    index,
                    ValueStackFrame::Index {
                        access,
                        state: IndexPath::OnIndexBranch { object: value },
                    },
                ),
                IndexPath::OnIndexBranch { object } => {
                    next.return_value(object.into_indexed(access, value)?)
                }
            },
            ValueStackFrame::Range {
                range_limits,
                state,
            } => match (state, range_limits) {
                (RangePath::OnLeftBranch { right: Some(right) }, range_limits) => next
                    .read_value_with_handler(
                        right,
                        ValueStackFrame::Range {
                            range_limits,
                            state: RangePath::OnRightBranch { left: Some(value) },
                        },
                    ),
                (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::HalfOpen(token)) => {
                    let inner = ExpressionRangeInner::RangeFrom {
                        start_inclusive: value,
                        token,
                    };
                    next.return_value(inner.to_value(token.span_range()))
                }
                (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::Closed(_)) => {
                    unreachable!(
                        "A closed range should have been given a right in continue_range(..)"
                    )
                }
                (
                    RangePath::OnRightBranch { left: Some(left) },
                    syn::RangeLimits::HalfOpen(token),
                ) => {
                    let inner = ExpressionRangeInner::Range {
                        start_inclusive: left,
                        token,
                        end_exclusive: value,
                    };
                    next.return_value(inner.to_value(token.span_range()))
                }
                (
                    RangePath::OnRightBranch { left: Some(left) },
                    syn::RangeLimits::Closed(token),
                ) => {
                    let inner = ExpressionRangeInner::RangeInclusive {
                        start_inclusive: left,
                        token,
                        end_inclusive: value,
                    };
                    next.return_value(inner.to_value(token.span_range()))
                }
                (RangePath::OnRightBranch { left: None }, syn::RangeLimits::HalfOpen(token)) => {
                    let inner = ExpressionRangeInner::RangeTo {
                        token,
                        end_exclusive: value,
                    };
                    next.return_value(inner.to_value(token.span_range()))
                }
                (RangePath::OnRightBranch { left: None }, syn::RangeLimits::Closed(token)) => {
                    let inner = ExpressionRangeInner::RangeToInclusive {
                        token,
                        end_inclusive: value,
                    };
                    next.return_value(inner.to_value(token.span_range()))
                }
            },
            ValueStackFrame::HandleValueForAssignment {
                assignee,
                equals_token,
            } => next.handle_assignment_and_return_to(
                assignee,
                value,
                AssignmentStackFrame::AssignmentRoot { equals_token },
            ),
            ValueStackFrame::HandleValueForCompoundAssignment { place, operation } => next
                .read_place_with_handler(
                    place,
                    PlaceStackFrame::CompoundAssignmentRoot { operation, value },
                ),
            ValueStackFrame::ResolveIndexedPlace { place, access } => {
                next.return_place(match place {
                    Place::MutableReference(reference) => {
                        Place::MutableReference(reference.resolve_indexed(access, value)?)
                    }
                    Place::Discarded(_) => {
                        return access.execution_err("Cannot index into a discarded value");
                    }
                })
            }
        })
    }
}

struct ArrayValueStackFrame {
    span: Span,
    unevaluated_items: Vec<ExpressionNodeId>,
    evaluated_items: Vec<ExpressionValue>,
}

impl ArrayValueStackFrame {
    fn next(self, action_creator: ActionCreator) -> NextAction {
        match self
            .unevaluated_items
            .get(self.evaluated_items.len())
            .cloned()
        {
            Some(next) => {
                action_creator.read_value_with_handler(next, ValueStackFrame::Array(self))
            }
            None => action_creator.return_value(ExpressionValue::Array(ExpressionArray {
                items: self.evaluated_items,
                span_range: self.span.span_range(),
            })),
        }
    }
}

struct ObjectValueStackFrame {
    span: Span,
    pending: Option<PendingEntryPath>,
    unevaluated_entries: Vec<(ObjectKey, ExpressionNodeId)>,
    evaluated_entries: BTreeMap<String, ExpressionValue>,
}

impl ObjectValueStackFrame {
    fn handle_value(
        mut self,
        value: ExpressionValue,
        next: ActionCreator,
    ) -> ExecutionResult<NextAction> {
        let pending = self.pending.take();
        Ok(match pending {
            Some(PendingEntryPath::OnIndexKeyBranch {
                brackets,
                value_node,
            }) => {
                let key = value.expect_string("An object key")?.value;
                if self.evaluated_entries.contains_key(&key) {
                    return brackets.execution_err(format!("The key {} has already been set", key));
                }
                self.pending = Some(PendingEntryPath::OnValueBranch { key });
                next.read_value_with_handler(value_node, ValueStackFrame::Object(self))
            }
            Some(PendingEntryPath::OnValueBranch { key }) => {
                self.evaluated_entries.insert(key, value);
                self.next(next)?
            }
            None => {
                unreachable!("Should not receive a value without a pending handler set")
            }
        })
    }

    fn next(mut self, action_creator: ActionCreator) -> ExecutionResult<NextAction> {
        Ok(
            match self
                .unevaluated_entries
                .get(self.evaluated_entries.len())
                .cloned()
            {
                Some((ObjectKey::Identifier(ident), node)) => {
                    let key = ident.to_string();
                    if self.evaluated_entries.contains_key(&key) {
                        return ident
                            .execution_err(format!("The key {} has already been set", key));
                    }
                    self.pending = Some(PendingEntryPath::OnValueBranch { key });
                    action_creator.read_value_with_handler(node, ValueStackFrame::Object(self))
                }
                Some((ObjectKey::Indexed { brackets, index }, value_node)) => {
                    self.pending = Some(PendingEntryPath::OnIndexKeyBranch {
                        brackets,
                        value_node,
                    });
                    action_creator.read_value_with_handler(index, ValueStackFrame::Object(self))
                }
                None => action_creator
                    .return_value(self.evaluated_entries.to_value(self.span.span_range())),
            },
        )
    }
}

enum PendingEntryPath {
    OnIndexKeyBranch {
        brackets: Brackets,
        value_node: ExpressionNodeId,
    },
    OnValueBranch {
        key: String,
    },
}

enum BinaryPath {
    OnLeftBranch { right: ExpressionNodeId },
    OnRightBranch { left: ExpressionValue },
}

enum IndexPath {
    OnObjectBranch { index: ExpressionNodeId },
    OnIndexBranch { object: ExpressionValue },
}

enum RangePath {
    OnLeftBranch { right: Option<ExpressionNodeId> },
    OnRightBranch { left: Option<ExpressionValue> },
}

enum AssignmentStackFrame {
    /// An instruction to return a `None` value to the Value stack
    AssignmentRoot {
        #[allow(unused)]
        equals_token: Token![=],
    },
    Grouped,
    Array(ArrayAssigneeStackFrame),
}

impl AssignmentStackFrame {
    fn ultimate_return_mode(&self) -> ReturnMode {
        match self {
            AssignmentStackFrame::AssignmentRoot { .. } => ReturnMode::Value,
            AssignmentStackFrame::Grouped { .. } => ReturnMode::AssignmentCompletion,
            AssignmentStackFrame::Array { .. } => ReturnMode::AssignmentCompletion,
        }
    }

    fn handle_assignment_complete(
        self,
        completion: AssignmentCompletion,
        next: ActionCreator,
    ) -> ExecutionResult<NextAction> {
        let AssignmentCompletion { span_range } = completion;
        Ok(match self {
            AssignmentStackFrame::AssignmentRoot { .. } => {
                next.return_value(ExpressionValue::None(span_range))
            }
            AssignmentStackFrame::Grouped => next.return_assignment_completion(span_range),
            AssignmentStackFrame::Array(array) => array.handle_next(next),
        })
    }
}

struct ArrayAssigneeStackFrame {
    span_range: SpanRange,
    assignee_stack: Vec<(ExpressionNodeId, ExpressionValue)>,
}

impl ArrayAssigneeStackFrame {
    /// See also `ArrayPattern` in `patterns.rs`
    fn new(
        nodes: &[ExpressionNode<Source>],
        assignee_span: Span,
        assignee_item_node_ids: &[ExpressionNodeId],
        value: ExpressionValue,
    ) -> ExecutionResult<Self> {
        let array = value.expect_array("The assignee of an array place")?;
        let span_range = SpanRange::new_between(assignee_span, array.span_range.end());
        let mut has_seen_dot_dot = false;
        let mut prefix_assignees = Vec::new();
        let mut suffix_assignees = Vec::new();
        for node in assignee_item_node_ids.iter() {
            match nodes[node.0] {
                ExpressionNode::Range {
                    left: None,
                    range_limits: syn::RangeLimits::HalfOpen(_),
                    right: None,
                } => {
                    if has_seen_dot_dot {
                        return assignee_span
                            .execution_err("Only one .. is allowed in an array assignee");
                    }
                    has_seen_dot_dot = true;
                }
                _ => {
                    if has_seen_dot_dot {
                        suffix_assignees.push(*node);
                    } else {
                        prefix_assignees.push(*node);
                    }
                }
            }
        }

        let array_length = array.items.len();

        let mut assignee_pairs: Vec<_> = if has_seen_dot_dot {
            let total_assignees = prefix_assignees.len() + suffix_assignees.len();
            if total_assignees > array_length {
                return assignee_span.execution_err(format!(
                    "The array has {} items, but the assignee expected at least {}",
                    array_length, total_assignees,
                ));
            }
            let discarded_count =
                array.items.len() - prefix_assignees.len() - suffix_assignees.len();
            let assignees = prefix_assignees
                .into_iter()
                .map(Some)
                .chain(std::iter::repeat(None).take(discarded_count))
                .chain(suffix_assignees.into_iter().map(Some));

            assignees
                .zip(array.items)
                .filter_map(|(assignee, value)| Some((assignee?, value)))
                .collect()
        } else {
            let total_assignees = prefix_assignees.len();
            if total_assignees != array_length {
                return assignee_span.execution_err(format!(
                    "The array has {} items, but the assignee expected {}",
                    array_length, total_assignees,
                ));
            }
            prefix_assignees.into_iter().zip(array.items).collect()
        };
        Ok(Self {
            span_range,
            assignee_stack: {
                assignee_pairs.reverse();
                assignee_pairs
            },
        })
    }

    fn handle_next(mut self, next: ActionCreator) -> NextAction {
        match self.assignee_stack.pop() {
            Some((node, value)) => {
                next.handle_assignment_and_return_to(node, value, AssignmentStackFrame::Array(self))
            }
            None => next.return_assignment_completion(self.span_range),
        }
    }
}

struct AssignmentCompletion {
    span_range: SpanRange,
}

enum PlaceStackFrame {
    Assignment {
        value: ExpressionValue,
    },
    CompoundAssignmentRoot {
        operation: CompoundAssignmentOperation,
        value: ExpressionValue,
    },
    Grouped,
    PropertyAccess {
        access: PropertyAccess,
    },
    Indexed {
        access: IndexAccess,
        index: ExpressionNodeId,
    },
}

impl PlaceStackFrame {
    fn ultimate_return_mode(&self) -> ReturnMode {
        match self {
            PlaceStackFrame::Assignment { .. } => ReturnMode::AssignmentCompletion,
            PlaceStackFrame::CompoundAssignmentRoot { .. } => ReturnMode::Value,
            PlaceStackFrame::Grouped { .. } => ReturnMode::Place,
            PlaceStackFrame::PropertyAccess { .. } => ReturnMode::Place,
            PlaceStackFrame::Indexed { .. } => ReturnMode::Place,
        }
    }

    fn handle_place(self, place: Place, next: ActionCreator) -> ExecutionResult<NextAction> {
        Ok(match self {
            PlaceStackFrame::Assignment { value } => {
                let span_range = match place {
                    Place::MutableReference(mut variable) => {
                        let span_range =
                            SpanRange::new_between(variable.span_range(), value.span_range());
                        variable.set(value);
                        span_range
                    }
                    Place::Discarded(token) => token.span_range(),
                };
                next.return_assignment_completion(span_range)
            }
            PlaceStackFrame::CompoundAssignmentRoot { operation, value } => {
                let span_range = match place {
                    Place::MutableReference(mut variable) => {
                        let span_range =
                            SpanRange::new_between(variable.span_range(), value.span_range());
                        variable
                            .value_mut()
                            .handle_compound_assignment(&operation, value, span_range)?;
                        span_range
                    }
                    Place::Discarded(token) => token.span_range(),
                };
                next.return_value(ExpressionValue::None(span_range))
            }
            PlaceStackFrame::PropertyAccess { access } => match place {
                Place::MutableReference(reference) => {
                    next.return_place(Place::MutableReference(reference.resolve_property(access)?))
                }
                Place::Discarded(underscore) => {
                    return underscore
                        .execution_err("Cannot access the property of a discarded value")
                }
            },
            PlaceStackFrame::Grouped => next.return_place(place),
            PlaceStackFrame::Indexed { access, index } => next.read_value_with_handler(
                index,
                ValueStackFrame::ResolveIndexedPlace { place, access },
            ),
        })
    }
}

enum Place {
    MutableReference(MutableReference<ExpressionValue>),
    Discarded(Token![_]),
}
