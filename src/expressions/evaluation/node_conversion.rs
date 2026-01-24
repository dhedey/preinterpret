use super::*;

impl ExpressionNode {
    pub(super) fn handle_as_value(
        &self,
        mut context: Context<ReturnsValue>,
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
                            context
                                .return_argument_value(Spanned(resolved, variable.span_range()))?
                        }
                    },
                    Leaf::TypeProperty(type_property) => {
                        context.evaluate(|_, ownership| type_property.resolve_spanned(ownership))?
                    }
                    Leaf::Block(block) => context.evaluate(|interpreter, ownership| {
                        block.evaluate_spanned(interpreter, ownership)
                    })?,
                    Leaf::Value(Spanned(value, span_range)) => {
                        // We return a freely clonable CopyOnWrite in order to delay the clone of the literal if it's not necessary
                        // This allows something like e.g. x[0][5][2] to only clone the innermost value instead of the full multi-dimensional array
                        let shared_cloned = Shared::clone(value);
                        let cow = CopyOnWriteValue::shared_in_place_of_owned(shared_cloned);
                        context.return_returned_value(Spanned(
                            ReturnedValue::CopyOnWrite(cow),
                            *span_range,
                        ))?
                    }
                    Leaf::StreamLiteral(stream_literal) => {
                        let span = stream_literal.span_range();
                        let value = context
                            .interpreter()
                            .capture_output(|interpreter| stream_literal.interpret(interpreter))?;
                        context.return_value(Spanned(value, span))?
                    }
                    Leaf::ParseTemplateLiteral(consume_literal) => {
                        context.evaluate(|interpreter, ownership| {
                            consume_literal.evaluate_spanned(interpreter, ownership)
                        })?
                    }
                    Leaf::IfExpression(if_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            if_expression.evaluate_spanned(interpreter, ownership)
                        })?
                    }
                    Leaf::LoopExpression(loop_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            loop_expression.evaluate_spanned(interpreter, ownership)
                        })?
                    }
                    Leaf::WhileExpression(while_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            while_expression.evaluate_spanned(interpreter, ownership)
                        })?
                    }
                    Leaf::ForExpression(for_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            for_expression.evaluate_spanned(interpreter, ownership)
                        })?
                    }
                    Leaf::AttemptExpression(attempt_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            attempt_expression.evaluate_spanned(interpreter, ownership)
                        })?
                    }
                    Leaf::ParseExpression(parse_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            parse_expression.evaluate_spanned(interpreter, ownership)
                        })?
                    }
                    Leaf::ClosureExpression(closure_expression) => {
                        context.evaluate(|interpreter, ownership| {
                            closure_expression.evaluate_spanned(interpreter, ownership)
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
            } => BinaryOperationBuilder::start(context, *operation, *left_input, *right_input),
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
            ExpressionNode::MethodCall {
                receiver,
                method,
                invocation,
            } => MethodCallBuilder::start(context, *receiver, method, invocation),
            ExpressionNode::Invocation {
                invokable,
                invocation,
            } => InvocationBuilder::start(context, *invokable, invocation),
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
        value: AnyValue,
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
