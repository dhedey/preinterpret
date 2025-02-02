use super::*;

// Interpretation = Source
// =======================

#[derive(Clone)]
pub(crate) struct InterpretationExpression {
    inner: Expression<Source>,
}

impl HasSpanRange for InterpretationExpression {
    fn span_range(&self) -> SpanRange {
        self.inner.span_range
    }
}

impl ParseFromSource for InterpretationExpression {
    fn parse_from_source(input: SourceParseStream) -> ParseResult<Self> {
        Ok(Self {
            inner: ExpressionParser::parse(input)?,
        })
    }
}

impl InterpretationExpression {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<EvaluationOutput> {
        Ok(EvaluationOutput {
            value: self.evaluate_to_value(interpreter)?,
            fallback_output_span: self.inner.span_range.join_into_span_else_start(),
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
        Source::evaluate_to_value(&self.inner, interpreter)
    }
}

pub(super) enum InterpretationExpressionLeaf {
    Command(Command),
    GroupedVariable(GroupedVariable),
    CodeBlock(CommandCodeInput),
    Value(EvaluationValue),
}

impl Expressionable for Source {
    type Leaf = InterpretationExpressionLeaf;
    type EvaluationContext = Interpreter;

    fn leaf_end_span(leaf: &Self::Leaf) -> Option<Span> {
        match leaf {
            InterpretationExpressionLeaf::Command(command) => Some(command.span()),
            InterpretationExpressionLeaf::GroupedVariable(variable) => {
                Some(variable.span_range().end())
            }
            InterpretationExpressionLeaf::CodeBlock(code_block) => Some(code_block.span()),
            InterpretationExpressionLeaf::Value(value) => value.source_span(),
        }
    }

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>> {
        Ok(match input.peek_grammar() {
            GrammarPeekMatch::Command(Some(output_kind)) => {
                match output_kind.expression_support() {
                    Ok(()) => UnaryAtom::Leaf(Self::Leaf::Command(input.parse()?)),
                    Err(error_message) => return input.parse_err(error_message),
                }
            }
            GrammarPeekMatch::Command(None) => return input.parse_err("Invalid command"),
            GrammarPeekMatch::GroupedVariable => UnaryAtom::Leaf(Self::Leaf::GroupedVariable(input.parse()?)),
            GrammarPeekMatch::FlattenedVariable => return input.parse_err("Flattened variables cannot be used directly in expressions. Consider removing the .. or wrapping it inside a command such as [!group! ..] which returns an expression"),
            GrammarPeekMatch::AppendVariableDestructuring => return input.parse_err("Append variable operations are not supported in an expression"),
            GrammarPeekMatch::Destructurer(_) => return input.parse_err("Destructurings are not supported in an expression"),
            GrammarPeekMatch::Group(Delimiter::None | Delimiter::Parenthesis) => {
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Group(delim_span)
            },
            GrammarPeekMatch::Group(Delimiter::Brace) => {
                let leaf = InterpretationExpressionLeaf::CodeBlock(input.parse()?);
                UnaryAtom::Leaf(leaf)
            }
            GrammarPeekMatch::Group(Delimiter::Bracket) => return input.parse_err("Square brackets [ .. ] are not supported in an expression"),
            GrammarPeekMatch::Punct(_) => {
                UnaryAtom::PrefixUnaryOperation(input.parse()?)
            },
            GrammarPeekMatch::Ident(_) => {
                let value = EvaluationValue::Boolean(EvaluationBoolean::for_litbool(input.parse()?));
                UnaryAtom::Leaf(Self::Leaf::Value(value))
            },
            GrammarPeekMatch::Literal(_) => {
                let value = EvaluationValue::for_literal(input.parse()?)?;
                UnaryAtom::Leaf(Self::Leaf::Value(value))
            },
            GrammarPeekMatch::End => return input.parse_err("The expression ended in an incomplete state"),
        })
    }

