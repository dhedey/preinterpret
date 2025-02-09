use super::*;

pub(super) struct ExpressionParser<'a, K: Expressionable> {
    streams: ParseStreamStack<'a, K>,
    nodes: ExpressionNodes<K>,
    expression_stack: Vec<ExpressionStackFrame>,
    kind: PhantomData<K>,
}

impl<'a, K: Expressionable> ExpressionParser<'a, K> {
    pub(super) fn parse(input: ParseStream<'a, K>) -> ParseResult<Expression<K>> {
        Self {
            streams: ParseStreamStack::new(input),
            nodes: ExpressionNodes::new(),
            expression_stack: Vec::with_capacity(10),
            kind: PhantomData,
        }
        .run()
    }

    fn run(mut self) -> ParseResult<Expression<K>> {
        let mut work_item = self.push_stack_frame(ExpressionStackFrame::Root);
        loop {
            work_item = match work_item {
                WorkItem::RequireUnaryAtom => {
                    let unary_atom = K::parse_unary_atom(&mut self.streams)?;
                    self.extend_with_unary_atom(unary_atom)?
                }
                WorkItem::TryParseAndApplyExtension { node } => {
                    let extension = K::parse_extension(&mut self.streams)?;
                    self.attempt_extension(node, extension)?
                }
                WorkItem::TryApplyAlreadyParsedExtension { node, extension } => {
                    self.attempt_extension(node, extension)?
                }
                WorkItem::Finished { root } => {
                    return Ok(self.nodes.complete(root));
                }
            }
        }
    }

