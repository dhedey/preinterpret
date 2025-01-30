use super::*;

pub(super) struct ExpressionParser<'a> {
    streams: ParseStreamStack<'a>,
    nodes: ExpressionNodes,
    expression_stack: Vec<ExpressionStackFrame>,
    span_range: SpanRange,
}

impl<'a> ExpressionParser<'a> {
    pub(super) fn new(input: ParseStream<'a>) -> Self {
        Self {
            streams: ParseStreamStack::new(input),
            nodes: ExpressionNodes::new(),
            expression_stack: Vec::with_capacity(10),
            span_range: SpanRange::new_single(input.span()),
        }
    }

    pub(super) fn parse(mut self) -> ParseResult<Expression> {
        let mut work_item = self.push_stack_frame(ExpressionStackFrame::Root);
        loop {
            work_item = match work_item {
                WorkItem::RequireUnaryAtom => {
                    let unary_atom = self.parse_unary_atom()?;
                    self.extend_with_unary_atom(unary_atom)?
                }
                WorkItem::TryParseAndApplyExtension { node } => {
                    let extension = self.parse_extension()?;
                    self.attempt_extension(node, extension)?
                }
                WorkItem::TryApplyAlreadyParsedExtension { node, extension } => {
                    self.attempt_extension(node, extension)?
                }
                WorkItem::Finished { root } => {
                    return Ok(self.nodes.complete(root, self.span_range));
                }
            }
        }
    }

    fn extend_with_unary_atom(&mut self, unary_atom: UnaryAtom) -> ParseResult<WorkItem> {
        Ok(match unary_atom {
            UnaryAtom::Command(command) => {
                self.span_range.set_end(command.span());
                self.add_leaf(ExpressionLeaf::Command(command))
            }
            UnaryAtom::GroupedVariable(variable) => {
                self.span_range.set_end(variable.span());
                self.add_leaf(ExpressionLeaf::GroupedVariable(variable))
            }
            UnaryAtom::CodeBlock(code_block) => {
                self.span_range.set_end(code_block.span());
                self.add_leaf(ExpressionLeaf::CodeBlock(code_block))
            }
            UnaryAtom::Value(value) => {
                if let Some(span) = value.source_span() {
                    self.span_range.set_end(span);
                }
                self.add_leaf(ExpressionLeaf::Value(value))
            }
            UnaryAtom::Group(delim_span) => {
                self.span_range.set_end(delim_span.close());
                self.push_stack_frame(ExpressionStackFrame::Group { delim_span })
            }
            UnaryAtom::UnaryOperation(operation) => {
                self.span_range.set_end(operation.operator.span());
                self.push_stack_frame(ExpressionStackFrame::IncompletePrefixOperation { operation })
            }
        })
    }

    fn attempt_extension(
        &mut self,
        node: ExpressionNodeId,
        extension: NodeExtension,
    ) -> ParseResult<WorkItem> {
        let parent_precedence = self.parent_precedence();
        let extension_precendence = extension.precedence();
        if extension_precendence > parent_precedence {
            // If the extension has a higher precedence than the parent, then the left_node gets pulled into
            // becoming part of the new operation
            Ok(match extension {
                NodeExtension::PostfixOperation(operation) => WorkItem::TryParseAndApplyExtension {
                    node: self.nodes.add_node(ExpressionNode::UnaryOperation {
                        operation,
                        input: node,
                    }),
                },
                NodeExtension::BinaryOperation(operation) => {
                    self.push_stack_frame(ExpressionStackFrame::IncompleteBinaryOperation {
                        lhs: node,
                        operation,
                    })
                }
                NodeExtension::NoneMatched => {
                    panic!("Not possible, as this has minimum precedence")
                }
            })
        } else {
            // Otherwise, the node gets combined into the parent stack frame, and the extension
            // is attempted to be applied against the next parent stack frame
            let parent_stack_frame = self.pop_stack_frame();
            Ok(match parent_stack_frame {
                ExpressionStackFrame::Root => {
                    assert!(matches!(extension, NodeExtension::NoneMatched));
                    WorkItem::Finished { root: node }
                }
                ExpressionStackFrame::Group { delim_span } => {
                    assert!(matches!(extension, NodeExtension::NoneMatched));
                    self.streams.exit_group();
                    let operation = UnaryOperation {
                        operator: UnaryOperator::GroupedNoOp {
                            span: delim_span.join(),
                        },
                    };
                    WorkItem::TryParseAndApplyExtension {
                        node: self.nodes.add_node(ExpressionNode::UnaryOperation {
                            operation,
                            input: node,
                        }),
                    }
                }
                ExpressionStackFrame::IncompletePrefixOperation { operation } => {
                    WorkItem::TryApplyAlreadyParsedExtension {
                        node: self.nodes.add_node(ExpressionNode::UnaryOperation {
                            operation,
                            input: node,
                        }),
                        extension,
                    }
                }
                ExpressionStackFrame::IncompleteBinaryOperation { lhs, operation } => {
                    WorkItem::TryApplyAlreadyParsedExtension {
                        node: self.nodes.add_node(ExpressionNode::BinaryOperation {
                            operation,
                            left_input: lhs,
                            right_input: node,
                        }),
                        extension,
                    }
                }
            })
        }
    }

