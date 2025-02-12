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
