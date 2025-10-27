use super::*;

pub(crate) struct Expression {
    root: ExpressionNodeId,
    nodes: Arena<ExpressionNodeId, ExpressionNode>,
}

impl ParseSource for Expression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        ExpressionParser::parse(input)
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        control_flow_visit(self.root, &mut self.nodes, context)
    }
}

impl Expression {
    pub(super) fn new(
        root: ExpressionNodeId,
        nodes: Arena<ExpressionNodeId, ExpressionNode>,
    ) -> Self {
        Self { root, nodes }
    }

    pub(super) fn is_block(&self) -> bool {
        matches!(
            self.nodes.get(self.root),
            ExpressionNode::Leaf(Leaf::Block(_))
        )
    }

    pub(super) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedValueOwnership,
    ) -> ExecutionResult<EvaluationItem> {
        ExpressionEvaluator::new(&self.nodes).evaluate(self.root, interpreter, ownership)
    }

    pub(crate) fn evaluate_owned(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        Ok(self
            .evaluate(interpreter, RequestedValueOwnership::owned())?
            .expect_owned())
    }

    pub(crate) fn evaluate_shared(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<SharedValue> {
        Ok(self
            .evaluate(interpreter, RequestedValueOwnership::shared())?
            .expect_shared())
    }

    pub(crate) fn is_valid_as_statement_without_semicolon(&self) -> bool {
        // Must align with evaluate_as_statement
        matches!(
            &self.nodes.get(self.root),
            ExpressionNode::Leaf(x) if x.is_valid_as_statement_without_semicolon(),
        )
    }

    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        // This must align with is_valid_as_statement_without_semicolon
        match &self.nodes.get(self.root) {
            ExpressionNode::Leaf(Leaf::Block(block)) => block
                .evaluate(interpreter, RequestedValueOwnership::owned())?
                .expect_owned()
                .into_statement_result(),
            ExpressionNode::Leaf(Leaf::IfExpression(if_expression)) => if_expression
                .evaluate(interpreter, RequestedValueOwnership::owned())?
                .expect_owned()
                .into_statement_result(),
            ExpressionNode::Leaf(Leaf::LoopExpression(loop_expression)) => {
                loop_expression.evaluate_as_statement(interpreter)
            }
            ExpressionNode::Leaf(Leaf::WhileExpression(while_expression)) => {
                while_expression.evaluate_as_statement(interpreter)
            }
            ExpressionNode::Leaf(Leaf::ForExpression(for_expression)) => {
                for_expression.evaluate_as_statement(interpreter)
            }
            _ => self.evaluate_owned(interpreter)?.into_statement_result(),
        }
    }
}

pub(super) enum ExpressionNode {
    Leaf(Leaf),
    Grouped {
        delim_span: DelimSpan,
        inner: ExpressionNodeId,
    },
    Array {
        brackets: Brackets,
        items: Vec<ExpressionNodeId>,
    },
    Object {
        braces: Braces,
        entries: Vec<(ObjectKey, ExpressionNodeId)>,
    },
    UnaryOperation {
        operation: UnaryOperation,
        input: ExpressionNodeId,
    },
    BinaryOperation {
        operation: BinaryOperation,
        left_input: ExpressionNodeId,
        right_input: ExpressionNodeId,
    },
    Property {
        node: ExpressionNodeId,
        access: PropertyAccess,
    },
    MethodCall {
        node: ExpressionNodeId,
        method: MethodAccess,
        parameters: Vec<ExpressionNodeId>,
    },
    Index {
        node: ExpressionNodeId,
        access: IndexAccess,
        index: ExpressionNodeId,
    },
    Range {
        left: Option<ExpressionNodeId>,
        range_limits: syn::RangeLimits,
        right: Option<ExpressionNodeId>,
    },
    Assignment {
        assignee: ExpressionNodeId,
        equals_token: Token![=],
        value: ExpressionNodeId,
    },
    CompoundAssignment {
        assignee: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
        value: ExpressionNodeId,
    },
}

pub(super) enum Leaf {
    Block(ExpressionBlock),
    Variable(VariableReference),
    Discarded(Token![_]),
    Value(SharedValue),
    StreamLiteral(StreamLiteral),
    IfExpression(IfExpression),
    LoopExpression(LoopExpression),
    WhileExpression(WhileExpression),
    ForExpression(ForExpression),
    AttemptExpression(AttemptExpression),
}

impl HasSpanRange for Leaf {
    fn span_range(&self) -> SpanRange {
        match self {
            Leaf::Variable(variable) => variable.span_range(),
            Leaf::Discarded(token) => token.span_range(),
            Leaf::Block(block) => block.span_range(),
            Leaf::Value(value) => value.span_range(),
            Leaf::StreamLiteral(stream) => stream.span_range(),
            Leaf::IfExpression(expression) => expression.span_range(),
            Leaf::LoopExpression(expression) => expression.span_range(),
            Leaf::WhileExpression(expression) => expression.span_range(),
            Leaf::ForExpression(expression) => expression.span_range(),
            Leaf::AttemptExpression(expression) => expression.span_range(),
        }
    }
}

impl Leaf {
    fn is_valid_as_statement_without_semicolon(&self) -> bool {
        match self {
            Leaf::Block(_)
            | Leaf::IfExpression(_)
            | Leaf::LoopExpression(_)
            | Leaf::WhileExpression(_)
            | Leaf::ForExpression(_)
            | Leaf::AttemptExpression(_) => true,
            Leaf::Variable(_) | Leaf::Discarded(_) | Leaf::Value(_) | Leaf::StreamLiteral(_) => {
                false
            }
        }
    }
}
