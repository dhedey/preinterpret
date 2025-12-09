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
        let Spanned(value, _span) =
            ExpressionEvaluator::new(&self.nodes).evaluate(self.root, interpreter, ownership)?;
        Ok(value)
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
        // TODO: Get proper span from expression nodes
        let fallback_span = Span::call_site().span_range();
        match &self.nodes.get(self.root) {
            ExpressionNode::Leaf(Leaf::Block(block)) => block
                .evaluate_unspanned(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(block.span().span_range()),
            ExpressionNode::Leaf(Leaf::IfExpression(if_expression)) => if_expression
                .evaluate_unspanned(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(if_expression.span_range()),
            ExpressionNode::Leaf(Leaf::LoopExpression(loop_expression)) => loop_expression
                .evaluate_unspanned(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(loop_expression.span_range()),
            ExpressionNode::Leaf(Leaf::WhileExpression(while_expression)) => while_expression
                .evaluate_unspanned(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(while_expression.span_range()),
            ExpressionNode::Leaf(Leaf::ForExpression(for_expression)) => for_expression
                .evaluate_unspanned(interpreter, RequestedOwnership::owned())?
                .expect_owned()
                .into_statement_result(for_expression.span_range()),
            _ => self
                .evaluate_owned(interpreter)?
                .into_statement_result(fallback_span),
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
    Value(Spanned<SharedValue>),
    StreamLiteral(StreamLiteral),
    ParseTemplateLiteral(ParseTemplateLiteral),
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
            Leaf::Value(spanned_value) => spanned_value.span_range(),
            Leaf::StreamLiteral(stream) => stream.span_range(),
            Leaf::ParseTemplateLiteral(stream) => stream.span_range(),
            Leaf::IfExpression(expression) => expression.span_range(),
            Leaf::LoopExpression(expression) => expression.span_range(),
            Leaf::WhileExpression(expression) => expression.span_range(),
            Leaf::ForExpression(expression) => expression.span_range(),
            Leaf::AttemptExpression(expression) => expression.span_range(),
            Leaf::ParseExpression(expression) => expression.span_range(),
        }
    }
}

impl ExpressionNode {
    /// Returns a span representing this node.
    ///
    /// For leaf nodes, this is the full span of the leaf.
    /// For compound nodes, this returns the span of the primary structural element
    /// (operator, brackets, etc.). The full span of a compound expression would
    /// require looking up child node spans in the Arena.
    pub(super) fn span_range(&self) -> SpanRange {
        match self {
            ExpressionNode::Leaf(leaf) => leaf.span_range(),
            ExpressionNode::Grouped { delim_span, .. } => delim_span.join().span_range(),
            ExpressionNode::Array { brackets, .. } => brackets.span_range(),
            ExpressionNode::Object { braces, .. } => braces.span_range(),
            ExpressionNode::UnaryOperation { operation, .. } => operation.span_range(),
            ExpressionNode::BinaryOperation { operation, .. } => operation.span_range(),
            ExpressionNode::Property { access, .. } => access.span_range(),
            ExpressionNode::MethodCall { method, .. } => method.span_range(),
            ExpressionNode::Index { access, .. } => access.span_range(),
            ExpressionNode::Range { range_limits, .. } => match range_limits {
                syn::RangeLimits::HalfOpen(token) => token.span_range(),
                syn::RangeLimits::Closed(token) => token.span_range(),
            },
            ExpressionNode::Assignment { equals_token, .. } => equals_token.span_range(),
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
            | Leaf::StreamLiteral(_)
            | Leaf::ParseTemplateLiteral(_) => false,
        }
    }
}
