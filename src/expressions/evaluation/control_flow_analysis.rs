use super::*;

pub(in super::super) fn control_flow_visit(
    root: ExpressionNodeId,
    nodes: &Arena<ExpressionNodeId, ExpressionNode>,
    context: FlowCapturer,
) -> ParseResult<()> {
    let mut stack = ControlFlowStack::new();
    stack.push_as_value(root);
    while let Some((as_kind, node_id)) = stack.pop() {
        let node = nodes.get(node_id);
        match as_kind {
            NodeAs::ValueOrAtomicAssignee => {
                // As per node-conversion / value_frames / assignee_frames
                // (NB - both value and atomic assignee frames have the same control flow)
                match node {
                    ExpressionNode::Leaf(leaf) => {
                        leaf.control_flow_pass(context)?;
                    }
                    ExpressionNode::Grouped { inner, .. } => {
                        stack.push_as_value(*inner);
                    }
                    ExpressionNode::Array { items, .. } => {
                        stack.push_as_value_reversed(items.iter().copied());
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
                        stack.push_as_value_reversed(nodes);
                    }
                    ExpressionNode::UnaryOperation { input, .. } => {
                        stack.push_as_value(*input);
                    }
                    ExpressionNode::BinaryOperation {
                        left_input,
                        right_input,
                        ..
                    } => {
                        stack.push_as_value_reversed([*left_input, *right_input]);
                    }
                    ExpressionNode::Property { node, .. } => {
                        stack.push_as_value(*node);
                    }
                    ExpressionNode::MethodCall {
                        node, parameters, ..
                    } => {
                        stack.push_as_value_reversed(parameters.iter().copied());
                        stack.push_as_value(*node); // This is a stack so this executes first
                    }
                    ExpressionNode::Index { node, index, .. } => {
                        stack.push_as_value_reversed([*node, *index]);
                    }
                    ExpressionNode::Range { left, right, .. } => {
                        // This is a stack, so left is activated first
                        if let Some(right) = right {
                            stack.push_as_value(*right);
                        }
                        if let Some(left) = left {
                            stack.push_as_value(*left);
                        }
                    }
                    ExpressionNode::Assignment {
                        assignee,
                        equals_token: _,
                        value,
                    } => {
                        stack.push_reversed([
                            (NodeAs::ValueOrAtomicAssignee, *value),
                            (NodeAs::Assignment, *assignee),
                        ]);
                    }
                    ExpressionNode::CompoundAssignment {
                        assignee,
                        operation: _,
                        value,
                    } => {
                        // NB - the assignee here is an atomic assignee, which is treated as a value for control flow purposes
                        stack.push_as_value_reversed([*value, *assignee]);
                    }
                }
            }
            NodeAs::Assignment => {
                // As per node-conversion / assignment_frames
                match node {
                    ExpressionNode::Grouped { inner, .. } => {
                        stack.push_as_assignment(*inner);
                    }
                    ExpressionNode::Object { entries, .. } => {
                        let mut nodes = vec![];
                        for (key, node_id) in entries.iter() {
                            match key {
                                ObjectKey::Identifier(_) => {}
                                ObjectKey::Indexed { index, .. } => {
                                    nodes.push((NodeAs::ValueOrAtomicAssignee, *index));
                                }
                            }
                            nodes.push((NodeAs::Assignment, *node_id));
                        }
                        stack.push_reversed(nodes);
                    }
                    ExpressionNode::Array { items, .. } => {
                        stack.push_as_assignment_reversed(items.iter().copied());
                    }
                    _ => {
                        stack.push_as_value(node_id);
                    }
                }
            }
        }
    }
    Ok(())
}

enum NodeAs {
    ValueOrAtomicAssignee,
    Assignment,
}

struct ControlFlowStack {
    nodes: Vec<(NodeAs, ExpressionNodeId)>,
}

impl ControlFlowStack {
    fn new() -> Self {
        Self { nodes: vec![] }
    }

    fn push_reversed<I: DoubleEndedIterator<Item = (NodeAs, ExpressionNodeId)>>(
        &mut self,
        node: impl IntoIterator<IntoIter = I>,
    ) {
        self.nodes.extend(node.into_iter().rev());
    }

    fn push_as_value_reversed<I: DoubleEndedIterator<Item = ExpressionNodeId>>(
        &mut self,
        node: impl IntoIterator<IntoIter = I>,
    ) {
        self.nodes.extend(
            node.into_iter()
                .rev()
                .map(|v| (NodeAs::ValueOrAtomicAssignee, v)),
        );
    }

    fn push_as_assignment_reversed<I: DoubleEndedIterator<Item = ExpressionNodeId>>(
        &mut self,
        node: impl IntoIterator<IntoIter = I>,
    ) {
        self.nodes
            .extend(node.into_iter().rev().map(|v| (NodeAs::Assignment, v)));
    }

    fn push_as_value(&mut self, node: ExpressionNodeId) {
        self.nodes.push((NodeAs::ValueOrAtomicAssignee, node));
    }

    fn push_as_assignment(&mut self, node: ExpressionNodeId) {
        self.nodes.push((NodeAs::Assignment, node));
    }

    fn pop(&mut self) -> Option<(NodeAs, ExpressionNodeId)> {
        self.nodes.pop()
    }
}

impl Leaf {
    fn control_flow_pass(&self, context: FlowCapturer) -> ParseResult<()> {
        match self {
            Leaf::Command(command) => command.control_flow_pass(context),
            Leaf::Variable(variable) => variable.control_flow_pass(context),
            Leaf::Block(block) => block.control_flow_pass(context),
            Leaf::StreamLiteral(stream_literal) => stream_literal.control_flow_pass(context),
            Leaf::IfExpression(if_expression) => if_expression.control_flow_pass(context),
            Leaf::LoopExpression(loop_expression) => loop_expression.control_flow_pass(context),
            Leaf::WhileExpression(while_expression) => while_expression.control_flow_pass(context),
            Leaf::ForExpression(for_expression) => for_expression.control_flow_pass(context),
            Leaf::Discarded(_) => Ok(()),
            Leaf::Value(_) => Ok(()),
        }
    }
}
