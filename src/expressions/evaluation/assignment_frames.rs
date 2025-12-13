use super::*;

pub(crate) struct AssignmentCompletion;

/// Handlers which return an AssignmentCompletion
pub(super) enum AnyAssignmentFrame {
    Assignee(AssigneeAssigner),
    Grouped(GroupedAssigner),
    Array(ArrayBasedAssigner),
    Object(Box<ObjectBasedAssigner>),
}

impl AnyAssignmentFrame {
    pub(super) fn handle_next(
        self,
        context: Context<ReturnsAssignmentCompletion>,
        value: Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        match self {
            Self::Assignee(frame) => frame.handle_next(context, value),
            Self::Grouped(frame) => frame.handle_next(context, value),
            Self::Object(frame) => frame.handle_next(context, value),
            Self::Array(frame) => frame.handle_next(context, value),
        }
    }
}

struct PrivateUnit;

pub(super) struct AssigneeAssigner {
    value: Value,
}

impl AssigneeAssigner {
    pub(super) fn start(
        context: AssignmentContext,
        assignee: ExpressionNodeId,
        value: Value,
    ) -> NextAction {
        let frame = Self { value };
        context.request_assignee(frame, assignee, true)
    }
}

impl EvaluationFrame for AssigneeAssigner {
    type ReturnType = ReturnsAssignmentCompletion;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Assignee(self)
    }

    fn handle_next(
        self,
        context: AssignmentContext,
        Spanned(value, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let mut assignee = value.expect_assignee();
        let value = self.value;
        assignee.set(value);
        Ok(context.return_assignment_completion(span))
    }
}

pub(super) struct GroupedAssigner(PrivateUnit);

impl GroupedAssigner {
    pub(super) fn start(
        context: AssignmentContext,
        inner: ExpressionNodeId,
        value: Value,
    ) -> NextAction {
        let frame = Self(PrivateUnit);
        context.request_assignment(frame, inner, value)
    }
}

impl EvaluationFrame for GroupedAssigner {
    type ReturnType = ReturnsAssignmentCompletion;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Grouped(self)
    }

    fn handle_next(
        self,
        context: AssignmentContext,
        Spanned(value, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let AssignmentCompletion = value.expect_assignment_completion();
        Ok(context.return_assignment_completion(span))
    }
}

pub(super) struct ArrayBasedAssigner {
    span_range: SpanRange,
    assignee_stack: Vec<(ExpressionNodeId, Value)>,
}

impl ArrayBasedAssigner {
    pub(super) fn start(
        context: AssignmentContext,
        nodes: &Arena<ExpressionNodeId, ExpressionNode>,
        brackets: &Brackets,
        assignee_item_node_ids: &[ExpressionNodeId],
        value: Value,
    ) -> ExecutionResult<NextAction> {
        let frame = Self::new(nodes, brackets.join(), assignee_item_node_ids, value)?;
        Ok(frame.handle_next_subassignment(context))
    }

