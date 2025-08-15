use syn::token;

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
        ExpressionEvaluator::new(&self.inner.nodes).evaluate(self.inner.root, interpreter)
    }
}

pub(super) enum SourceExpressionLeaf {
    Command(Command),
    Variable(VariableIdentifier),
    Discarded(Token![_]),
    ExpressionBlock(ExpressionBlock),
    Value(ExpressionValue),
}

impl HasSpanRange for SourceExpressionLeaf {
    fn span_range(&self) -> SpanRange {
        match self {
            SourceExpressionLeaf::Command(command) => command.span_range(),
            SourceExpressionLeaf::Variable(variable) => variable.span_range(),
            SourceExpressionLeaf::Discarded(token) => token.span_range(),
            SourceExpressionLeaf::ExpressionBlock(block) => block.span_range(),
            SourceExpressionLeaf::Value(value) => value.span_range(),
        }
    }
}

impl Expressionable for Source {
    type Leaf = SourceExpressionLeaf;
    type EvaluationContext = Interpreter;

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => UnaryAtom::Leaf(Self::Leaf::Command(input.parse()?)),
            SourcePeekMatch::Variable(Grouping::Grouped) => {
                return input.parse_err(
                    "In an expression, the # variable prefix is not allowed. The # prefix should only be used when embedding a variable into an output stream.",
                )
            }
            SourcePeekMatch::Variable(Grouping::Flattened) => {
                return input.parse_err(
                    "In an expression, the #.. variable prefix is not allowed. The # prefix should only be used when embedding a variable into an output sream.",
                )
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
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Object(Braces { delim_span })
            }
            SourcePeekMatch::Group(Delimiter::Bracket) => {
                // This could be handled as parsing a vector of SourceExpressions,
                // but it's more efficient to handle nested vectors as a single expression
                // in the expression parser
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Array(Brackets { delim_span })
            }
            SourcePeekMatch::Punct(punct) => {
                if punct.as_char() == '.' {
                    UnaryAtom::Range(input.parse()?)
                } else {
                    UnaryAtom::PrefixUnaryOperation(input.parse()?)
                }
            }
            SourcePeekMatch::Ident(_) => {
                if input.peek(Token![_]) {
                    return Ok(UnaryAtom::Leaf(Self::Leaf::Discarded(input.parse()?)));
                }
                match input.try_parse_or_revert() {
                    Ok(bool) => UnaryAtom::Leaf(Self::Leaf::Value(ExpressionValue::Boolean(
                        ExpressionBoolean::for_litbool(bool),
                    ))),
                    Err(_) => UnaryAtom::Leaf(Self::Leaf::Variable(input.parse()?)),
                }
            },
            SourcePeekMatch::Literal(_) => {
                let value = ExpressionValue::for_syn_lit(input.parse()?);
                UnaryAtom::Leaf(Self::Leaf::Value(value))
            }
            SourcePeekMatch::End => return input.parse_err("Expected an expression"),
        })
    }

    fn parse_extension(
        input: &mut ParseStreamStack<Self>,
        parent_stack_frame: &ExpressionStackFrame,
    ) -> ParseResult<NodeExtension> {
        // We fall through if we have no match
        match input.peek_grammar() {
            SourcePeekMatch::Group(Delimiter::Bracket) => {
                let (_, delim_span) = input.parse_and_enter_group()?;
                return Ok(NodeExtension::Index(IndexAccess {
                    brackets: Brackets { delim_span },
                }));
            }
            SourcePeekMatch::Punct(punct) if punct.as_char() == ',' => {
                match parent_stack_frame {
                    ExpressionStackFrame::NonEmptyArray { .. }
                    | ExpressionStackFrame::NonEmptyMethodCallParametersList { .. }
                    | ExpressionStackFrame::NonEmptyObject {
                        state: ObjectStackFrameState::EntryValue { .. },
                        ..
                    } => {
                        input.parse::<Token![,]>()?;
                        if input.is_current_empty() {
                            return Ok(NodeExtension::EndOfStreamOrGroup);
                        } else {
                            return Ok(NodeExtension::NonTerminalComma);
                        }
                    }
                    ExpressionStackFrame::Group { .. } => {
                        return input.parse_err("Commas are only permitted inside preinterpret arrays []. Preinterpret arrays [a, b] can be used as a drop-in replacement for rust tuples (a, b).")
                    }
                    // Fall through for an unmatched extension
                    _ => {}
                }
            }
            SourcePeekMatch::Punct(punct) => {
                if punct.as_char() == '.' && input.peek2(syn::Ident) {
                    let dot = input.parse()?;
                    let ident = input.parse()?;
                    if input.peek(token::Paren) {
                        let (_, delim_span) = input.parse_and_enter_group()?;
                        return Ok(NodeExtension::MethodCall(MethodAccess {
                            dot,
                            method: ident,
                            parentheses: Parentheses { delim_span },
                        }));
                    }
                    return Ok(NodeExtension::Property(PropertyAccess {
                        dot,
                        property: ident,
                    }));
                }
                if let Ok(operation) = input.try_parse_or_revert() {
                    return Ok(NodeExtension::CompoundAssignmentOperation(operation));
                }
                if let Ok(operation) = input.try_parse_or_revert() {
                    return Ok(NodeExtension::BinaryOperation(operation));
                }
                if let Ok(range_limits) = input.try_parse_or_revert() {
                    return Ok(NodeExtension::Range(range_limits));
                }
                if let Ok(eq) = input.try_parse_or_revert() {
                    return Ok(NodeExtension::AssignmentOperation(eq));
                }
            }
            SourcePeekMatch::Ident(ident) if ident == "as" => {
                let cast_operation =
                    UnaryOperation::for_cast_operation(input.parse()?, input.parse_any_ident()?)?;
                return Ok(NodeExtension::PostfixOperation(cast_operation));
            }
            SourcePeekMatch::End => return Ok(NodeExtension::EndOfStreamOrGroup),
            _ => {}
        };
        // We are not at the end of the stream, but the tokens which follow are
        // not a valid extension...
        match parent_stack_frame {
            ExpressionStackFrame::Root => Ok(NodeExtension::NoValidExtensionForCurrentParent),
            ExpressionStackFrame::Group { .. } => input.parse_err("Expected ) or operator"),
            ExpressionStackFrame::NonEmptyArray { .. } => input.parse_err("Expected comma, ], or operator"),
            ExpressionStackFrame::IncompleteIndex { .. }
            | ExpressionStackFrame::NonEmptyObject {
                state: ObjectStackFrameState::EntryIndex { .. },
                ..
            } => input.parse_err("Expected ], or operator"),
            ExpressionStackFrame::NonEmptyObject {
                state: ObjectStackFrameState::EntryValue { .. },
                ..
            } => input.parse_err("Expected comma, }, or operator"),
            ExpressionStackFrame::NonEmptyMethodCallParametersList { .. } => {
                input.parse_err("Expected comma, ) or operator")
            }
            // e.g. I've just matched the true in !true or false || true,
            // and I want to see if there's an extension (e.g. a cast).
            // There's nothing matching, so we fall through to an EndOfFrame
            ExpressionStackFrame::IncompleteUnaryPrefixOperation { .. }
            | ExpressionStackFrame::IncompleteBinaryOperation { .. }
            | ExpressionStackFrame::IncompleteRange { .. }
            | ExpressionStackFrame::IncompleteAssignment { .. }
            | ExpressionStackFrame::IncompleteCompoundAssignment { .. } => {
                Ok(NodeExtension::NoValidExtensionForCurrentParent)
            }
        }
    }

    fn evaluate_leaf(
        leaf: &Self::Leaf,
        interpreter: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue> {
        Ok(match leaf {
            SourceExpressionLeaf::Command(command) => {
                command.clone().interpret_to_value(interpreter)?
            }
            SourceExpressionLeaf::Discarded(token) => {
                return token.execution_err("This cannot be used in a value expression");
            }
            SourceExpressionLeaf::Variable(variable_path) => {
                variable_path.interpret_to_value(interpreter)?
            }
            SourceExpressionLeaf::ExpressionBlock(block) => {
                block.interpret_to_value(interpreter)?
            }
            SourceExpressionLeaf::Value(value) => value.clone(),
        })
    }
}

impl Parse<Source> for Expression<Source> {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        ExpressionParser::parse(input)
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct ExpressionNodeId(pub(super) usize);

pub(super) enum ExpressionNode<K: Expressionable> {
    Leaf(K::Leaf),
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

impl ExpressionNode<Source> {
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

pub(super) trait Expressionable: Sized {
    type Leaf;
    type EvaluationContext;

    fn parse_unary_atom(input: &mut ParseStreamStack<Self>) -> ParseResult<UnaryAtom<Self>>;
    fn parse_extension(
        input: &mut ParseStreamStack<Self>,
        parent_stack_frame: &ExpressionStackFrame,
    ) -> ParseResult<NodeExtension>;

    fn evaluate_leaf(
        leaf: &Self::Leaf,
        context: &mut Self::EvaluationContext,
    ) -> ExecutionResult<ExpressionValue>;
}