    fn parse_extension(&mut self) -> ParseResult<NodeExtension> {
        Ok(match self.streams.peek_grammar() {
            PeekMatch::Punct(punct) => match punct.as_char() {
                '.' => NodeExtension::NoneMatched,
                _ => {
                    let operation = BinaryOperation::for_binary_operator(self.streams.parse()?)?;
                    NodeExtension::BinaryOperation(operation)
                }
            },
            PeekMatch::Ident(ident) if ident == "as" => {
                let cast_operation = UnaryOperation::for_cast_operation(self.streams.parse()?, {
                    let target_type = self.streams.parse::<Ident>()?;
                    self.span_range.set_end(target_type.span());
                    target_type
                })?;
                NodeExtension::PostfixOperation(cast_operation)
            }
            _ => NodeExtension::NoneMatched,
        })
    }

    fn parse_unary_atom(&mut self) -> ParseResult<UnaryAtom> {
        Ok(match self.streams.peek_grammar() {
            PeekMatch::GroupedCommand(Some(command_kind)) => {
                match command_kind.grouped_output_kind().expression_support() {
                    Ok(()) => UnaryAtom::Command(self.streams.parse()?),
                    Err(error_message) => return self.streams.parse_err(error_message),
                }
            }
            PeekMatch::GroupedCommand(None) => return self.streams.parse_err("Invalid command"),
            PeekMatch::FlattenedCommand(_) => return self.streams.parse_err(CommandOutputKind::FlattenedStream.expression_support().unwrap_err()),
            PeekMatch::GroupedVariable => UnaryAtom::GroupedVariable(self.streams.parse()?),
            PeekMatch::FlattenedVariable => return self.streams.parse_err("Flattened variables cannot be used directly in expressions. Consider removing the .. or wrapping it inside a command such as [!group! ..] which returns an expression"),
            PeekMatch::AppendVariableDestructuring => return self.streams.parse_err("Append variable operations are not supported in an expression"),
            PeekMatch::Destructurer(_) => return self.streams.parse_err("Destructurings are not supported in an expression"),
            PeekMatch::Group(Delimiter::None | Delimiter::Parenthesis) => {
                let (_, delim_span) = self.streams.parse_and_enter_group()?;
                UnaryAtom::Group(delim_span)
            },
            PeekMatch::Group(Delimiter::Brace) => UnaryAtom::CodeBlock(self.streams.parse()?),
            PeekMatch::Group(Delimiter::Bracket) => return self.streams.parse_err("Square brackets [ .. ] are not supported in an expression"),
            PeekMatch::Punct(_) => {
                let unary_operation = UnaryOperation::for_unary_operator(self.streams.parse()?)?;
                UnaryAtom::UnaryOperation(unary_operation)
            },
            PeekMatch::Ident(_) => {
                UnaryAtom::Value(EvaluationValue::Boolean(EvaluationBoolean::for_litbool(self.streams.parse()?)))
            },
            PeekMatch::Literal(_) => {
                UnaryAtom::Value(EvaluationValue::for_literal(self.streams.parse()?)?)
            },
            PeekMatch::End => return self.streams.parse_err("The expression ended in an incomplete state"),
        })
    }

    fn add_leaf(&mut self, leaf: ExpressionLeaf) -> WorkItem {
        let node = self.nodes.add_node(ExpressionNode::Leaf(leaf));
        WorkItem::TryParseAndApplyExtension { node }
    }

