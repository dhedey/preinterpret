use super::*;

#[derive(Clone)]
pub(crate) struct Expression {
    pub(super) root: ExpressionNodeId,
    pub(super) span_range: SpanRange,
    pub(super) nodes: std::rc::Rc<[ExpressionNode]>,
}

impl HasSpanRange for Expression {
    fn span_range(&self) -> SpanRange {
        self.span_range
    }
}

impl Parse for Expression {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        ExpressionParser::new(input).parse()
    }
}

impl Expression {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<EvaluationOutput> {
        Ok(EvaluationOutput {
            value: self.evaluate_to_value(interpreter)?,
            fallback_output_span: self.span_range.join_into_span_else_start(),
        })
    }

    pub(crate) fn evaluate_with_span(
        &self,
        interpreter: &mut Interpreter,
        fallback_output_span: Span,
    ) -> ExecutionResult<EvaluationOutput> {
        Ok(EvaluationOutput {
            value: self.evaluate_to_value(interpreter)?,
            fallback_output_span,
        })
    }

    pub(crate) fn evaluate_to_value(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<EvaluationValue> {
        ExpressionEvaluator::new(&self.nodes).evaluate(self.root, interpreter)
    }
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionNodeId(pub(super) usize);

pub(super) enum ExpressionNode {
    Leaf(ExpressionLeaf),
    UnaryOperation {
        operation: UnaryOperation,
        input: ExpressionNodeId,
    },
    BinaryOperation {
        operation: BinaryOperation,
        left_input: ExpressionNodeId,
        right_input: ExpressionNodeId,
    },
}

pub(super) enum ExpressionLeaf {
    Command(Command),
    GroupedVariable(GroupedVariable),
    CodeBlock(CommandCodeInput),
    Value(EvaluationValue),
}
