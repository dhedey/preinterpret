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
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        ExpressionEvaluator::new(&self.nodes).evaluate(self.root, interpreter, ownership)
    }

    pub(crate) fn evaluate_owned(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<OwnedValue> {
        Ok(self
            .evaluate(interpreter, RequestedOwnership::owned())?
            .expect_owned())
    }

    pub(crate) fn evaluate_shared(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<SharedValue> {
        Ok(self
            .evaluate(interpreter, RequestedOwnership::shared())?
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
                .evaluate(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(),
            ExpressionNode::Leaf(Leaf::IfExpression(if_expression)) => if_expression
                .evaluate(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(),
            ExpressionNode::Leaf(Leaf::LoopExpression(loop_expression)) => loop_expression
                .evaluate(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(),
            ExpressionNode::Leaf(Leaf::WhileExpression(while_expression)) => while_expression
                .evaluate(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(),
            ExpressionNode::Leaf(Leaf::ForExpression(for_expression)) => for_expression
                .evaluate(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(),
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
}

// We Box some of these variants to reduce the size of ExpressionNode
pub(super) enum Leaf {
    Block(Box<ExpressionBlock>),
    Variable(VariableReference),
    TypeProperty(TypeProperty),
    Discarded(Token![_]),
    Value(SharedValue),
    StreamLiteral(StreamLiteral),
    IfExpression(Box<IfExpression>),
    LoopExpression(Box<LoopExpression>),
    WhileExpression(Box<WhileExpression>),
    ForExpression(Box<ForExpression>),
    AttemptExpression(Box<AttemptExpression>),
    ParseExpression(Box<ParseExpression>),
}

impl HasSpanRange for Leaf {
    fn span_range(&self) -> SpanRange {
        match self {
            Leaf::Variable(variable) => variable.span_range(),
            Leaf::TypeProperty(type_property) => type_property.span_range(),
            Leaf::Discarded(token) => token.span_range(),
            Leaf::Block(block) => block.span_range(),
            Leaf::Value(value) => value.span_range(),
            Leaf::StreamLiteral(stream) => stream.span_range(),
            Leaf::IfExpression(expression) => expression.span_range(),
            Leaf::LoopExpression(expression) => expression.span_range(),
            Leaf::WhileExpression(expression) => expression.span_range(),
            Leaf::ForExpression(expression) => expression.span_range(),
            Leaf::AttemptExpression(expression) => expression.span_range(),
            Leaf::ParseExpression(expression) => expression.span_range(),
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
            | Leaf::AttemptExpression(_)
            | Leaf::ParseExpression(_) => true,
            Leaf::Variable(_)
            | Leaf::TypeProperty(_)
            | Leaf::Discarded(_)
            | Leaf::Value(_)
            | Leaf::StreamLiteral(_) => false,
        }
    }
}
