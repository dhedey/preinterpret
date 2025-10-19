use super::*;

impl ExpressionNode {
    pub(super) fn handle_as_value(
        &self,
        mut context: Context<ValueType>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(leaf) => {
                match leaf {
                    Leaf::Discarded(token) => {
                        return token.execution_err("This cannot be used in a value expression.");
                    }
                    Leaf::Variable(variable) => match context.requested_ownership() {
                        RequestedValueOwnership::LateBound => {
                            let late_bound = variable.resolve_late_bound(context.interpreter())?;
                            context.return_late_bound(late_bound)?
                        }
                        RequestedValueOwnership::Concrete(ownership) => {
                            let resolved =
                                variable.resolve_resolved(context.interpreter(), ownership)?;
                            context.return_resolved_value(resolved)?
                        }
                    },
                    Leaf::Block(block) => {
                        let ownership = context.requested_ownership();
                        let item = block.evaluate(context.interpreter(), ownership)?;
                        context.return_item(item)?
                    }
                    Leaf::Value(value) => {
                        // We return a freely clonable CopyOnWrite in order to delay the clone of the literal if it's not necessary
                        // This allows something like e.g. x[0][5][2] to only clone the innermost value instead of the full multi-dimensional array
                        let value = CopyOnWrite::shared_in_place_of_owned(Shared::clone(value));
                        context.return_copy_on_write(value)?
                    }
                    Leaf::StreamLiteral(stream_literal) => {
                        let value = stream_literal.evaluate(context.interpreter())?;
                        context.return_owned(value.into_owned_value(stream_literal.span_range()))?
                    }
                    Leaf::IfExpression(if_expression) => {
                        let ownership = context.requested_ownership();
                        let item = if_expression.evaluate(context.interpreter(), ownership)?;
                        context.return_item(item)?
                    }
                    Leaf::LoopExpression(loop_expression) => {
                        let value =
                            loop_expression.evaluate_as_expression(context.interpreter())?;
                        context.return_owned(value)?
                    }
                    Leaf::WhileExpression(while_expression) => {
                        let value =
                            while_expression.evaluate_as_expression(context.interpreter())?;
                        context.return_owned(value)?
                    }
                    Leaf::ForExpression(for_expression) => {
                        let value = for_expression.evaluate_as_expression(context.interpreter())?;
                        context.return_owned(value)?
                    }
                }
            }
            ExpressionNode::Grouped { delim_span, inner } => {
                GroupBuilder::start(context, delim_span, *inner)
            }
            ExpressionNode::Array { brackets, items } => {
                ArrayBuilder::start(context, brackets, items)?
            }
            ExpressionNode::Object { braces, entries } => {
                ObjectBuilder::start(context, braces, entries)?
            }
            ExpressionNode::UnaryOperation { operation, input } => {
                UnaryOperationBuilder::start(context, operation.clone(), *input)
            }
            ExpressionNode::BinaryOperation {
                operation,
                left_input,
                right_input,
            } => {
                BinaryOperationBuilder::start(context, operation.clone(), *left_input, *right_input)
            }
            ExpressionNode::Property { node, access } => {
                ValuePropertyAccessBuilder::start(context, access.clone(), *node)
            }
            ExpressionNode::Index {
                node,
                access,
                index,
            } => ValueIndexAccessBuilder::start(context, *node, *access, *index),
            ExpressionNode::Range {
                left,
                range_limits,
                right,
            } => RangeBuilder::start(context, left, range_limits, right)?,
            ExpressionNode::Assignment {
                assignee,
                equals_token,
                value,
            } => AssignmentBuilder::start(context, *assignee, *equals_token, *value),
            ExpressionNode::CompoundAssignment {
                assignee,
                operation,
                value,
            } => CompoundAssignmentBuilder::start(context, *assignee, *operation, *value),
            ExpressionNode::MethodCall {
                node,
                method,
                parameters,
            } => MethodCallBuilder::start(context, *node, method.clone(), parameters),
        })
    }

    pub(super) fn handle_as_assignment_target(
        &self,
        context: AssignmentContext,
        nodes: &Arena<ExpressionNodeId, ExpressionNode>,
        self_node_id: ExpressionNodeId,
        // NB: This might intrisically be a part of a larger value, and might have been
        // created many lines previously, so doesn't have an obvious span associated with it
        // Instead, we put errors on the assignee syntax side
        value: ExpressionValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(Leaf::Discarded(underscore)) => {
                context.return_assignment_completion(underscore.span_range())
            }
            ExpressionNode::Array {
                brackets,
                items: assignee_item_node_ids,
            } => {
                ArrayBasedAssigner::start(context, nodes, brackets, assignee_item_node_ids, value)?
            }
            ExpressionNode::Object { braces, entries } => {
                ObjectBasedAssigner::start(context, braces, entries, value)?
            }
            ExpressionNode::Grouped { inner, .. } => GroupedAssigner::start(context, *inner, value),
            // This handles:
            // - Standard Variable assignment
            // - Property assignment (allowing for creation of fields)
            // - Index assignment (allowing for creation of keys)
            // - Assignment to any mutable value (e.g. x.as_mut())
            _ => AssigneeAssigner::start(context, self_node_id, value),
        })
    }

    pub(super) fn handle_as_assignee(
        &self,
        mut context: AssigneeContext,
        self_node_id: ExpressionNodeId,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(Leaf::Variable(variable)) => {
                let mutable = variable.resolve_assignee(context.interpreter())?;
                context.return_assignee(mutable)
            }
            ExpressionNode::Index {
                node,
                access,
                index,
            } => IndexedAssignee::start(context, *node, *access, *index),
            ExpressionNode::Property { node, access, .. } => {
                PropertyAccessedAssignee::start(context, *node, access.clone())
            }
            ExpressionNode::Grouped { inner, .. } => GroupedAssignee::start(context, *inner),
            // If we don't need special place-based handling (e.g. for creating a new entry in an object)
            // Then let's just resolve via a mutable value
            _ => ValueBasedAssignee::start(context, self_node_id),
        })
    }
}
