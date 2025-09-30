use super::*;

impl ExpressionNode<Source> {
    pub(super) fn handle_as_value(
        &self,
        interpreter: &mut Interpreter,
        context: Context<ValueType>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(leaf) => {
                match leaf {
                    SourceExpressionLeaf::Command(command) => {
                        // TODO[interpret_to_value]: Allow command to return a reference
                        context.return_owned(command.clone().interpret_to_value(interpreter)?)?
                    }
                    SourceExpressionLeaf::Discarded(token) => {
                        return token.execution_err("This cannot be used in a value expression");
                    }
                    SourceExpressionLeaf::Variable(variable_path) => {
                        let variable_ref = variable_path.binding(interpreter)?;
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
                    SourceExpressionLeaf::EmbeddedExpression(block) => {
                        // TODO[interpret_to_value]: Allow block to return reference
                        context.return_owned(block.interpret_to_value(interpreter)?)?
                    }
                    SourceExpressionLeaf::Value(value) => {
                        // We return a freely clonable CopyOnWrite in order to delay the clone of the literal if it's not necessary
                        let value = CopyOnWrite::shared_in_place_of_owned(Shared::clone(value));
                        context.return_copy_on_write(value)?
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
        _: &mut Interpreter,
        context: AssignmentContext,
        nodes: &[ExpressionNode<Source>],
        self_node_id: ExpressionNodeId,
        value: ExpressionValue,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(SourceExpressionLeaf::Variable(_))
            | ExpressionNode::Index { .. }
            | ExpressionNode::Property { .. } => PlaceAssigner::start(context, self_node_id, value),
            ExpressionNode::Leaf(SourceExpressionLeaf::Discarded(underscore)) => {
                context.return_assignment_completion(SpanRange::new_between(*underscore, value))
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

    pub(super) fn handle_as_place(
        &self,
        interpreter: &mut Interpreter,
        context: PlaceContext,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(SourceExpressionLeaf::Variable(variable)) => {
                let variable_ref = variable.binding(interpreter)?;
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