    fn parse_extension(input: &mut ParseStreamStack<Self>) -> ParseResult<NodeExtension> {
        Ok(match input.peek_grammar() {
            GrammarPeekMatch::Punct(_) => match input.try_parse_or_revert::<BinaryOperation>() {
                Ok(operation) => NodeExtension::BinaryOperation(operation),
                Err(_) => NodeExtension::NoneMatched,
            },
            GrammarPeekMatch::Ident(ident) if ident == "as" => {
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
    ) -> ExecutionResult<EvaluationValue> {
        let interpreted = match leaf {
            InterpretationExpressionLeaf::Command(command) => {
                command.clone().interpret_to_new_stream(interpreter)?
            }
            InterpretationExpressionLeaf::GroupedVariable(grouped_variable) => {
                grouped_variable.interpret_to_new_stream(interpreter)?
            }
            InterpretationExpressionLeaf::CodeBlock(code_block) => {
                code_block.clone().interpret_to_new_stream(interpreter)?
            }
            InterpretationExpressionLeaf::Value(value) => return Ok(value.clone()),
        };
        let parsed_expression = unsafe {
            // RUST-ANALYZER SAFETY: This isn't very safe, as it could have a none-delimited group in it
            interpreted.parse_as::<InterpretedExpression>()?
        };
        parsed_expression.evaluate_to_value()
    }
}

// Interpreted
// ===========

#[derive(Clone)]
pub(crate) struct InterpretedExpression {
    inner: Expression<Interpreted>,
}

impl ParseFromInterpreted for InterpretedExpression {
    fn parse_from_interpreted(input: InterpretedParseStream) -> ParseResult<Self> {
        Ok(Self {
            inner: ExpressionParser::parse(input)?,
        })
    }
}

impl InterpretedExpression {
    pub(crate) fn evaluate(&self) -> ExecutionResult<EvaluationOutput> {
        Ok(EvaluationOutput {
            value: self.evaluate_to_value()?,
            fallback_output_span: self.inner.span_range.join_into_span_else_start(),
        })
    }

    pub(crate) fn evaluate_to_value(&self) -> ExecutionResult<EvaluationValue> {
        Interpreted::evaluate_to_value(&self.inner, &mut ())
    }
}

impl Expressionable for Interpreted {
    type Leaf = EvaluationValue;
    type EvaluationContext = ();

    fn leaf_end_span(leaf: &Self::Leaf) -> Option<Span> {
        leaf.source_span()
    }

    fn evaluate_leaf(
        leaf: &Self::Leaf,
        _: &mut Self::EvaluationContext,
    ) -> ExecutionResult<EvaluationValue> {
        Ok(leaf.clone())
    }

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>> {
        Ok(match input.peek_token() {
            InterpretedPeekMatch::Group(Delimiter::None | Delimiter::Parenthesis) => {
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Group(delim_span)
            }
            InterpretedPeekMatch::Group(Delimiter::Brace) => {
                return input
                    .parse_err("Curly braces are not supported in a re-interpreted expression")
            }
            InterpretedPeekMatch::Group(Delimiter::Bracket) => {
                return input.parse_err("Square brackets [ .. ] are not supported in an expression")
            }
            InterpretedPeekMatch::Ident(_) => UnaryAtom::Leaf(EvaluationValue::Boolean(
                EvaluationBoolean::for_litbool(input.parse()?),
            )),
            InterpretedPeekMatch::Punct(_) => UnaryAtom::PrefixUnaryOperation(input.parse()?),
            InterpretedPeekMatch::Literal(_) => {
                UnaryAtom::Leaf(EvaluationValue::for_literal(input.parse()?)?)
            }
            InterpretedPeekMatch::End => {
                return input.parse_err("The expression ended in an incomplete state")
            }
        })
    }

    fn parse_extension(input: &mut ParseStreamStack<Self>) -> ParseResult<NodeExtension> {
        Ok(match input.peek_token() {
            InterpretedPeekMatch::Punct(_) => {
                match input.try_parse_or_revert::<BinaryOperation>() {
                    Ok(operation) => NodeExtension::BinaryOperation(operation),
                    Err(_) => NodeExtension::NoneMatched,
                }
            }
            InterpretedPeekMatch::Ident(ident) if ident == "as" => {
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
    ) -> ExecutionResult<EvaluationValue>;

    fn evaluate_to_value(
        expression: &Expression<Self>,
        context: &mut Self::EvaluationContext,
    ) -> ExecutionResult<EvaluationValue> {
        ExpressionEvaluator::new(&expression.nodes).evaluate(expression.root, context)
    }
}
