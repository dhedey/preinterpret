use super::*;

pub(super) struct EvaluationTree {
    /// We store the tree as a normalized stack of nodes to make it easier to evaluate
    /// without risking hitting stack overflow issues for deeply unbalanced trees
    evaluation_stack: Vec<EvaluationNode>,
}

impl EvaluationTree {
    pub(super) fn build_from(expression: &Expr) -> Result<EvaluationTree> {
        EvaluationTreeBuilder::new(expression).build()
    }

    pub(super) fn evaluate(mut self) -> Result<EvaluationOutput> {
        loop {
            let EvaluationNode { result_placement, content } = self.evaluation_stack.pop()
                .expect("The builder should ensure that the stack is non-empty and has a final element of a RootResult which results in a return below.");
            let result = content.evaluate()?;
            match result_placement {
                ResultPlacement::RootResult => return Ok(result),
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
    fn build(mut self) -> Result<EvaluationTree> {
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
                        .span_range()
                        .err("This expression is not supported in preinterpret expressions");
                }
            }
        }
        Ok(EvaluationTree {
            evaluation_stack: self.evaluation_stack,
        })
    }

    fn add_binary_operation(
        &mut self,
        placement: ResultPlacement,
        operation: BinaryOperation,
        lhs: &'a Expr,
        rhs: &'a Expr,
    ) {
        let parent_node_stack_index = self.evaluation_stack.len();
        self.evaluation_stack.push(EvaluationNode {
            result_placement: placement,
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
            content: EvaluationNodeContent::Literal(literal),
        });
    }
}

struct EvaluationNode {
    result_placement: ResultPlacement,
    content: EvaluationNodeContent,
}

enum EvaluationNodeContent {
    Literal(EvaluationValue),
    Operator(EvaluationOperator),
}

impl EvaluationNodeContent {
    fn evaluate(self) -> Result<EvaluationOutput> {
        match self {
            Self::Literal(literal) => Ok(EvaluationOutput::Value(literal)),
            Self::Operator(operator) => operator.evaluate(),
        }
    }

    fn set_unary_input(&mut self, input: EvaluationOutput) {
        match self {
            Self::Operator(EvaluationOperator::Unary {
                input: existing_input,
                ..
            }) => *existing_input = Some(input),
            _ => panic!("Attempted to set unary input on a non-unary operator"),
        }
    }

    fn set_binary_left_input(&mut self, input: EvaluationOutput) {
        match self {
            Self::Operator(EvaluationOperator::Binary {
                left_input: existing_input,
                ..
            }) => *existing_input = Some(input),
            _ => panic!("Attempted to set binary left input on a non-binary operator"),
        }
    }

    fn set_binary_right_input(&mut self, input: EvaluationOutput) {
        match self {
            Self::Operator(EvaluationOperator::Binary {
                right_input: existing_input,
                ..
            }) => *existing_input = Some(input),
            _ => panic!("Attempted to set binary right input on a non-binary operator"),
        }
    }
}

pub(crate) enum EvaluationOutput {
    Value(EvaluationValue),
}

pub(super) trait ToEvaluationOutput: Sized {
    fn to_output(self, span: SpanRange) -> EvaluationOutput;
}