    fn push_stack_frame(&mut self, frame: ExpressionStackFrame) -> WorkItem {
        self.expression_stack.push(frame);
        WorkItem::RequireUnaryAtom
    }

    /// Panics if called after the Root has been popped
    fn pop_stack_frame(&mut self) -> ExpressionStackFrame {
        self.expression_stack
            .pop()
            .expect("There should always be at least a root")
    }

    /// Panics if called after the Root has been popped
    fn parent_precedence(&self) -> OperatorPrecendence {
        self.expression_stack
            .last()
            .map(|s| s.precedence())
            .unwrap()
    }
}

pub(super) struct ExpressionNodes {
    nodes: Vec<ExpressionNode>,
}

impl ExpressionNodes {
    pub(super) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(super) fn add_node(&mut self, node: ExpressionNode) -> ExpressionNodeId {
        let node_id = ExpressionNodeId(self.nodes.len());
        self.nodes.push(node);
        node_id
    }

    pub(super) fn complete(self, root: ExpressionNodeId, span_range: SpanRange) -> Expression {
        Expression {
            root,
            span_range,
            nodes: self.nodes.into(),
        }
    }
}

//===============================================================================================
// The OperatorPrecendence is an amended copy of the Precedence enum from Syn.
//
// Syn is dual-licensed under MIT and Apache, and a subset of it is reproduced from version 2.0.96
// of syn, and then further edited as a derivative work as part of preinterpret, which is released
// under the same licenses.
//
// LICENSE-MIT: https://github.com/dtolnay/syn/blob/2.0.96/LICENSE-MIT
// LICENSE-APACHE: https://github.com/dtolnay/syn/blob/2.0.96/LICENSE-APACHE
//===============================================================================================

/// This is an amended copy of the `Precedence` enum from Syn.
///
/// Reference: https://doc.rust-lang.org/reference/expressions.html#expression-precedence
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(unused)]
pub(crate) enum OperatorPrecendence {
    // return, break, closures
    Jump,
    /// = += -= *= /= %= &= |= ^= <<= >>=
    Assign,
    // .. ..=
    Range,
    // ||
    Or,
    // &&
    And,
    // let
    Let,
    /// == != < > <= >=
    Compare,
    // |
    BitOr,
    // ^
    BitXor,
    // &
    BitAnd,
    // << >>
    Shift,
    // + -
    Sum,
    // * / %
    Product,
    // as
    Cast,
    // unary - * ! & &mut
    Prefix,
    // paths, loops, function calls, array indexing, field expressions, method calls
    Unambiguous,
}

impl OperatorPrecendence {
    pub(crate) const MIN: Self = OperatorPrecendence::Jump;

    pub(crate) fn of_unary_operator(op: &UnaryOperator) -> Self {
        match op {
            UnaryOperator::GroupedNoOp { .. } => Self::Unambiguous,
            UnaryOperator::Cast { .. } => Self::Cast,
            UnaryOperator::Neg { .. } | UnaryOperator::Not { .. } => Self::Prefix,
        }
    }

    pub(crate) fn of_binary_operator(op: &BinaryOperator) -> Self {
        match op {
            BinaryOperator::Integer(op) => Self::of_integer_binary_operator(op),
            BinaryOperator::Paired(op) => Self::of_paired_binary_operator(op),
        }
    }

    fn of_integer_binary_operator(op: &IntegerBinaryOperator) -> Self {
        match op {
            IntegerBinaryOperator::ShiftLeft | IntegerBinaryOperator::ShiftRight => {
                OperatorPrecendence::Shift
            }
        }
    }

    fn of_paired_binary_operator(op: &PairedBinaryOperator) -> Self {
        match op {
            PairedBinaryOperator::Addition | PairedBinaryOperator::Subtraction => Self::Sum,
            PairedBinaryOperator::Multiplication
            | PairedBinaryOperator::Division
            | PairedBinaryOperator::Remainder => Self::Product,
            PairedBinaryOperator::LogicalAnd => Self::And,
            PairedBinaryOperator::LogicalOr => Self::Or,
            PairedBinaryOperator::BitXor => Self::BitXor,
            PairedBinaryOperator::BitAnd => Self::BitAnd,
            PairedBinaryOperator::BitOr => Self::BitOr,
            PairedBinaryOperator::Equal
            | PairedBinaryOperator::LessThan
            | PairedBinaryOperator::LessThanOrEqual
            | PairedBinaryOperator::NotEqual
            | PairedBinaryOperator::GreaterThanOrEqual
            | PairedBinaryOperator::GreaterThan => Self::Compare,
        }
    }
}

