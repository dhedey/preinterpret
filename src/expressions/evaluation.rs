use super::*;

pub(super) struct ExpressionEvaluator<'a, K: Expressionable> {
    nodes: &'a [ExpressionNode<K>],
    operation_stack: Vec<EvaluationStackFrame>,
}

impl<'a> ExpressionEvaluator<'a, Source> {
    pub(super) fn new(nodes: &'a [ExpressionNode<Source>]) -> Self {
        Self {
            nodes,
            operation_stack: Vec::new(),
        }
    }

    pub(super) fn evaluate(
        mut self,
        root: ExpressionNodeId,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<ExpressionValue> {
        let mut next = self.begin_node_evaluation(root, interpreter)?;

        loop {
            match next {
                NextAction::HandleValue(value) => {
                    let top_of_stack = match self.operation_stack.pop() {
                        Some(top) => top,
                        None => return Ok(value),
                    };
                    next = self.handle_value(top_of_stack, value, interpreter)?;
                }
                NextAction::EnterValueNode(next_node) => {
                    next = self.begin_node_evaluation(next_node, interpreter)?;
                }
            }
        }
    }

    fn begin_node_evaluation(
        &mut self,
        node_id: ExpressionNodeId,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<NextAction> {
        Ok(match &self.nodes[node_id.0] {
            ExpressionNode::Leaf(leaf) => {
                NextAction::HandleValue(Source::evaluate_leaf(leaf, interpreter)?)
            }
            ExpressionNode::Grouped { delim_span, inner } => {
                self.operation_stack.push(EvaluationStackFrame::Group {
                    span: delim_span.join(),
                });
                NextAction::EnterValueNode(*inner)
            }
            ExpressionNode::Array { delim_span, items } => ArrayStackFrame {
                span: delim_span.join(),
                unevaluated_items: items.clone(),
                evaluated_items: Vec::with_capacity(items.len()),
            }
            .next(&mut self.operation_stack),
            ExpressionNode::UnaryOperation { operation, input } => {
                self.operation_stack
                    .push(EvaluationStackFrame::UnaryOperation {
                        operation: operation.clone(),
                    });
                NextAction::EnterValueNode(*input)
            }
            ExpressionNode::BinaryOperation {
                operation,
                left_input,
                right_input,
            } => {
                self.operation_stack
                    .push(EvaluationStackFrame::BinaryOperation {
                        operation: operation.clone(),
                        state: BinaryPath::OnLeftBranch {
                            right: *right_input,
                        },
                    });
                NextAction::EnterValueNode(*left_input)
            }
            ExpressionNode::Property { node, access } => {
                self.operation_stack.push(EvaluationStackFrame::Property {
                    access: access.clone(),
                });
                NextAction::EnterValueNode(*node)
            }
            ExpressionNode::Index {
                node,
                access,
                index,
            } => {
                self.operation_stack.push(EvaluationStackFrame::Index {
                    access: access.clone(),
                    state: IndexPath::OnObjectBranch { index: *index },
                });
                NextAction::EnterValueNode(*node)
            }
            ExpressionNode::Range {
                left,
                range_limits,
                right,
            } => {
                match (left, right) {
                    (None, None) => match range_limits {
                        syn::RangeLimits::HalfOpen(token) => {
                            let inner = ExpressionRangeInner::RangeFull { token: *token };
                            NextAction::HandleValue(inner.to_value(token.span_range()))
                        }
                        syn::RangeLimits::Closed(_) => {
                            unreachable!("A closed range should have been given a right in continue_range(..)")
                        }
                    },
                    (None, Some(right)) => {
                        self.operation_stack.push(EvaluationStackFrame::Range {
                            range_limits: *range_limits,
                            state: RangePath::OnRightBranch { left: None },
                        });
                        NextAction::EnterValueNode(*right)
                    }
                    (Some(left), right) => {
                        self.operation_stack.push(EvaluationStackFrame::Range {
                            range_limits: *range_limits,
                            state: RangePath::OnLeftBranch { right: *right },
                        });
                        NextAction::EnterValueNode(*left)
                    }
                }
            }
            ExpressionNode::Assignment {
                assignee,
                equals_token,
                value,
            } => {
                self.operation_stack
                    .push(EvaluationStackFrame::AssignmentValue {
                        assignee: *assignee,
                        equals_token: *equals_token,
                    });
                NextAction::EnterValueNode(*value)
            }
            ExpressionNode::CompoundAssignment {
                place,
                operation,
                value,
            } => {
                self.operation_stack
                    .push(EvaluationStackFrame::CompoundAssignmentValue {
                        place: *place,
                        operation: *operation,
                    });
                NextAction::EnterValueNode(*value)
            }
        })
    }

    fn handle_value(
        &mut self,
        top_of_stack: EvaluationStackFrame,
        value: ExpressionValue,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<NextAction> {
        Ok(match top_of_stack {
            EvaluationStackFrame::Group { span } => NextAction::HandleValue(value.with_span(span)),
            EvaluationStackFrame::UnaryOperation { operation } => {
                NextAction::HandleValue(operation.evaluate(value)?)
            }
            EvaluationStackFrame::Array(mut array) => {
                array.evaluated_items.push(value);
                array.next(&mut self.operation_stack)
            }
            EvaluationStackFrame::BinaryOperation { operation, state } => match state {
                BinaryPath::OnLeftBranch { right } => {
                    if let Some(result) = operation.lazy_evaluate(&value)? {
                        NextAction::HandleValue(result)
                    } else {
                        self.operation_stack
                            .push(EvaluationStackFrame::BinaryOperation {
                                operation,
                                state: BinaryPath::OnRightBranch { left: value },
                            });
                        NextAction::EnterValueNode(right)
                    }
                }
                BinaryPath::OnRightBranch { left } => {
                    let result = operation.evaluate(left, value)?;
                    NextAction::HandleValue(result)
                }
            },
            EvaluationStackFrame::Property { access } => {
                let result = value.handle_property_access(access)?;
                NextAction::HandleValue(result)
            }
            EvaluationStackFrame::Index { access, state } => match state {
                IndexPath::OnObjectBranch { index } => {
                    self.operation_stack.push(EvaluationStackFrame::Index {
                        access,
                        state: IndexPath::OnIndexBranch { object: value },
                    });
                    NextAction::EnterValueNode(index)
                }
                IndexPath::OnIndexBranch { object } => {
                    let result = object.handle_index_access(access, value)?;
                    NextAction::HandleValue(result)
                }
            },
            EvaluationStackFrame::Range {
                range_limits,
                state,
            } => match (state, range_limits) {
                (RangePath::OnLeftBranch { right: Some(right) }, range_limits) => {
                    self.operation_stack.push(EvaluationStackFrame::Range {
                        range_limits,
                        state: RangePath::OnRightBranch { left: Some(value) },
                    });
                    NextAction::EnterValueNode(right)
                }
                (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::HalfOpen(token)) => {
                    let inner = ExpressionRangeInner::RangeFrom {
                        start_inclusive: value,
                        token,
                    };
                    NextAction::HandleValue(inner.to_value(token.span_range()))
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
                    NextAction::HandleValue(inner.to_value(token.span_range()))
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
                    NextAction::HandleValue(inner.to_value(token.span_range()))
                }
                (RangePath::OnRightBranch { left: None }, syn::RangeLimits::HalfOpen(token)) => {
                    let inner = ExpressionRangeInner::RangeTo {
                        token,
                        end_exclusive: value,
                    };
                    NextAction::HandleValue(inner.to_value(token.span_range()))
                }
                (RangePath::OnRightBranch { left: None }, syn::RangeLimits::Closed(token)) => {
                    let inner = ExpressionRangeInner::RangeToInclusive {
                        token,
                        end_inclusive: value,
                    };
                    NextAction::HandleValue(inner.to_value(token.span_range()))
                }
            },
            EvaluationStackFrame::AssignmentValue {
                assignee,
                equals_token,
            } => {
                // TODO: This should be replaced with setting a stack frame for AssignmentResolution
                self.handle_assignment(assignee, equals_token, value, interpreter)?
            }
            EvaluationStackFrame::CompoundAssignmentValue { place, operation } => {
                // TODO: This should be replaced with setting a stack frame for CompoundAssignmentResolution
                self.handle_compound_assignment(place, operation, value, interpreter)?
            }
        })
    }

    fn handle_assignment(
        &mut self,
        assignee: ExpressionNodeId,
        equals_token: Token![=],
        value: ExpressionValue,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<NextAction> {
        // TODO: When we add place resolution, we likely wish to make use of the execution stack.
        let assignee = self.resolve_assignee_or_place(assignee, interpreter)?;
        Ok(match assignee {
            Assignee::Place(place) => {
                let span_range =
                    SpanRange::new_between(place.span_range().start(), value.span_range().end());
                place.set(value)?;
                NextAction::HandleValue(ExpressionValue::None(span_range))
            }
            Assignee::CompoundWip => {
                return equals_token.execution_err("Compound assignment is not yet supported")
            }
        })
    }

    fn handle_compound_assignment(
        &mut self,
        place: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
        value: ExpressionValue,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<NextAction> {
        // TODO: When we add place resolution, we likely wish to make use of the execution stack.
        let assignee = self.resolve_assignee_or_place(place, interpreter)?;
        Ok(match assignee {
            Assignee::Place(place) => {
                let span_range =
                    SpanRange::new_between(place.span_range().start(), value.span_range().end());
                let left = place.get_cloned()?.clone();
                place.set(operation.to_binary().evaluate(left, value)?)?;
                NextAction::HandleValue(ExpressionValue::None(span_range))
            }
            Assignee::CompoundWip => {
                return operation
                    .execution_err("Compound values are not supported for operation assignment")
            }
        })
    }

    /// See the [rust reference] for a good description of assignee vs place.
    ///
    /// [rust reference]: https://doc.rust-lang.org/reference/expressions/assignment-expressions.html#assignee-vs-place
    fn resolve_assignee_or_place(
        &self,
        mut assignee: ExpressionNodeId,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Assignee> {
        let resolved = loop {
            match &self.nodes[assignee.0] {
                ExpressionNode::Leaf(SourceExpressionLeaf::Variable(variable)) => {
                    break Assignee::Place(variable.read_existing(interpreter)?);
                }
                ExpressionNode::Index { .. } => {
                    todo!()
                }
                ExpressionNode::Property { .. } => {
                    todo!()
                }
                ExpressionNode::Array { .. } => {
                    break Assignee::CompoundWip;
                }
                ExpressionNode::Grouped { inner, .. } => {
                    assignee = *inner;
                    continue;
                }
                other => {
                    return other
                        .operator_span_range()
                        .execution_err("This type of expression is not supported as an assignee");
                }
            };
        };
        Ok(resolved)
    }
}

enum Assignee {
    Place(VariableData),
    CompoundWip, // To come
}

enum NextAction {
    HandleValue(ExpressionValue),
    EnterValueNode(ExpressionNodeId),
}

enum EvaluationStackFrame {
    Group {
        span: Span,
    },
    Array(ArrayStackFrame),
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
    AssignmentValue {
        assignee: ExpressionNodeId,
        equals_token: Token![=],
    },
    CompoundAssignmentValue {
        place: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
    },
}

struct ArrayStackFrame {
    span: Span,
    unevaluated_items: Vec<ExpressionNodeId>,
    evaluated_items: Vec<ExpressionValue>,
}

impl ArrayStackFrame {
    fn next(self, operation_stack: &mut Vec<EvaluationStackFrame>) -> NextAction {
        match self
            .unevaluated_items
            .get(self.evaluated_items.len())
            .cloned()
        {
            Some(next) => {
                operation_stack.push(EvaluationStackFrame::Array(self));
                NextAction::EnterValueNode(next)
            }
            None => NextAction::HandleValue(ExpressionValue::Array(ExpressionArray {
                items: self.evaluated_items,
                span_range: self.span.span_range(),
            })),
        }
    }
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
