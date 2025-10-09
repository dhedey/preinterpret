use super::*;

#[derive(Clone)]
pub(crate) struct Expression {
    root: ExpressionNodeId,
    nodes: RcArena<ExpressionNodeId, ExpressionNode>,
}

impl ParseSource for Expression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        ExpressionParser::parse(input)
    }
}

impl Expression {
    pub(super) fn new(
        root: ExpressionNodeId,
        nodes: RcArena<ExpressionNodeId, ExpressionNode>,
    ) -> Self {
        Self { root, nodes }
    }

    pub(crate) fn evaluate(&self, interpreter: &mut Interpreter) -> ExecutionResult<OwnedValue> {
        ExpressionEvaluator::new(&self.nodes).evaluate(self.root, interpreter)
    }

    pub(crate) fn is_valid_as_statement_without_semicolon(&self) -> bool {
        // Must align with evaluate_as_statement
        matches!(
            &self.nodes.get(self.root),
            ExpressionNode::Leaf(Leaf::Block(_))
                | ExpressionNode::Leaf(Leaf::IfExpression(_))
                | ExpressionNode::Leaf(Leaf::LoopExpression(_))
                | ExpressionNode::Leaf(Leaf::WhileExpression(_))
                | ExpressionNode::Leaf(Leaf::ForExpression(_))
        )
    }

    pub(crate) fn evaluate_as_statement(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        // This must align with is_valid_as_statement_without_semicolon
        match &self.nodes.get(self.root) {
            ExpressionNode::Leaf(Leaf::Block(block)) => {
                block.evaluate(interpreter)?.into_statement_result()
            }
            ExpressionNode::Leaf(Leaf::IfExpression(if_expression)) => {
                if_expression.evaluate(interpreter)?.into_statement_result()
            }
            ExpressionNode::Leaf(Leaf::LoopExpression(loop_expression)) => {
                loop_expression.evaluate_as_statement(interpreter)
            }
            ExpressionNode::Leaf(Leaf::WhileExpression(while_expression)) => {
                while_expression.evaluate_as_statement(interpreter)
            }
            ExpressionNode::Leaf(Leaf::ForExpression(for_expression)) => {
                for_expression.evaluate_as_statement(interpreter)
            }
            _ => self.evaluate(interpreter)?.into_statement_result(),
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
        place: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
        value: ExpressionNodeId,
    },
}

impl ExpressionNode {
    pub(super) fn operator_span_range(&self) -> SpanRange {
        match self {
            ExpressionNode::Leaf(leaf) => leaf.span_range(),
            ExpressionNode::Grouped { delim_span, .. } => delim_span.span_range(),
            ExpressionNode::Array { brackets, .. } => brackets.span_range(),
            ExpressionNode::Object { braces, .. } => braces.span_range(),
            ExpressionNode::MethodCall { method, .. } => method.span_range(),
            ExpressionNode::Property { access, .. } => access.span_range(),
            ExpressionNode::Index { access, .. } => access.span_range(),
            ExpressionNode::UnaryOperation { operation, .. } => operation.span_range(),
            ExpressionNode::BinaryOperation { operation, .. } => operation.span_range(),
            ExpressionNode::Range { range_limits, .. } => range_limits.span_range(),
            ExpressionNode::Assignment { equals_token, .. } => equals_token.span_range(),
            ExpressionNode::CompoundAssignment { operation, .. } => operation.span_range(),
        }
    }
}

pub(super) enum Leaf {
    Block(ExpressionBlock),
    Command(Command),
    Variable(VariableIdentifier),
    Discarded(Token![_]),
    Value(SharedValue),
    StreamLiteral(StreamLiteral),
    IfExpression(IfExpression),
    LoopExpression(LoopExpression),
    WhileExpression(WhileExpression),
    ForExpression(ForExpression),
}

impl HasSpanRange for Leaf {
    fn span_range(&self) -> SpanRange {
        match self {
            Leaf::Command(command) => command.span_range(),
            Leaf::Variable(variable) => variable.span_range(),
            Leaf::Discarded(token) => token.span_range(),
            Leaf::Block(block) => block.span_range(),
            Leaf::Value(value) => value.span_range(),
            Leaf::StreamLiteral(stream) => stream.span_range(),
            Leaf::IfExpression(expression) => expression.span_range(),
            Leaf::LoopExpression(expression) => expression.span_range(),
            Leaf::WhileExpression(expression) => expression.span_range(),
            Leaf::ForExpression(expression) => expression.span_range(),
        }
    }
}