    /// See also `ArrayPattern` in `patterns.rs`
    fn new(
        nodes: &Arena<ExpressionNodeId, ExpressionNode>,
        assignee_span: Span,
        assignee_item_node_ids: &[ExpressionNodeId],
        value: Value,
    ) -> ExecutionResult<Self> {
        let span_range = assignee_span.span_range();
        let array: ArrayValue = Spanned(value.into_owned(), span_range)
            .resolve_as("The value destructured as an array")?;
        let mut has_seen_dot_dot = false;
        let mut prefix_assignees = Vec::new();
        let mut suffix_assignees = Vec::new();
        for node in assignee_item_node_ids.iter() {
            match nodes.get(*node) {
                ExpressionNode::Range {
                    left: None,
                    range_limits: syn::RangeLimits::HalfOpen(_),
                    right: None,
                } => {
                    if has_seen_dot_dot {
                        return assignee_span
                            .syntax_err("Only one .. is allowed in an array assignee");
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
                return assignee_span.value_err(format!(
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
                return assignee_span.value_err(format!(
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

    fn handle_next_subassignment(mut self, context: AssignmentContext) -> NextAction {
        match self.assignee_stack.pop() {
            Some((node, value)) => context.request_assignment(self, node, value),
            None => context.return_assignment_completion(self.span_range),
        }
    }
}

impl EvaluationFrame for ArrayBasedAssigner {
    type ReturnType = ReturnsAssignmentCompletion;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Array(self)
    }

    fn handle_next(
        self,
        context: AssignmentContext,
        Spanned(value, _span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let AssignmentCompletion = value.expect_assignment_completion();
        Ok(self.handle_next_subassignment(context))
    }
}

pub(super) struct ObjectBasedAssigner {
    span_range: SpanRange,
    entries: BTreeMap<String, ObjectEntry>,
    already_used_keys: HashSet<String>,
    unresolved_stack: Vec<(ObjectKey, ExpressionNodeId)>,
    state: ObjectAssignmentState,
}

enum ObjectAssignmentState {
    WaitingForSubassignment,
    ResolvingIndex {
        assignee_node: ExpressionNodeId,
        access: IndexAccess,
    },
}

impl ObjectBasedAssigner {
    pub(super) fn start(
        context: AssignmentContext,
        braces: &Braces,
        assignee_pairs: &[(ObjectKey, ExpressionNodeId)],
        value: Value,
    ) -> ExecutionResult<NextAction> {
        let frame = Box::new(Self::new(braces.join(), assignee_pairs, value)?);
        frame.handle_next_subassignment(context)
    }

    fn new(
        assignee_span: Span,
        assignee_pairs: &[(ObjectKey, ExpressionNodeId)],
        value: Value,
    ) -> ExecutionResult<Self> {
        let span_range = assignee_span.span_range();
        let object: ObjectValue = Spanned(value.into_owned(), span_range)
            .resolve_as("The value destructured as an object")?;

        Ok(Self {
            span_range,
            entries: object.entries,
            already_used_keys: HashSet::with_capacity(assignee_pairs.len()),
            unresolved_stack: assignee_pairs.iter().rev().cloned().collect(),
            state: ObjectAssignmentState::WaitingForSubassignment,
        })
    }

    fn handle_index_value(
        mut self: Box<Self>,
        context: AssignmentContext,
        access: IndexAccess,
        index: &Value,
        assignee_node: ExpressionNodeId,
    ) -> ExecutionResult<NextAction> {
        let key: &str = index
            .spanned(access.span_range())
            .resolve_as("An object key")?;
        let value = self.resolve_value(key.to_string(), access.span())?;
        Ok(context.request_assignment(self, assignee_node, value))
    }

    fn handle_next_subassignment(
        mut self: Box<Self>,
        context: AssignmentContext,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.unresolved_stack.pop() {
            Some((ObjectKey::Identifier(ident), assignee_node)) => {
                let key = ident.to_string();
                let value = self.resolve_value(key, ident.span())?;
                self.state = ObjectAssignmentState::WaitingForSubassignment;
                context.request_assignment(self, assignee_node, value)
            }
            Some((ObjectKey::Indexed { index, access }, assignee_node)) => {
                self.state = ObjectAssignmentState::ResolvingIndex {
                    assignee_node,
                    access,
                };
                // This only needs to be read-only, as we are just using it to work out which field/s to assign
                context.request_shared(self, index)
            }
            None => context.return_assignment_completion(self.span_range),
        })
    }

    fn resolve_value(&mut self, key: String, key_span: Span) -> ExecutionResult<Value> {
        if self.already_used_keys.contains(&key) {
            return key_span.syntax_err(format!("The key `{}` has already used", key));
        }
        let value = self
            .entries
            .remove(&key)
            .map(|entry| entry.value)
            .unwrap_or_else(|| Value::None);
        self.already_used_keys.insert(key);
        Ok(value)
    }
}

impl EvaluationFrame for Box<ObjectBasedAssigner> {
    type ReturnType = ReturnsAssignmentCompletion;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Object(self)
    }

    fn handle_next(
        self,
        context: AssignmentContext,
        Spanned(value, _span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        match self.state {
            ObjectAssignmentState::ResolvingIndex {
                assignee_node,
                access,
            } => {
                let index_place = value.expect_shared();
                self.handle_index_value(context, access, index_place.as_ref(), assignee_node)
            }
            ObjectAssignmentState::WaitingForSubassignment => {
                let AssignmentCompletion = value.expect_assignment_completion();
                self.handle_next_subassignment(context)
            }
        }
    }
}
