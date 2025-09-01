use super::*;

impl ExpressionNode<Source> {
    pub(super) fn handle_as_value(
        &self,
        interpreter: &mut Interpreter,
        context: Context<ValueType>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self {
            ExpressionNode::Leaf(leaf) => {
                context.return_owned_value(Source::evaluate_leaf(leaf, interpreter)?)
            }
            ExpressionNode::Grouped { delim_span, inner } => {
                GroupBuilder::start(context, delim_span, *inner)
            }
            ExpressionNode::Array { brackets, items } => {
                ArrayBuilder::start(context, brackets, items)
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
            } => RangeBuilder::start(context, left, range_limits, right),
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
            // TODO - Change this to follow outline in changelog
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
                let variable_ref = variable.reference(interpreter)?;
                match context.requested_ownership() {
                    RequestedPlaceOwnership::LateBound => {
                        match variable_ref.into_mut() {
                            Ok(place) => context.return_mutable(place),
                            Err(ExecutionInterrupt::Error(reason_not_mutable)) => {
                                // If we get an error with a mutable and shared reference, a mutable reference must already exist.
                                // We can just propogate the error from taking the shared reference, it should be good enough.
                                let shared_place =
                                    variable.reference(interpreter)?.into_shared()?;
                                context.return_shared(shared_place, Some(reason_not_mutable))
                            }
                            // Propogate any other errors, these shouldn't happen mind
                            Err(err) => Err(err)?,
                        }
                    }
                    RequestedPlaceOwnership::SharedReference => {
                        context.return_shared(variable_ref.into_shared()?, None)
                    }
                    RequestedPlaceOwnership::MutableReference => {
                        context.return_mutable(variable_ref.into_mut()?)
                    }
                }
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
