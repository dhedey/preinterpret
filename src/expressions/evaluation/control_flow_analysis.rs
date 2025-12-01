use super::*;

pub(in super::super) fn control_flow_visit(
    root: ExpressionNodeId,
    nodes: &mut Arena<ExpressionNodeId, ExpressionNode>,
    context: FlowCapturer,
) -> ParseResult<()> {
    let mut stack = ControlFlowStack::new();
    stack.push(root);
    while let Some(node_id) = stack.pop() {
        let node = nodes.get_mut(node_id);
        match node {
            ExpressionNode::Leaf(leaf) => {
                leaf.control_flow_pass(context)?;
            }
            ExpressionNode::Grouped { inner, .. } => {
                stack.push(*inner);
            }
            ExpressionNode::Array { items, .. } => {
                stack.push_reversed(items.iter().copied());
            }
            ExpressionNode::Object { entries, .. } => {
                let mut nodes = vec![];
                for (key, node_id) in entries.iter() {
                    match key {
                        ObjectKey::Identifier(_) => {}
                        ObjectKey::Indexed { index, .. } => {
                            nodes.push(*index);
                        }
                    }
                    nodes.push(*node_id);
                }
                stack.push_reversed(nodes);
            }
            ExpressionNode::UnaryOperation { input, .. } => {
                stack.push(*input);
            }
            ExpressionNode::BinaryOperation {
                left_input,
                right_input,
                ..
            } => {
                stack.push_reversed([*left_input, *right_input]);
            }
            ExpressionNode::Property { node, .. } => {
                stack.push(*node);
            }
            ExpressionNode::MethodCall {
                node, parameters, ..
            } => {
                stack.push_reversed(parameters.iter().copied());
                stack.push(*node); // This is a stack so this executes first
            }
            ExpressionNode::Index { node, index, .. } => {
                stack.push_reversed([*node, *index]);
            }
            ExpressionNode::Range { left, right, .. } => {
                // This is a stack, so left is activated first
                if let Some(right) = right {
                    stack.push(*right);
                }
                if let Some(left) = left {
                    stack.push(*left);
                }
            }
            ExpressionNode::Assignment {
                assignee,
                equals_token: _,
                value,
            } => {
                stack.push_reversed([*value, *assignee]);
            }
        }
    }
    Ok(())
}

struct ControlFlowStack {
    nodes: Vec<ExpressionNodeId>,
}

impl ControlFlowStack {
    fn new() -> Self {
        Self { nodes: vec![] }
    }

    fn push_reversed<I: DoubleEndedIterator<Item = ExpressionNodeId>>(
        &mut self,
        node: impl IntoIterator<IntoIter = I>,
    ) {
        self.nodes.extend(
            node.into_iter()
                .rev(),
        );
    }

    fn push(&mut self, node: ExpressionNodeId) {
        self.nodes.push(node);
    }

    fn pop(&mut self) -> Option<ExpressionNodeId> {
        self.nodes.pop()
    }
}

impl Leaf {
    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            Leaf::Variable(variable) => variable.control_flow_pass(context),
            Leaf::Block(block) => block.control_flow_pass(context),
            Leaf::StreamLiteral(stream_literal) => stream_literal.control_flow_pass(context),
            Leaf::IfExpression(if_expression) => if_expression.control_flow_pass(context),
            Leaf::LoopExpression(loop_expression) => loop_expression.control_flow_pass(context),
            Leaf::WhileExpression(while_expression) => while_expression.control_flow_pass(context),
            Leaf::ForExpression(for_expression) => for_expression.control_flow_pass(context),
            Leaf::AttemptExpression(attempt_expression) => {
                attempt_expression.control_flow_pass(context)
            }
            Leaf::ParseExpression(parse_expression) => parse_expression.control_flow_pass(context),
            Leaf::Discarded(_) => Ok(()),
            Leaf::Value(_) => Ok(()),
        }
    }
}
