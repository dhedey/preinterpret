use super::*;

// Source
// =======================

#[derive(Clone)]
pub(crate) struct SourceExpression {
    inner: Expression<Source>,
}

impl Parse<Source> for SourceExpression {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        Ok(Self {
            inner: input.parse()?,
        })
    }
}

impl InterpretToValue for &SourceExpression {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        Source::evaluate(&self.inner, interpreter)
    }
}

pub(super) enum SourceExpressionLeaf {
    Command(Command),
    VariablePath(VariablePath),
    MarkedVariable(MarkedVariable),
    ExpressionBlock(ExpressionBlock),
    Value(ExpressionValue),
}

impl Expressionable for Source {
    type Leaf = SourceExpressionLeaf;
    type EvaluationContext = Interpreter;

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => UnaryAtom::Leaf(Self::Leaf::Command(input.parse()?)),
            SourcePeekMatch::Variable(_) => {
                UnaryAtom::Leaf(Self::Leaf::MarkedVariable(input.parse()?))
            }
            SourcePeekMatch::ExpressionBlock(_) => {
                UnaryAtom::Leaf(Self::Leaf::ExpressionBlock(input.parse()?))
            }
            SourcePeekMatch::AppendVariableBinding => {
                return input
                    .parse_err("Append variable operations are not supported in an expression")
            }
            SourcePeekMatch::ExplicitTransformStream | SourcePeekMatch::Transformer(_) => {
                return input.parse_err("Destructurings are not supported in an expression")
            }
            SourcePeekMatch::Group(Delimiter::None | Delimiter::Parenthesis) => {
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Group(delim_span)
            }
            SourcePeekMatch::Group(Delimiter::Brace) => {
                return input.parse_err("Braces { ... } are not supported in an expression")
            }
            SourcePeekMatch::Group(Delimiter::Bracket) => {
                // This could be handled as parsing a vector of SourceExpressions,
                // but it's more efficient to handle nested vectors as a single expression
                // in the expression parser
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Array {
                    delim_span,
                    is_empty: input.is_empty(),
                }
            }
            SourcePeekMatch::Punct(_) => UnaryAtom::PrefixUnaryOperation(input.parse()?),
            SourcePeekMatch::Ident(_) => match input.try_parse_or_revert() {
                Ok(bool) => UnaryAtom::Leaf(Self::Leaf::Value(ExpressionValue::Boolean(
                    ExpressionBoolean::for_litbool(bool),
                ))),
                Err(_) => UnaryAtom::Leaf(Self::Leaf::VariablePath(input.parse()?)),
            },
            SourcePeekMatch::Literal(_) => {
                let value = ExpressionValue::for_syn_lit(input.parse()?);
                UnaryAtom::Leaf(Self::Leaf::Value(value))
            }
            SourcePeekMatch::End => {
                return input.parse_err("The expression ended in an incomplete state")
            }
        })
    }

    fn parse_extension(input: &mut ParseStreamStack<Self>) -> ParseResult<NodeExtension> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Punct(punct) if punct.as_char() == ',' => {
                NodeExtension::CommaOperator {
                    comma: input.parse()?,
                    is_end_of_stream: input.is_empty(),
                }
            }
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
        Ok(match leaf {
            SourceExpressionLeaf::Command(command) => {
                command.clone().interpret_to_value(interpreter)?
            }
            SourceExpressionLeaf::MarkedVariable(variable) => {
                variable.interpret_to_value(interpreter)?
            }
            SourceExpressionLeaf::VariablePath(variable_path) => {
                variable_path.interpret_to_value(interpreter)?
            }
            SourceExpressionLeaf::ExpressionBlock(block) => {
                block.interpret_to_value(interpreter)?
            }
            SourceExpressionLeaf::Value(value) => value.clone(),
        })
    }
}

// Generic
// =======

pub(super) struct Expression<K: Expressionable> {
    pub(super) root: ExpressionNodeId,
    pub(super) nodes: std::rc::Rc<[ExpressionNode<K>]>,
}

impl<K: Expressionable> Clone for Expression<K> {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            nodes: self.nodes.clone(),
        }
    }
}

impl<K: Expressionable> Parse<K> for Expression<K> {
    fn parse(input: ParseStream<K>) -> ParseResult<Self> {
        ExpressionParser::parse(input)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct ExpressionNodeId(pub(super) usize);

pub(super) enum ExpressionNode<K: Expressionable> {
    Leaf(K::Leaf),
    Grouped {
        delim_span: DelimSpan,
        inner: ExpressionNodeId,
    },
    Array {
        delim_span: DelimSpan,
        items: Vec<ExpressionNodeId>,
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
}

pub(super) trait Expressionable: Sized {
    type Leaf;
    type EvaluationContext;

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>>;
    fn parse_extension(input: &mut ParseStreamStack<Self>) -> ParseResult<NodeExtension>;

    fn evaluate_leaf(
        leaf: &Self::Leaf,
        context: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue>;

    fn evaluate(
        expression: &Expression<Self>,
        context: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue> {
        ExpressionEvaluator::new(&expression.nodes).evaluate(expression.root, context)
    }
}
