use super::*;

impl ExpressionNode {
    pub(super) fn handle_as_value(
        &self,
        mut context: Context<ValueType>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(leaf) => {
                match leaf {
                    Leaf::Command(command) => {
                        // TODO[interpret_to_value]: Allow command to return a reference
                        let value = command.evaluate(context.interpreter())?;
                        context.return_owned(value.into_owned(command.span_range()))?
                    }
                    Leaf::Discarded(token) => {
                        return token.execution_err("This cannot be used in a value expression");
                    }
                    Leaf::Variable(variable_path) => {
                        let variable_ref = variable_path.binding(context.interpreter())?;
                        match context.requested_ownership() {
                            RequestedValueOwnership::LateBound => {
                                context.return_late_bound(variable_ref.into_late_bound()?)?
                            }
                            RequestedValueOwnership::Concrete(ownership) => match ownership {
                                ResolvedValueOwnership::Owned
                                | ResolvedValueOwnership::CopyOnWrite
                                | ResolvedValueOwnership::Shared => {
                                    context.return_shared(variable_ref.into_shared()?)?
                                }
                                ResolvedValueOwnership::Mutable => {
                                    context.return_mutable(variable_ref.into_mut()?)?
                                }
                            },
                        }
                    }
                    Leaf::Block(block) => {
                        // TODO[interpret_to_value]: Allow block to return reference
                        let value = block.evaluate(context.interpreter())?;
                        context.return_owned(value)?
                    }
                    Leaf::Value(value) => {
                        // We return a freely clonable CopyOnWrite in order to delay the clone of the literal if it's not necessary
                        let value = CopyOnWrite::shared_in_place_of_owned(Shared::clone(value));
                        context.return_copy_on_write(value)?
                    }
                    Leaf::StreamLiteral(stream_literal) => {
                        let value = stream_literal.clone().evaluate(context.interpreter())?;
                        context.return_owned(value.into_owned(stream_literal.span_range()))?
                    }
                    Leaf::IfExpression(if_expression) => {
                        let value = if_expression.evaluate(context.interpreter())?;
                        context.return_owned(value)?
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
                place,
                operation,
                value,
            } => CompoundAssignmentBuilder::start(context, *place, *operation, *value),
            ExpressionNode::MethodCall {
                node,
                method,
                parameters,
            } => MethodCallBuilder::start(context, *node, method.clone(), parameters),
        })
    }

    pub(super) fn handle_as_assignee(
        &self,
        context: AssignmentContext,
        nodes: &ReadOnlyArena<ExpressionNodeId, ExpressionNode>,
        self_node_id: ExpressionNodeId,
        // NB: This might intrisically be a part of a larger value, and might have been
        // created many lines previously, so doesn't have an obvious span associated with it
        // Instead, we put errors on the assignee syntax side
        value: ExpressionValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(Leaf::Variable(_))
            | ExpressionNode::Index { .. }
            | ExpressionNode::Property { .. } => PlaceAssigner::start(context, self_node_id, value),
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
            other => {
                return other
                    .operator_span_range()
                    .execution_err("This type of expression is not supported as an assignee. You may wish to use `_` to ignore the value.");
            }
        })
    }

    pub(super) fn handle_as_place(&self, mut context: PlaceContext) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(Leaf::Variable(variable)) => {
                let variable_ref = variable.binding(context.interpreter())?;
                context.return_place(variable_ref.into_mut()?)
            }
            ExpressionNode::Index {
                node,
                access,
                index,
            } => PlaceIndexer::start(context, *node, *access, *index),
            ExpressionNode::Property { node, access, .. } => {
                PlacePropertyAccessor::start(context, *node, access.clone())
            }
            ExpressionNode::Grouped { inner, .. } => PlaceGrouper::start(context, *inner),
            other => {
                return other
                    .operator_span_range()
                    .execution_err("This expression cannot be resolved into a memory location.");
            }
        })
    }
}