/// Use of a stack avoids recursion, which hits limits with long/deep expressions.
///
/// ## Expression Types (in decreasing precedence)
/// * Leaf Expression: Command, Variable, Value (literal, true/false)
/// * Prefix Expression: prefix-based unary operators
/// * Postfix Expression: postfix-based unary operators such as casting
/// * Binary Expression: binary operators
///
/// ## Worked Algorithm Sketch
/// Let ParseStream P be:
/// a b cdx  e fg h i  j k
/// 1 + -(1) + (2 + 4) * 3
///
/// The algorithm proceeds as follows:
/// ```text
/// => Start
/// ===> PushParseBuffer([P])
/// ===> WorkStack: [Root]
/// => Root detects leaf a:1
/// ===> PushEvalNode(A: Leaf(a:1), NewParentPrecedence: Root => >MIN)
/// ===> WorkStack: [Root, TryExtend(A, >MIN)]
/// => TryExtend detects binop b:+
/// ===> WorkStack: [Root, BinOp(A, b:+)]
/// => BinOp detects unop c:-
/// ===> WorkStack: [Root, BinOp(A, b:+), PrefixOp(c:-)]
/// => PrefixOp detects group d:([D])
/// ===> PushParseBuffer([D])
/// ===> WorkStack: [Root, BinOp(A, b:+), PrefixOp(c:-), Group(d:Paren), EmptyExpression]
/// => EmptyExpression detects leaf x:1
/// ===> PushEvalNode(X: Leaf(x:1), NewParentPrecedence: Group => >MIN)
/// ===> WorkStack: [Root, BinOp(A, +), PrefixOp(c:-), Group(d:Paren), TryExtend(X, >MIN)]
/// => TryExtend detects no valid extension, cascade X with Group:
/// ===> PopWorkStack: It's a group, so PopParseBuffer, PushEvalNode(D: UnOp(d:(), X), NewParentPrecedence: PrefixOp => >PREFIX)
/// ===> WorkStack: [Root, BinOp(A, b:+), PrefixOp(c:-), TryExtend(D, >PREFIX)]
/// => TryExtend detects no valid extension (it's impossible), cascade D with PrefixOp:
/// ===> PopWorkStack: PushEvalNode(C: UnOp(c:-, D), NewParentPrecedence: BinOp => >SUM)
/// ===> WorkStack: [Root, BinOp(A, b:+), TryExtend(C, >SUM)]
/// => TryExtend detects no valid extension, so cascade C with BinOp:
/// ===> PopWorkStack: PushEvalNode(B: BinOp(A, b:+, C), NewParentPrecedence: EMPTY => >MIN)
/// ===> WorkStack: [Root, TryExtend(B, >MIN)]
/// => TryExtend detects binop e:+
/// ===> WorkStack: [Root, BinOp(B, e:+)]
/// => BinOp detects group f:([F])
/// ===> PushParseBuffer([F])
/// ===> WorkStack: [Root, BinOp(B, e:+), Group(f:Paren), EmptyExpression]
/// => EmptyExpression detects leaf g:2
/// ===> PushEvalNode(G: Leaf(g:2), NewParentPrecedence: Group => >MIN)
/// ===> WorkStack: [Root, BinOp(B, e:+), Group(f:Paren), TryExtend(G, >MIN)]
/// => TryExtend detects binop h:+
/// ===> WorkStack: [Root, BinOp(B, e:+), Group(f:Paren), BinOp(G, h:+)]
/// => BinOp detects leaf i:2
/// ===> PushEvalNode(I: Leaf(i:2), NewParentPrecedence: BinOp => >SUM)
/// ===> WorkStack: [Root, BinOp(B, e:+), Group(f:Paren), BinOp(G, h:+), TryExtend(I, >SUM)]
/// => TryExtend detects no valid extension, so cascade I with BinOp:
/// ===> PopWorkStack: PushEvalNode(H: BinOp(G, h:+, I), NewParentPrecedence: Group => >MIN)
/// ===> WorkStack: [Root, BinOp(B, e:+), Group(f:Paren), TryExtend(H, >MIN)]
/// => TryExtend detects no valid extension, so cascade H with Group:
/// ===> PopWorkStack: It's a group, so PopParseBuffer, PushEvalNode(F: UnOp(f:Paren, H), NewParentPrecedence: BinOp => >SUM)
/// ===> WorkStack: [Root, BinOp(B, e:+), TryExtend(F, >SUM)]
/// => TryExtend detects binop j:*
/// ===> WorkStack: [Root, BinOp(B, e:+), BinOp(F, j:*)]
/// => BinOp detects leaf k:3
/// ===> PushEvalNode(K: Leaf(k:3), NewParentPrecedence: BinOp => >PRODUCT)
/// ===> WorkStack: [Root, BinOp(B, e:+), BinOp(F, j:*), TryExtend(K, >PRODUCT)]
/// => TryExtend detects no valid extension, so cascade K with BinOp:
/// ===> PopWorkStack: PushEvalNode(J: BinOp(F, j:*, K), NewParentPrecedence: BinOp => >SUM)
/// ===> WorkStack: [Root, BinOp(B, e:+), TryExtend(J, >SUM)]
/// => TryExtend detects no valid extension, so cascade J with BinOp:
/// ===> PopWorkStack: PushEvalNode(E: BinOp(B, e:*, J), NewParentPrecedence: Root => >MIN)
/// ===> WorkStack: [Root, TryExtend(E, >MIN)]
/// => TryExtend detects no valid extension, so cascade E with Root:
/// ===> DONE Root = E
/// ```
///
/// TODO SPECIAL CASES:
/// * Some operators can't follow others, e.g. comparison operators can't be chained
enum ExpressionStackFrame {
    /// A marker for the root of the expression
    Root,
    /// A marker for the parenthesized or transparent group.
    /// * When the group is opened, we add its inside to the parse stream stack
    /// * When the group is closed, we pop it from the parse stream stack
    Group { delim_span: DelimSpan },
    /// An incomplete unary prefix operation
    /// NB: unary postfix operations such as `as` casting go straight to ExtendableNode
    IncompletePrefixOperation { operation: UnaryOperation },
    /// An incomplete binary operation
    IncompleteBinaryOperation {
        lhs: ExpressionNodeId,
        operation: BinaryOperation,
    },
}

