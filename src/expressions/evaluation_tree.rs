use super::*;

pub(super) struct EvaluationTree {
    /// We store the tree as a normalized stack of nodes to make it easier to evaluate
    /// without risking hitting stack overflow issues for deeply unbalanced trees
    evaluation_stack: Vec<EvaluationNode>,
    fallback_output_span: Span,
}

impl EvaluationTree {
    pub(super) fn build_from(
        fallback_output_span: Span,
        expression: &Expr,
    ) -> ExecutionResult<EvaluationTree> {
        EvaluationTreeBuilder::new(expression).build(fallback_output_span)
    }

    pub(super) fn evaluate(mut self) -> ExecutionResult<EvaluationOutput> {
        loop {
            let EvaluationNode { result_placement, content } = self.evaluation_stack.pop()
                .expect("The builder should ensure that the stack is non-empty and has a final element of a RootResult which results in a return below.");
            let result = content.evaluate()?;
            match result_placement {
                ResultPlacement::RootResult => {
                    return Ok(EvaluationOutput {
                        value: result,
                        fallback_output_span: self.fallback_output_span,
                    })
                }
                ResultPlacement::UnaryOperationInput {
                    parent_node_stack_index,
                } => {
                    self.evaluation_stack[parent_node_stack_index]
                        .content
                        .set_unary_input(result);
                }
                ResultPlacement::BinaryOperationLeftChild {
                    parent_node_stack_index,
                } => {
                    self.evaluation_stack[parent_node_stack_index]
                        .content
                        .set_binary_left_input(result);
                }
                ResultPlacement::BinaryOperationRightChild {
                    parent_node_stack_index,
                } => {
                    self.evaluation_stack[parent_node_stack_index]
                        .content
                        .set_binary_right_input(result);
                }
            }
        }
    }
}

struct EvaluationTreeBuilder<'a> {
    work_stack: Vec<(&'a Expr, ResultPlacement)>,
    evaluation_stack: Vec<EvaluationNode>,
}

impl<'a> EvaluationTreeBuilder<'a> {
    fn new(expression: &'a Expr) -> Self {
        Self {
            work_stack: vec![(expression, ResultPlacement::RootResult)],
            evaluation_stack: Vec::new(),
        }
    }

    /// Attempts to construct a preinterpret expression tree from a syn [Expr].
    /// It tries to align with the [rustc expression] building approach.
    /// [rustc expression]: https://doc.rust-lang.org/reference/expressions.html
    fn build(mut self, fallback_output_span: Span) -> ExecutionResult<EvaluationTree> {
        while let Some((expression, placement)) = self.work_stack.pop() {
            match expression {
                Expr::Binary(expr) => {
                    self.add_binary_operation(
                        placement,
                        BinaryOperation::for_binary_expression(expr)?,
                        &expr.left,
                        &expr.right,
                    );
                }
                Expr::Cast(expr) => {
                    self.add_unary_operation(
                        placement,
                        UnaryOperation::for_cast_expression(expr)?,
                        &expr.expr,
                    );
                }
                Expr::Group(expr) => {
                    // We handle these as a no-op operation so that they get the span from the group
                    self.add_unary_operation(
                        placement,
                        UnaryOperation::for_group_expression(expr)?,
                        &expr.expr,
                    );
                }
                Expr::Lit(expr) => {
                    self.add_literal(placement, EvaluationValue::for_literal_expression(expr)?);
                }
                Expr::Paren(expr) => {
                    // We handle these as a no-op operation so that they get the span from the paren
                    self.add_unary_operation(
                        placement,
                        UnaryOperation::for_paren_expression(expr)?,
                        &expr.expr,
                    );
                }
                Expr::Unary(expr) => {
                    self.add_unary_operation(
                        placement,
                        UnaryOperation::for_unary_expression(expr)?,
                        &expr.expr,
                    );
                }
                other_expression => {
                    return other_expression
                        .span_range_from_iterating_over_all_tokens()
                        .execution_err(
                            "This expression is not supported in preinterpret expressions",
                        );
                }
            }
        }
        Ok(EvaluationTree {
            evaluation_stack: self.evaluation_stack,
            fallback_output_span,
        })
    }

    fn add_binary_operation(
        &mut self,
        result_placement: ResultPlacement,
        operation: BinaryOperation,
        lhs: &'a Expr,
        rhs: &'a Expr,
    ) {
        let parent_node_stack_index = self.evaluation_stack.len();
        self.evaluation_stack.push(EvaluationNode {
            result_placement,
            content: EvaluationNodeContent::Operator(EvaluationOperator::Binary {
                operation,
                left_input: None,
                right_input: None,
            }),
        });
        // Note - we put the lhs towards the end of the stack so it's evaluated first
        self.work_stack.push((
            rhs,
            ResultPlacement::BinaryOperationRightChild {
                parent_node_stack_index,
            },
        ));
        self.work_stack.push((
            lhs,
            ResultPlacement::BinaryOperationLeftChild {
                parent_node_stack_index,
            },
        ));
    }

    fn add_unary_operation(
        &mut self,
        placement: ResultPlacement,
        operation: UnaryOperation,
        input: &'a Expr,
    ) {
        let parent_node_stack_index = self.evaluation_stack.len();
        self.evaluation_stack.push(EvaluationNode {
            result_placement: placement,
            content: EvaluationNodeContent::Operator(EvaluationOperator::Unary {
                operation,
                input: None,
            }),
        });
        self.work_stack.push((
            input,
            ResultPlacement::UnaryOperationInput {
                parent_node_stack_index,
            },
        ));
    }

    fn add_literal(&mut self, placement: ResultPlacement, literal: EvaluationValue) {
        self.evaluation_stack.push(EvaluationNode {
            result_placement: placement,
            content: EvaluationNodeContent::Value(literal),
        });
    }
}

struct EvaluationNode {
    result_placement: ResultPlacement,
    content: EvaluationNodeContent,
}

enum EvaluationNodeContent {
    Value(EvaluationValue),
    Operator(EvaluationOperator),
}

impl EvaluationNodeContent {
    fn evaluate(self) -> ExecutionResult<EvaluationValue> {
        match self {
            Self::Value(value) => Ok(value),
            Self::Operator(operator) => operator.evaluate(),
        }
    }

    fn set_unary_input(&mut self, input: EvaluationValue) {
        match self {
            Self::Operator(EvaluationOperator::Unary {
                input: existing_input,
                ..
            }) => *existing_input = Some(input),
            _ => panic!("Attempted to set unary input on a non-unary operator"),
        }
    }

    fn set_binary_left_input(&mut self, input: EvaluationValue) {
        match self {
            Self::Operator(EvaluationOperator::Binary {
                left_input: existing_input,
                ..
            }) => *existing_input = Some(input),
            _ => panic!("Attempted to set binary left input on a non-binary operator"),
        }
    }

    fn set_binary_right_input(&mut self, input: EvaluationValue) {
        match self {
            Self::Operator(EvaluationOperator::Binary {
                right_input: existing_input,
                ..
            }) => *existing_input = Some(input),
            _ => panic!("Attempted to set binary right input on a non-binary operator"),
        }
    }
}

pub(crate) struct EvaluationOutput {
    value: EvaluationValue,
    fallback_output_span: Span,
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

enum ResultPlacement {
    RootResult,
    UnaryOperationInput { parent_node_stack_index: usize },
    BinaryOperationLeftChild { parent_node_stack_index: usize },
    BinaryOperationRightChild { parent_node_stack_index: usize },
}
