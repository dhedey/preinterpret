use super::*;

pub(super) struct ExpressionEvaluator<'a> {
    nodes: &'a [ExpressionNode],
    operation_stack: Vec<EvaluationStackFrame>,
}

impl<'a> ExpressionEvaluator<'a> {
    pub(super) fn new(nodes: &'a [ExpressionNode]) -> Self {
        Self {
            nodes,
            operation_stack: Vec::new(),
        }
    }

    pub(super) fn evaluate(
        mut self,
        root: ExpressionNodeId,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<EvaluationValue> {
        let mut next = self.begin_node_evaluation(root, interpreter)?;

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
                let interpreted = match leaf {
                    ExpressionLeaf::Command(command) => {
                        command.clone().interpret_to_new_stream(interpreter)?
                    }
                    ExpressionLeaf::GroupedVariable(grouped_variable) => {
                        grouped_variable.interpret_to_new_stream(interpreter)?
                    }
                    ExpressionLeaf::CodeBlock(code_block) => {
                        code_block.clone().interpret_to_new_stream(interpreter)?
                    }
                    ExpressionLeaf::Value(value) => {
                        return Ok(NextAction::HandleValue(value.clone()))
                    }
                };
                let parsed_expression = unsafe {
                    // RUST-ANALYZER SAFETY: This isn't very safe, as it could have a none-delimited group in it
                    interpreted.syn_parse(Expression::parse)?
                };
                NextAction::HandleValue(parsed_expression.evaluate_to_value(interpreter)?)
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
        evaluation_value: EvaluationValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match top_of_stack {
            EvaluationStackFrame::UnaryOperation { operation } => {
                let result = operation.evaluate(evaluation_value)?;
                NextAction::HandleValue(result)
            }
            EvaluationStackFrame::BinaryOperation { operation, state } => match state {
                BinaryPath::OnLeftBranch { right } => {
                    self.operation_stack
                        .push(EvaluationStackFrame::BinaryOperation {
                            operation,
                            state: BinaryPath::OnRightBranch {
                                left: evaluation_value,
                            },
                        });
                    NextAction::EnterNode(right)
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
    HandleValue(EvaluationValue),
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
    OnRightBranch { left: EvaluationValue },
}

pub(crate) struct EvaluationOutput {
    pub(super) value: EvaluationValue,
    pub(super) fallback_output_span: Span,
}

impl EvaluationOutput {
    #[allow(unused)]
    pub(super) fn into_value(self) -> EvaluationValue {
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

impl HasSpanRange for EvaluationOutput {
    fn span(&self) -> Span {
        self.value
            .source_span()
            .unwrap_or(self.fallback_output_span)
    }

    fn span_range(&self) -> SpanRange {
        self.span().span_range()
    }
}

impl quote::ToTokens for EvaluationOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_token_tree().to_tokens(tokens);
    }
}