impl ExpressionStackFrame {
    fn precedence(&self) -> OperatorPrecendence {
        match self {
            ExpressionStackFrame::Root => OperatorPrecendence::MIN,
            ExpressionStackFrame::Group { .. } => OperatorPrecendence::MIN,
            ExpressionStackFrame::IncompletePrefixOperation { operation, .. } => {
                OperatorPrecendence::of_unary_operator(&operation.operator)
            }
            ExpressionStackFrame::IncompleteBinaryOperation { operation, .. } => {
                OperatorPrecendence::of_binary_operator(&operation.operator)
            }
        }
    }
}

enum WorkItem {
    /// We require reading a UnaryAtom, such as a command, variable, literal, or unary operation
    RequireUnaryAtom,
    /// We have a partial expression, which can possibly be extended to the right.
    /// => If on the right, there is an operation of higher precedence than its parent,
    ///    it gets pulled into being part of the right expression
    /// => Otherwise it will be combined into its parent
    TryParseAndApplyExtension {
        node: ExpressionNodeId,
    },
    TryApplyAlreadyParsedExtension {
        node: ExpressionNodeId,
        extension: NodeExtension,
    },
    Finished {
        root: ExpressionNodeId,
    },
}

enum UnaryAtom {
    Command(Command),
    GroupedVariable(GroupedVariable),
    CodeBlock(CommandCodeInput),
    Value(EvaluationValue),
    Group(DelimSpan),
    UnaryOperation(UnaryOperation),
}

enum NodeExtension {
    PostfixOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    NoneMatched,
}

impl NodeExtension {
    fn precedence(&self) -> OperatorPrecendence {
        match self {
            NodeExtension::PostfixOperation(op) => {
                OperatorPrecendence::of_unary_operator(&op.operator)
            }
            NodeExtension::BinaryOperation(op) => {
                OperatorPrecendence::of_binary_operator(&op.operator)
            }
            NodeExtension::NoneMatched => OperatorPrecendence::MIN,
        }
    }
}
