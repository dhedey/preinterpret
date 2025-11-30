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
                        return token.syntax_err("This cannot be used in a value expression.");
                    }
                    Leaf::Variable(variable) => match context.requested_ownership() {
                        RequestedOwnership::LateBound => {
                            let late_bound = variable.resolve_late_bound(context.interpreter())?;
                            context.return_late_bound(late_bound)?
                        }
                        RequestedOwnership::Concrete(ownership) => {
                            let resolved =
                                variable.resolve_concrete(context.interpreter(), ownership)?;
                            context.return_argument_value(resolved)?
                        }
                    },
                    Leaf::Block(block) => context.evaluate(|interpreter, ownership| {
                        block.evaluate(interpreter, ownership)
                    })?,
                    Leaf::Value(value) => {
                        // We return a freely clonable CopyOnWrite in order to delay the clone of the literal if it's not necessary
                        // This allows something like e.g. x[0][5][2] to only clone the innermost value instead of the full multi-dimensional array
                        let value = CopyOnWrite::shared_in_place_of_owned(Shared::clone(value));
                        context.return_returned_value(ReturnedValue::CopyOnWrite(value))?
                    }
                    Leaf::StreamLiteral(stream_literal) => {
                        let value = context
                            .interpreter()
                            .capture_output(|interpreter| stream_literal.interpret(interpreter))?;
                        context.return_value(value, stream_literal.span_range())?
                    }
                    Leaf::IfExpression(if_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            if_expression.evaluate(interpreter, ownership)
                        })?
                    }
                    Leaf::LoopExpression(loop_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            loop_expression.evaluate(interpreter, ownership)
                        })?
                    }
                    Leaf::WhileExpression(while_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            while_expression.evaluate(interpreter, ownership)
                        })?
                    }
                    Leaf::ForExpression(for_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            for_expression.evaluate(interpreter, ownership)
                        })?
                    }
                    Leaf::AttemptExpression(attempt_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            attempt_expression.evaluate(interpreter, ownership)
                        })?
                    }
                    Leaf::ParseExpression(parse_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            parse_expression.evaluate(interpreter, ownership)
                        })?
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
}
