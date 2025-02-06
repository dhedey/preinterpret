use super::*;

// Source
// =======================

#[derive(Clone)]
pub(crate) struct SourceExpression {
    inner: Expression<Source>,
}

impl HasSpanRange for SourceExpression {
    fn span_range(&self) -> SpanRange {
        self.inner.span_range
    }
}

impl Parse<Source> for SourceExpression {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            inner: ExpressionParser::parse(input)?,
        })
    }
}

impl SourceExpression {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<ExpressionOutput> {
        Ok(ExpressionOutput {
            value: self.evaluate_to_value(interpreter)?,
            fallback_output_span: self.inner.span_range.join_into_span_else_start(),
        })
    }

    pub(crate) fn evaluate_with_span(
        &self,
        interpreter: &mut Interpreter,
        fallback_output_span: Span,
    ) -> ExecutionResult<ExpressionOutput> {
        Ok(ExpressionOutput {
            value: self.evaluate_to_value(interpreter)?,
            fallback_output_span,
        })
    }

    pub(crate) fn evaluate_to_value(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<ExpressionValue> {
        Source::evaluate_to_value(&self.inner, interpreter)
    }
}

pub(super) enum SourceExpressionLeaf {
    Command(Command),
    GroupedVariable(GroupedVariable),
    CodeBlock(SourceCodeBlock),
    Value(ExpressionValue),
}

impl Expressionable for Source {
    type Leaf = SourceExpressionLeaf;
    type EvaluationContext = Interpreter;

    fn leaf_end_span(leaf: &Self::Leaf) -> Option<Span> {
        match leaf {
            SourceExpressionLeaf::Command(command) => Some(command.span()),
            SourceExpressionLeaf::GroupedVariable(variable) => Some(variable.span_range().end()),
            SourceExpressionLeaf::CodeBlock(code_block) => Some(code_block.span()),
            SourceExpressionLeaf::Value(value) => value.source_span(),
        }
    }

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(Some(output_kind)) => {
                match output_kind.expression_support() {
                    Ok(()) => UnaryAtom::Leaf(Self::Leaf::Command(input.parse()?)),
                    Err(error_message) => return input.parse_err(error_message),
                }
            }
            SourcePeekMatch::Command(None) => return input.parse_err("Invalid command"),
            SourcePeekMatch::GroupedVariable => UnaryAtom::Leaf(Self::Leaf::GroupedVariable(input.parse()?)),
            SourcePeekMatch::FlattenedVariable => return input.parse_err("Flattened variables cannot be used directly in expressions. Consider removing the .. or wrapping it inside a command such as [!group! ..] which returns an expression"),
            SourcePeekMatch::AppendVariableBinding => return input.parse_err("Append variable operations are not supported in an expression"),
            SourcePeekMatch::ExplicitTransformStream | SourcePeekMatch::Transformer(_) => return input.parse_err("Destructurings are not supported in an expression"),
            SourcePeekMatch::Group(Delimiter::None | Delimiter::Parenthesis) => {
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Group(delim_span)
            },
            SourcePeekMatch::Group(Delimiter::Brace) => {
                let leaf = SourceExpressionLeaf::CodeBlock(input.parse()?);
                UnaryAtom::Leaf(leaf)
            }
            SourcePeekMatch::Group(Delimiter::Bracket) => return input.parse_err("Square brackets [ .. ] are not supported in an expression"),
            SourcePeekMatch::Punct(_) => {
                UnaryAtom::PrefixUnaryOperation(input.parse()?)
            },
            SourcePeekMatch::Ident(_) => {
                let value = ExpressionValue::Boolean(ExpressionBoolean::for_litbool(input.parse()?));
                UnaryAtom::Leaf(Self::Leaf::Value(value))
            },
            SourcePeekMatch::Literal(_) => {
                let value = ExpressionValue::for_literal(input.parse()?)?;
                UnaryAtom::Leaf(Self::Leaf::Value(value))
            },
            SourcePeekMatch::End => return input.parse_err("The expression ended in an incomplete state"),
        })
    }

    fn parse_extension(input: &mut ParseStreamStack<Self>) -> ParseResult<NodeExtension> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Punct(_) => match input.try_parse_or_revert::<BinaryOperation>() {
                Ok(operation) => NodeExtension::BinaryOperation(operation),
                Err(_) => NodeExtension::NoneMatched,
            },
            SourcePeekMatch::Ident(ident) if ident == "as" => {
                let cast_operation =
                    UnaryOperation::for_cast_operation(input.parse()?, input.parse_any_ident()?)?;
                NodeExtension::PostfixOperation(cast_operation)
            }
            _ => NodeExtension::NoneMatched,
        })
    }

    fn evaluate_leaf(
        leaf: &Self::Leaf,
        interpreter: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue> {
        let interpreted = match leaf {
            SourceExpressionLeaf::Command(command) => {
                command.clone().interpret_to_new_stream(interpreter)?
            }
            SourceExpressionLeaf::GroupedVariable(grouped_variable) => {
                grouped_variable.interpret_to_new_stream(interpreter)?
            }
            SourceExpressionLeaf::CodeBlock(code_block) => {
                code_block.clone().interpret_to_new_stream(interpreter)?
            }
            SourceExpressionLeaf::Value(value) => return Ok(value.clone()),
        };
        let parsed_expression = unsafe {
            // RUST-ANALYZER SAFETY: This isn't very safe, as it could have a none-delimited group in it
            interpreted.parse_as::<OutputExpression>()?
        };
        parsed_expression.evaluate_to_value()
    }
}

