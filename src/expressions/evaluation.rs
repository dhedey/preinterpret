use super::*;

pub(super) struct ExpressionEvaluator<'a, K: Expressionable> {
    nodes: &'a [ExpressionNode<K>],
    operation_stack: Vec<EvaluationStackFrame>,
}

impl<'a, K: Expressionable> ExpressionEvaluator<'a, K> {
    pub(super) fn new(nodes: &'a [ExpressionNode<K>]) -> Self {
        Self {
            nodes,
            operation_stack: Vec::new(),
        }
    }

    pub(super) fn evaluate(
        mut self,
        root: ExpressionNodeId,
        evaluation_context: &mut K::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue> {
        let mut next = self.begin_node_evaluation(root, evaluation_context)?;

        loop {
            match next {
                NextAction::HandleValue(value) => {
                    let top_of_stack = match self.operation_stack.pop() {
                        Some(top) => top,
                        None => return Ok(value),
                    };
                    next = self.continue_node_evaluation(top_of_stack, value)?;
                }
                NextAction::EnterNode(next_node) => {
                    next = self.begin_node_evaluation(next_node, evaluation_context)?;
                }
            }
        }
    }

    fn begin_node_evaluation(
        &mut self,
        node_id: ExpressionNodeId,
        evaluation_context: &mut K::EvaluationContext,
    ) -> ExecutionResult<NextAction> {
        Ok(match &self.nodes[node_id.0] {
            ExpressionNode::Leaf(leaf) => {
                NextAction::HandleValue(K::evaluate_leaf(leaf, evaluation_context)?)
            }
            ExpressionNode::Grouped { delim_span, inner } => {
                self.operation_stack.push(EvaluationStackFrame::Group {
                    span: delim_span.join(),
                });
                NextAction::EnterNode(*inner)
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
                NextAction::EnterNode(*input)
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
                NextAction::EnterNode(*left_input)
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
                        NextAction::EnterNode(*right)
                    }
                    (Some(left), right) => {
                        self.operation_stack.push(EvaluationStackFrame::Range {
                            range_limits: *range_limits,
                            state: RangePath::OnLeftBranch { right: *right },
                        });
                        NextAction::EnterNode(*left)
                    }
                }
            }
        })
    }

    fn continue_node_evaluation(
        &mut self,
        top_of_stack: EvaluationStackFrame,
        value: ExpressionValue,
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
                        NextAction::EnterNode(right)
                    }
                }
                BinaryPath::OnRightBranch { left } => {
                    let result = operation.evaluate(left, value)?;
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
                    NextAction::EnterNode(right)
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
        })
    }
}

enum NextAction {
    HandleValue(ExpressionValue),
    EnterNode(ExpressionNodeId),
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
    Range {
        range_limits: syn::RangeLimits,
        state: RangePath,
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
                NextAction::EnterNode(next)
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

enum RangePath {
    OnLeftBranch { right: Option<ExpressionNodeId> },
    OnRightBranch { left: Option<ExpressionValue> },
}
