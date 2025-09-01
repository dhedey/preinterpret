use super::*;

pub(super) struct AssignmentCompletion {
    pub(super) span_range: SpanRange,
}

/// Handlers which return an AssignmentCompletion
pub(super) enum AnyAssignmentFrame {
    Place(PlaceAssigner),
    Grouped(GroupedAssigner),
    Array(ArrayBasedAssigner),
    Object(Box<ObjectBasedAssigner>),
}

impl AnyAssignmentFrame {
    pub(super) fn handle_item(
        self,
        context: Context<AssignmentType>,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        match self {
            Self::Place(frame) => frame.handle_item(context, item),
            Self::Grouped(frame) => frame.handle_item(context, item),
            Self::Object(frame) => frame.handle_item(context, item),
            Self::Array(frame) => frame.handle_item(context, item),
        }
    }
}

struct PrivateUnit;

pub(super) struct PlaceAssigner {
    value: ExpressionValue,
}

impl PlaceAssigner {
    pub(super) fn start(
        context: AssignmentContext,
        place: ExpressionNodeId,
        value: ExpressionValue,
    ) -> NextAction {
        let frame = Self { value };
        context.handle_node_as_place(frame, place, RequestedPlaceOwnership::MutableReference)
    }
}

impl EvaluationFrame for PlaceAssigner {
    type ReturnType = AssignmentType;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Place(self)
    }

    fn handle_item(
        self,
        context: AssignmentContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let mut mutable_place = item.expect_mutable_ref();
        let value = self.value;
        let span_range = SpanRange::new_between(mutable_place.span_range(), value.span_range());
        mutable_place.set(value);
        Ok(context.return_assignment_completion(span_range))
    }
}

pub(super) struct GroupedAssigner(PrivateUnit);

impl GroupedAssigner {
    pub(super) fn start(
        context: AssignmentContext,
        inner: ExpressionNodeId,
        value: ExpressionValue,
    ) -> NextAction {
        let frame = Self(PrivateUnit);
        context.handle_node_as_assignment(frame, inner, value)
    }
}

impl EvaluationFrame for GroupedAssigner {
    type ReturnType = AssignmentType;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Grouped(self)
    }

    fn handle_item(
        self,
        context: AssignmentContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let AssignmentCompletion { span_range } = item.expect_assignment_complete();
        Ok(context.return_assignment_completion(span_range))
    }
}

pub(super) struct ArrayBasedAssigner {
    span_range: SpanRange,
    assignee_stack: Vec<(ExpressionNodeId, ExpressionValue)>,
}

impl ArrayBasedAssigner {
    pub(super) fn start(
        context: AssignmentContext,
        nodes: &[ExpressionNode<Source>],
        brackets: &Brackets,
        assignee_item_node_ids: &[ExpressionNodeId],
        value: ExpressionValue,
    ) -> ExecutionResult<NextAction> {
        let frame = Self::new(nodes, brackets.join(), assignee_item_node_ids, value)?;
        Ok(frame.handle_next(context))
    }

    /// See also `ArrayPattern` in `patterns.rs`
    fn new(
        nodes: &[ExpressionNode<Source>],
        assignee_span: Span,
        assignee_item_node_ids: &[ExpressionNodeId],
        value: ExpressionValue,
    ) -> ExecutionResult<Self> {
        let array = value.expect_array("The value destructured as an array")?;
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

    fn handle_next(mut self, context: AssignmentContext) -> NextAction {
        match self.assignee_stack.pop() {
            Some((node, value)) => context.handle_node_as_assignment(self, node, value),
            None => context.return_assignment_completion(self.span_range),
        }
    }
}

impl EvaluationFrame for ArrayBasedAssigner {
    type ReturnType = AssignmentType;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Array(self)
    }

    fn handle_item(
        self,
        context: AssignmentContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let AssignmentCompletion { .. } = item.expect_assignment_complete();
        Ok(self.handle_next(context))
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
        value: ExpressionValue,
    ) -> ExecutionResult<NextAction> {
        let frame = Box::new(Self::new(braces.join(), assignee_pairs, value)?);
        frame.handle_next(context)
    }

    fn new(
        assignee_span: Span,
        assignee_pairs: &[(ObjectKey, ExpressionNodeId)],
        value: ExpressionValue,
    ) -> ExecutionResult<Self> {
        let object = value.expect_object("The value destructured as an object")?;
        let span_range = SpanRange::new_between(assignee_span, object.span_range.end());

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
        index: &ExpressionValue,
        assignee_node: ExpressionNodeId,
    ) -> ExecutionResult<NextAction> {
        let key = index.ref_expect_string("An object key")?.clone().value;
        let value = self.resolve_value(key, access.span())?;
        Ok(context.handle_node_as_assignment(self, assignee_node, value))
    }

    fn handle_next(mut self: Box<Self>, context: AssignmentContext) -> ExecutionResult<NextAction> {
        Ok(match self.unresolved_stack.pop() {
            Some((ObjectKey::Identifier(ident), assignee_node)) => {
                let key = ident.to_string();
                let value = self.resolve_value(key, ident.span())?;
                self.state = ObjectAssignmentState::WaitingForSubassignment;
                context.handle_node_as_assignment(self, assignee_node, value)
            }
            Some((ObjectKey::Indexed { index, access }, assignee_node)) => {
                self.state = ObjectAssignmentState::ResolvingIndex {
                    assignee_node,
                    access,
                };
                context.handle_node_as_value(
                    self,
                    index,
                    // This only needs to be read-only, as we are just using it to work out which field/s to assign
                    RequestedValueOwnership::SharedReference,
                )
            }
            None => context.return_assignment_completion(self.span_range),
        })
    }

    fn resolve_value(&mut self, key: String, key_span: Span) -> ExecutionResult<ExpressionValue> {
        if self.already_used_keys.contains(&key) {
            return key_span.execution_err(format!("The key `{}` has already used", key));
        }
        let value = self
            .entries
            .remove(&key)
            .map(|entry| entry.value)
            .unwrap_or_else(|| ExpressionValue::None(key_span.span_range()));
        self.already_used_keys.insert(key);
        Ok(value)
    }
}

impl EvaluationFrame for Box<ObjectBasedAssigner> {
    type ReturnType = AssignmentType;

    fn into_any(self) -> AnyAssignmentFrame {
        AnyAssignmentFrame::Object(self)
    }

    fn handle_item(
        self,
        context: AssignmentContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        match self.state {
            ObjectAssignmentState::ResolvingIndex {
                assignee_node,
                access,
            } => {
                let index_place = item.expect_shared_ref();
                self.handle_index_value(context, access, index_place.as_ref(), assignee_node)
            }
            ObjectAssignmentState::WaitingForSubassignment => {
                let AssignmentCompletion { .. } = item.expect_assignment_complete();
                self.handle_next(context)
            }
        }
    }
}