// Output
// ===========

#[derive(Clone)]
pub(crate) struct OutputExpression {
    inner: Expression<Output>,
}

impl Parse<Output> for OutputExpression {
    fn parse(input: ParseStream<Output>) -> ParseResult<Self> {
        Ok(Self {
            inner: ExpressionParser::parse(input)?,
        })
    }
}

impl OutputExpression {
    pub(crate) fn evaluate(&self) -> ExecutionResult<ExpressionOutput> {
        Ok(ExpressionOutput {
            value: self.evaluate_to_value()?,
            fallback_output_span: self.inner.span_range.join_into_span_else_start(),
        })
    }

    pub(crate) fn evaluate_to_value(&self) -> ExecutionResult<ExpressionValue> {
        Output::evaluate_to_value(&self.inner, &mut ())
    }
}

impl Expressionable for Output {
    type Leaf = ExpressionValue;
    type EvaluationContext = ();

    fn leaf_end_span(leaf: &Self::Leaf) -> Option<Span> {
        leaf.source_span()
    }

    fn evaluate_leaf(
        leaf: &Self::Leaf,
        _: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue> {
        Ok(leaf.clone())
    }

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>> {
        Ok(match input.peek_grammar() {
            OutputPeekMatch::Group(Delimiter::None | Delimiter::Parenthesis) => {
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Group(delim_span)
            }
            OutputPeekMatch::Group(Delimiter::Brace) => {
                return input
                    .parse_err("Curly braces are not supported in a re-interpreted expression")
            }
            OutputPeekMatch::Group(Delimiter::Bracket) => {
                return input.parse_err("Square brackets [ .. ] are not supported in an expression")
            }
            OutputPeekMatch::Ident(_) => UnaryAtom::Leaf(ExpressionValue::Boolean(
                ExpressionBoolean::for_litbool(input.parse()?),
            )),
            OutputPeekMatch::Punct(_) => UnaryAtom::PrefixUnaryOperation(input.parse()?),
            OutputPeekMatch::Literal(_) => {
                UnaryAtom::Leaf(ExpressionValue::for_literal(input.parse()?)?)
            }
            OutputPeekMatch::End => {
                return input.parse_err("The expression ended in an incomplete state")
            }
        })
    }

    fn parse_extension(input: &mut ParseStreamStack<Self>) -> ParseResult<NodeExtension> {
        Ok(match input.peek_grammar() {
            OutputPeekMatch::Punct(_) => match input.try_parse_or_revert::<BinaryOperation>() {
                Ok(operation) => NodeExtension::BinaryOperation(operation),
                Err(_) => NodeExtension::NoneMatched,
            },
            OutputPeekMatch::Ident(ident) if ident == "as" => {
                let cast_operation =
                    UnaryOperation::for_cast_operation(input.parse()?, input.parse_any_ident()?)?;
                NodeExtension::PostfixOperation(cast_operation)
            }
            _ => NodeExtension::NoneMatched,
        })
    }
}

// Generic
// =======

pub(super) struct Expression<K: Expressionable> {
    pub(super) root: ExpressionNodeId,
    pub(super) span_range: SpanRange,
    pub(super) nodes: std::rc::Rc<[ExpressionNode<K>]>,
}

impl<K: Expressionable> Clone for Expression<K> {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            span_range: self.span_range,
            nodes: self.nodes.clone(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionNodeId(pub(super) usize);

pub(super) enum ExpressionNode<K: Expressionable> {
    Leaf(K::Leaf),
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

pub(super) trait Expressionable: Sized {
    type Leaf;
    type EvaluationContext;

    fn leaf_end_span(leaf: &Self::Leaf) -> Option<Span>;

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>>;
    fn parse_extension(input: &mut ParseStreamStack<Self>) -> ParseResult<NodeExtension>;

    fn evaluate_leaf(
        leaf: &Self::Leaf,
        context: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue>;

    fn evaluate_to_value(
        expression: &Expression<Self>,
        context: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue> {
        ExpressionEvaluator::new(&expression.nodes).evaluate(expression.root, context)
    }
}