    fn extend_with_unary_atom(&mut self, unary_atom: UnaryAtom<K>) -> ParseResult<WorkItem> {
        Ok(match unary_atom {
            UnaryAtom::Leaf(leaf) => self.add_leaf(leaf),
            UnaryAtom::Group(delim_span) => {
                self.push_stack_frame(ExpressionStackFrame::Group { delim_span })
            }
            UnaryAtom::PrefixUnaryOperation(operation) => {
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
                    WorkItem::TryParseAndApplyExtension {
                        node: self.nodes.add_node(ExpressionNode::Grouped {
                            delim_span,
                            inner: node,
                        }),
                    }
                }
                ExpressionStackFrame::IncompletePrefixOperation { operation } => {
                    WorkItem::TryApplyAlreadyParsedExtension {
                        node: self.nodes.add_node(ExpressionNode::UnaryOperation {
                            operation: operation.into(),
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

    fn add_leaf(&mut self, leaf: K::Leaf) -> WorkItem {
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

pub(super) struct ExpressionNodes<K: Expressionable> {
    nodes: Vec<ExpressionNode<K>>,
}

impl<K: Expressionable> ExpressionNodes<K> {
    pub(super) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(super) fn add_node(&mut self, node: ExpressionNode<K>) -> ExpressionNodeId {
        let node_id = ExpressionNodeId(self.nodes.len());
        self.nodes.push(node);
        node_id
    }

    pub(super) fn complete(self, root: ExpressionNodeId) -> Expression<K> {
        Expression {
            root,
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
enum OperatorPrecendence {
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
    const MIN: Self = OperatorPrecendence::Jump;

    fn of_prefix_unary_operation(op: &PrefixUnaryOperation) -> Self {
        match op {
            PrefixUnaryOperation::Neg { .. } | PrefixUnaryOperation::Not { .. } => Self::Prefix,
        }
    }

    fn of_unary_operation(op: &UnaryOperation) -> Self {
        match op {
            UnaryOperation::Cast { .. } => Self::Cast,
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => Self::Prefix,
        }
    }

    fn of_binary_operation(op: &BinaryOperation) -> Self {
        match op {
            BinaryOperation::Integer(op) => Self::of_integer_binary_operator(op),
            BinaryOperation::Paired(op) => Self::of_paired_binary_operator(op),
        }
    }

    fn of_integer_binary_operator(op: &IntegerBinaryOperation) -> Self {
        match op {
            IntegerBinaryOperation::ShiftLeft { .. }
            | IntegerBinaryOperation::ShiftRight { .. } => OperatorPrecendence::Shift,
        }
    }

    fn of_paired_binary_operator(op: &PairedBinaryOperation) -> Self {
        match op {
            PairedBinaryOperation::Addition { .. } | PairedBinaryOperation::Subtraction { .. } => {
                Self::Sum
            }
            PairedBinaryOperation::Multiplication { .. }
            | PairedBinaryOperation::Division { .. }
            | PairedBinaryOperation::Remainder { .. } => Self::Product,
            PairedBinaryOperation::LogicalAnd { .. } => Self::And,
            PairedBinaryOperation::LogicalOr { .. } => Self::Or,
            PairedBinaryOperation::BitXor { .. } => Self::BitXor,
            PairedBinaryOperation::BitAnd { .. } => Self::BitAnd,
            PairedBinaryOperation::BitOr { .. } => Self::BitOr,
            PairedBinaryOperation::Equal { .. }
            | PairedBinaryOperation::LessThan { .. }
            | PairedBinaryOperation::LessThanOrEqual { .. }
            | PairedBinaryOperation::NotEqual { .. }
            | PairedBinaryOperation::GreaterThanOrEqual { .. }
            | PairedBinaryOperation::GreaterThan { .. } => Self::Compare,
        }
    }
}

/// Use of a stack avoids recursion, which hits limits with long/deep expressions.
///
/// ## Worked Algorithm Sketch
///
/// Let ParseStream P be:
/// a b cdx  e fg h i  j k
/// 1 + -(1) + (2 + 4) * 3
///
/// The algorithm proceeds as follows:
/// ```text
/// => Start
/// ===> Set parse stack to have a root parsebuffer of [P]
/// ===> Stack: [Root]
/// ===> WorkItem::RequireUnaryAtom
/// => Read leaf a:1
/// ===> PushEvalNode(A: Leaf(a:1))
/// ===> Stack: [Root]
/// ===> WorkItem::TryParseAndApplyExtension(A) with precedence >MIN from parent=Root
/// => Read binop b:+
/// ===> Stack: [Root, BinOp(A, b:+)]
/// ===> WorkItem::RequireUnaryAtom
/// => Read unop c:-
/// ===> Stack: [Root, BinOp(A, b:+), PrefixOp(c:-)]
/// ===> WorkItem::RequireUnaryAtom
/// => Read group d:([D])
/// ===> PushParseBuffer([D])
/// ===> Stack: [Root, BinOp(A, b:+), PrefixOp(c:-), Group(d:Paren)]
/// ===> WorkItem::RequireUnaryAtom
/// => Read leaf x:1
/// ===> PushEvalNode(X: Leaf(x:1), NewParentPrecedence: Group => >MIN)
/// ===> Stack: [Root, BinOp(A, +), PrefixOp(c:-), Group(d:Paren)]
/// ===> WorkItem::TryParseAndApplyExtension(X) with precedence >MIN from parent=Group
/// => Extension of None is not valid, cascade X with Group:
/// ===> PopStack: It's a group, so PopParseBuffer, PushEvalNode(D: UnOp(d:(), X))
/// ===> Stack: [Root, BinOp(A, b:+), PrefixOp(c:-), TryExtend(D, >PREFIX)]
/// ===> WorkItem::TryParseAndApplyExtension(X) with precedence >PREFIX from parent=PrefixOp(c:-)
/// => Extension of + is not valid, cascade D with PrefixOp:
/// ===> PopStack: PushEvalNode(C: UnOp(c:-, D))
/// ===> Stack: [Root, BinOp(A, b:+)]
/// ===> WorkItem::TryApplyAlreadyParsedExtension(C, +) with precedence >SUM from parent=BinOp(A, b:+)
/// => Extension of + is not valid, so cascade C with BinOp:
/// ===> PopStack: PushEvalNode(B: BinOp(A, b:+, C))
/// ===> Stack: [Root]
/// ===> WorkItem::TryApplyAlreadyParsedExtension(B, +) with precedence >MIN from parent=Root
/// => Read binop e:+
/// ===> Stack: [Root, BinOp(B, e:+)]
/// ===> WorkItem::RequireUnaryAtom
/// => Read group f:([F])
/// ===> PushParseBuffer([F])
/// ===> Stack: [Root, BinOp(B, e:+), Group(f:Paren)]
/// ===> WorkItem::RequireUnaryAtom
/// => Read leaf g:2
/// ===> PushEvalNode(G: Leaf(g:2))
/// ===> Stack: [Root, BinOp(B, e:+), Group(f:Paren)]
/// ===> WorkItem::TryParseAndApplyExtension(G) with precedence >MIN from parent=Group
/// => Read binop h:+
/// ===> Stack: [Root, BinOp(B, e:+), Group(f:Paren), BinOp(G, h:+)]
/// ===> WorkItem::RequireUnaryAtom
/// => Read leaf i:2
/// ===> PushEvalNode(I: Leaf(i:2))
/// ===> Stack: [Root, BinOp(B, e:+), Group(f:Paren), BinOp(G, h:+)]
/// ===> WorkItem::TryParseAndApplyExtension(I) with precedence >SUM from parent=BinOp(G, h:+)
/// => Extension of None is not valid, so cascade I with BinOp:
/// ===> PopStack: PushEvalNode(H: BinOp(G, h:+, I))
/// ===> Stack: [Root, BinOp(B, e:+), Group(f:Paren)]
/// ===> WorkItem::TryApplyAlreadyParsedExtension(H, None) with precedence >MIN from parent=Group
/// => Extension of None is not valid, so cascade H with Group:
/// ===> PopStack: It's a group, so PopParseBuffer, PushEvalNode(F: UnOp(f:Paren, H))
/// ===> Stack: [Root, BinOp(B, e:+)]
/// ===> WorkItem::TryParseAndApplyExtension(F) with precedence >SUM from parent=BinOp(B, e:+)
/// => Read binop j:*
/// ===> Stack: [Root, BinOp(B, e:+), BinOp(F, j:*)]
/// ===> WorkItem::RequireUnaryAtom
/// => Read leaf k:3
/// ===> PushEvalNode(K: Leaf(k:3))
/// ===> Stack: [Root, BinOp(B, e:+), BinOp(F, j:*)]
/// ===> WorkItem::TryParseAndApplyExtension(K) with precedence >PRODUCT from parent=BinOp(F, j:*)
/// => Extension of None is not valid, so cascade K with BinOp:
/// ===> PopStack: PushEvalNode(J: BinOp(F, j:*, K))
/// ===> Stack: [Root, BinOp(B, e:+)]
/// ===> WorkItem::TryApplyAlreadyParsedExtension(J, None) with precedence >SUM from parent=BinOp(B, e:+)
/// => Extension of None is not valid, so cascade J with BinOp:
/// ===> PopStack: PushEvalNode(E: BinOp(B, e:*, J))
/// ===> Stack: [Root]
/// ===> WorkItem::TryApplyAlreadyParsedExtension(E, None) with precedence >MIN from parent=Root
/// => Extension of None is not valid, so cascade E with Root:
/// ===> Stack: []
/// ===> WorkItem::Finished(E)
/// ```
enum ExpressionStackFrame {
    /// A marker for the root of the expression
    Root,
    /// A marker for the parenthesized or transparent group.
    /// * When the group is opened, we add its inside to the parse stream stack
    /// * When the group is closed, we pop it from the parse stream stack
    Group { delim_span: DelimSpan },
    /// An incomplete unary prefix operation
    /// NB: unary postfix operations such as `as` casting go straight to ExtendableNode
    IncompletePrefixOperation { operation: PrefixUnaryOperation },
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
                OperatorPrecendence::of_prefix_unary_operation(operation)
            }
            ExpressionStackFrame::IncompleteBinaryOperation { operation, .. } => {
                OperatorPrecendence::of_binary_operation(operation)
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

pub(super) enum UnaryAtom<K: Expressionable> {
    Leaf(K::Leaf),
    Group(DelimSpan),
    PrefixUnaryOperation(PrefixUnaryOperation),
}

pub(super) enum NodeExtension {
    PostfixOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    NoneMatched,
}

impl NodeExtension {
    fn precedence(&self) -> OperatorPrecendence {
        match self {
            NodeExtension::PostfixOperation(op) => OperatorPrecendence::of_unary_operation(op),
            NodeExtension::BinaryOperation(op) => OperatorPrecendence::of_binary_operation(op),
            NodeExtension::NoneMatched => OperatorPrecendence::MIN,
        }
    }
}