impl EvaluationOutput {
    pub(super) fn expect_value_pair(
        self,
        operator: PairedBinaryOperator,
        right: EvaluationOutput,
        operator_span: SpanRange,
    ) -> Result<EvaluationLiteralPair> {
        let left_lit = self.into_value();
        let right_lit = right.into_value();
        Ok(match (left_lit, right_lit) {
            (EvaluationValue::Integer(left), EvaluationValue::Integer(right)) => {
                let integer_pair = match (left.value, right.value) {
                    (EvaluationIntegerValue::Untyped(untyped_lhs), rhs) => match rhs {
                        EvaluationIntegerValue::Untyped(untyped_rhs) => {
                            EvaluationIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationIntegerValue::U8(rhs) => {
                            EvaluationIntegerValuePair::U8(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U16(rhs) => {
                            EvaluationIntegerValuePair::U16(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U32(rhs) => {
                            EvaluationIntegerValuePair::U32(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U64(rhs) => {
                            EvaluationIntegerValuePair::U64(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::U128(rhs) => {
                            EvaluationIntegerValuePair::U128(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::Usize(rhs) => {
                            EvaluationIntegerValuePair::Usize(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I8(rhs) => {
                            EvaluationIntegerValuePair::I8(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I16(rhs) => {
                            EvaluationIntegerValuePair::I16(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I32(rhs) => {
                            EvaluationIntegerValuePair::I32(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I64(rhs) => {
                            EvaluationIntegerValuePair::I64(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::I128(rhs) => {
                            EvaluationIntegerValuePair::I128(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationIntegerValue::Isize(rhs) => {
                            EvaluationIntegerValuePair::Isize(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, EvaluationIntegerValue::Untyped(untyped_rhs)) => match lhs {
                        EvaluationIntegerValue::Untyped(untyped_lhs) => {
                            EvaluationIntegerValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationIntegerValue::U8(lhs) => {
                            EvaluationIntegerValuePair::U8(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U16(lhs) => {
                            EvaluationIntegerValuePair::U16(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U32(lhs) => {
                            EvaluationIntegerValuePair::U32(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U64(lhs) => {
                            EvaluationIntegerValuePair::U64(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::U128(lhs) => {
                            EvaluationIntegerValuePair::U128(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::Usize(lhs) => {
                            EvaluationIntegerValuePair::Usize(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I8(lhs) => {
                            EvaluationIntegerValuePair::I8(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I16(lhs) => {
                            EvaluationIntegerValuePair::I16(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I32(lhs) => {
                            EvaluationIntegerValuePair::I32(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I64(lhs) => {
                            EvaluationIntegerValuePair::I64(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::I128(lhs) => {
                            EvaluationIntegerValuePair::I128(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationIntegerValue::Isize(lhs) => {
                            EvaluationIntegerValuePair::Isize(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (EvaluationIntegerValue::U8(lhs), EvaluationIntegerValue::U8(rhs)) => {
                        EvaluationIntegerValuePair::U8(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U16(lhs), EvaluationIntegerValue::U16(rhs)) => {
                        EvaluationIntegerValuePair::U16(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U32(lhs), EvaluationIntegerValue::U32(rhs)) => {
                        EvaluationIntegerValuePair::U32(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U64(lhs), EvaluationIntegerValue::U64(rhs)) => {
                        EvaluationIntegerValuePair::U64(lhs, rhs)
                    }
                    (EvaluationIntegerValue::U128(lhs), EvaluationIntegerValue::U128(rhs)) => {
                        EvaluationIntegerValuePair::U128(lhs, rhs)
                    }
                    (EvaluationIntegerValue::Usize(lhs), EvaluationIntegerValue::Usize(rhs)) => {
                        EvaluationIntegerValuePair::Usize(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I8(lhs), EvaluationIntegerValue::I8(rhs)) => {
                        EvaluationIntegerValuePair::I8(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I16(lhs), EvaluationIntegerValue::I16(rhs)) => {
                        EvaluationIntegerValuePair::I16(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I32(lhs), EvaluationIntegerValue::I32(rhs)) => {
                        EvaluationIntegerValuePair::I32(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I64(lhs), EvaluationIntegerValue::I64(rhs)) => {
                        EvaluationIntegerValuePair::I64(lhs, rhs)
                    }
                    (EvaluationIntegerValue::I128(lhs), EvaluationIntegerValue::I128(rhs)) => {
                        EvaluationIntegerValuePair::I128(lhs, rhs)
                    }
                    (EvaluationIntegerValue::Isize(lhs), EvaluationIntegerValue::Isize(rhs)) => {
                        EvaluationIntegerValuePair::Isize(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operator_span.err(format!("The {} operator cannot infer a common integer operand type from {} and {}. Consider using `as` to cast to matching types.", operator.symbol(), left_value.describe_type(), right_value.describe_type()));
                    }
                };
                EvaluationLiteralPair::Integer(integer_pair)
            }
            (EvaluationValue::Boolean(left), EvaluationValue::Boolean(right)) => {
                EvaluationLiteralPair::BooleanPair(left, right)
            }
            (EvaluationValue::Float(left), EvaluationValue::Float(right)) => {
                let float_pair = match (left.value, right.value) {
                    (EvaluationFloatValue::Untyped(untyped_lhs), rhs) => match rhs {
                        EvaluationFloatValue::Untyped(untyped_rhs) => {
                            EvaluationFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationFloatValue::F32(rhs) => {
                            EvaluationFloatValuePair::F32(untyped_lhs.parse_as()?, rhs)
                        }
                        EvaluationFloatValue::F64(rhs) => {
                            EvaluationFloatValuePair::F64(untyped_lhs.parse_as()?, rhs)
                        }
                    },
                    (lhs, EvaluationFloatValue::Untyped(untyped_rhs)) => match lhs {
                        EvaluationFloatValue::Untyped(untyped_lhs) => {
                            EvaluationFloatValuePair::Untyped(untyped_lhs, untyped_rhs)
                        }
                        EvaluationFloatValue::F32(lhs) => {
                            EvaluationFloatValuePair::F32(lhs, untyped_rhs.parse_as()?)
                        }
                        EvaluationFloatValue::F64(lhs) => {
                            EvaluationFloatValuePair::F64(lhs, untyped_rhs.parse_as()?)
                        }
                    },
                    (EvaluationFloatValue::F32(lhs), EvaluationFloatValue::F32(rhs)) => {
                        EvaluationFloatValuePair::F32(lhs, rhs)
                    }
                    (EvaluationFloatValue::F64(lhs), EvaluationFloatValue::F64(rhs)) => {
                        EvaluationFloatValuePair::F64(lhs, rhs)
                    }
                    (left_value, right_value) => {
                        return operator_span.err(format!("The {} operator cannot infer a common float operand type from {} and {}. Consider using `as` to cast to matching types.", operator.symbol(), left_value.describe_type(), right_value.describe_type()));
                    }
                };
                EvaluationLiteralPair::Float(float_pair)
            }
            (left, right) => {
                return operator_span.err(format!("The {} operator cannot infer a common operand type from {} and {}. Consider using `as` to cast to matching types.", operator.symbol(), left.describe_type(), right.describe_type()));
            }
        })
    }

    pub(crate) fn expect_integer(self, error_message: &str) -> Result<EvaluationInteger> {
        match self.into_value() {
            EvaluationValue::Integer(value) => Ok(value),
            other => other.source_span().err(error_message),
        }
    }

    pub(crate) fn expect_bool(self, error_message: &str) -> Result<EvaluationBoolean> {
        match self.into_value() {
            EvaluationValue::Boolean(value) => Ok(value),
            other => other.source_span().err(error_message),
        }
    }

    pub(super) fn into_value(self) -> EvaluationValue {
        match self {
            Self::Value(value) => value,
        }
    }

    pub(crate) fn into_interpreted_stream(self) -> InterpretedStream {
        let value = self.into_value();
        InterpretedStream::raw(value.source_span(), value.into_token_stream())
    }
}

impl From<EvaluationValue> for EvaluationOutput {
    fn from(literal: EvaluationValue) -> Self {
        Self::Value(literal)
    }
}

impl quote::ToTokens for EvaluationOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            EvaluationOutput::Value(evaluation_literal) => evaluation_literal.to_tokens(tokens),
        }
    }
}

enum ResultPlacement {
    RootResult,
    UnaryOperationInput { parent_node_stack_index: usize },
    BinaryOperationLeftChild { parent_node_stack_index: usize },
    BinaryOperationRightChild { parent_node_stack_index: usize },
}
