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
                NextAction::HandleValue(evaluation_value) => {
                    let top_of_stack = match self.operation_stack.pop() {
                        Some(top) => top,
                        None => return Ok(evaluation_value),
                    };
                    next = self.continue_node_evaluation(top_of_stack, evaluation_value)?;
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
        evaluation_value: ExpressionValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match top_of_stack {
            EvaluationStackFrame::UnaryOperation { operation } => {
                let result = operation.evaluate(evaluation_value)?;
                NextAction::HandleValue(result)
            }
            EvaluationStackFrame::BinaryOperation { operation, state } => match state {
                BinaryPath::OnLeftBranch { right } => {
                    if let Some(result) = operation.lazy_evaluate(&evaluation_value)? {
                        NextAction::HandleValue(result)
                    } else {
                        self.operation_stack
                            .push(EvaluationStackFrame::BinaryOperation {
                                operation,
                                state: BinaryPath::OnRightBranch {
                                    left: evaluation_value,
                                },
                            });
                        NextAction::EnterNode(right)
                    }
                }
                BinaryPath::OnRightBranch { left } => {
                    let result = operation.evaluate(left, evaluation_value)?;
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
    UnaryOperation {
        operation: UnaryOperation,
    },
    BinaryOperation {
        operation: BinaryOperation,
        state: BinaryPath,
    },
}

enum BinaryPath {
    OnLeftBranch { right: ExpressionNodeId },
    OnRightBranch { left: ExpressionValue },
}

pub(crate) struct EvaluationOutput {
    pub(super) value: ExpressionValue,
    pub(super) fallback_output_span: Span,
}

impl EvaluationOutput {
    #[allow(unused)]
    pub(super) fn into_value(self) -> ExpressionValue {
        self.value
    }

    #[allow(unused)]
    pub(crate) fn expect_integer(self, error_message: &str) -> ExecutionResult<EvaluationInteger> {
        let error_span = self.span();
        match self.value.into_integer() {
            Some(integer) => Ok(integer),
            None => error_span.execution_err(error_message),
        }
    }

    pub(crate) fn try_into_i128(self, error_message: &str) -> ExecutionResult<i128> {
        let error_span = self.span();
        match self
            .value
            .into_integer()
            .and_then(|integer| integer.try_into_i128())
        {
            Some(integer) => Ok(integer),
            None => error_span.execution_err(error_message),
        }
    }

    pub(crate) fn expect_bool(self, error_message: &str) -> ExecutionResult<bool> {
        let error_span = self.span();
        match self.value.into_bool() {
            Some(boolean) => Ok(boolean.value),
            None => error_span.execution_err(error_message),
        }
    }

    pub(crate) fn to_token_tree(&self) -> TokenTree {
        self.value.to_token_tree(self.fallback_output_span)
    }
}

impl HasSpan for EvaluationOutput {
    fn span(&self) -> Span {
        self.value
            .source_span()
            .unwrap_or(self.fallback_output_span)
    }
}

impl quote::ToTokens for EvaluationOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_token_tree().to_tokens(tokens);
    }
}
