use super::*;

/// ## Overview
///
/// Expression parsing is quite complicated, but this is possibly slightly more complicated in some ways than it needs to be.
/// Its design is intended to make expression parsing more intuitive (although I'm not sure that's been achieved really),
/// and to avoid recursion by making use of an explicit stack frame approach with [`ExpressionStackFrame`]s.
///
/// This allows parsing of very long expressions (e.g. a sum of 1 million terms) without hitting recursion limits.
///
/// ## Intuition
///
/// You can think of these frames in two ways - as:
/// (a) the local variables of a function in an un-flattened parser call stack
/// (b) the specifics of a parent operator, which can allow judging if/when an introduced expression with a possible
///     extension should either bind to its parent (ignoring the extension for now, and retrying the extension with
///     its parent) or bind to the extension itself.
///
/// ## Examples
///
/// See the rust doc on the [`ExpressionStackFrame`] for further details.
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
                    let extension = K::parse_extension(
                        &mut self.streams,
                        self.expression_stack.last().unwrap(),
                    )?;
                    self.attempt_extension(node, extension)?
                }
                WorkItem::TryApplyAlreadyParsedExtension { node, extension } => {
                    self.attempt_extension(node, extension)?
                }
                WorkItem::ContinueRange { lhs, range_limits } => {
                    self.continue_range(lhs, range_limits)?
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
            UnaryAtom::Array {
                delim_span,
                is_empty: true,
            } => {
                self.streams.exit_group();
                WorkItem::TryParseAndApplyExtension {
                    node: self.nodes.add_node(ExpressionNode::Array {
                        delim_span,
                        items: Vec::new(),
                    }),
                }
            }
            UnaryAtom::Array {
                delim_span,
                is_empty: false,
            } => self.push_stack_frame(ExpressionStackFrame::Array {
                delim_span,
                items: Vec::new(),
            }),
            UnaryAtom::PrefixUnaryOperation(operation) => {
                self.push_stack_frame(ExpressionStackFrame::IncompleteUnaryPrefixOperation {
                    operation,
                })
            }
            UnaryAtom::Range(range_limits) => WorkItem::ContinueRange {
                lhs: None,
                range_limits,
            },
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
                NodeExtension::NonTerminalArrayComma => {
                    match self.expression_stack.last_mut().unwrap() {
                        ExpressionStackFrame::Array { items, .. } => {
                            items.push(node);
                        }
                        _ => unreachable!("CommaOperator is only returned under an Array parent."),
                    }
                    WorkItem::RequireUnaryAtom
                }
                NodeExtension::Property(access) => WorkItem::TryParseAndApplyExtension {
                    node: self
                        .nodes
                        .add_node(ExpressionNode::Property { node, access }),
                },
                NodeExtension::Index(access) => {
                    self.push_stack_frame(ExpressionStackFrame::IncompleteIndex { node, access })
                }
                NodeExtension::Range(range_limits) => WorkItem::ContinueRange {
                    lhs: Some(node),
                    range_limits,
                },
                NodeExtension::AssignmentOperation(equals_token) => {
                    self.push_stack_frame(ExpressionStackFrame::IncompleteAssignment {
                        assignee: node,
                        equals_token,
                    })
                }
                NodeExtension::CompoundAssignmentOperation(operation) => {
                    self.push_stack_frame(ExpressionStackFrame::IncompleteCompoundAssignment {
                        place: node,
                        operation,
                    })
                }
                NodeExtension::EndOfStream | NodeExtension::NoValidExtensionForCurrentParent => {
                    unreachable!("Not possible, as these have minimum precedence")
                }
            })
        } else {
            // Otherwise, the node gets combined into the parent stack frame, and the extension
            // is attempted to be applied against the next parent stack frame
            let parent_stack_frame = self.pop_stack_frame();
            Ok(match parent_stack_frame {
                ExpressionStackFrame::Root => {
                    assert!(matches!(
                        extension,
                        NodeExtension::EndOfStream
                            | NodeExtension::NoValidExtensionForCurrentParent
                    ));
                    WorkItem::Finished { root: node }
                }
                ExpressionStackFrame::Group { delim_span } => {
                    assert!(matches!(extension, NodeExtension::EndOfStream));
                    self.streams.exit_group();
                    WorkItem::TryParseAndApplyExtension {
                        node: self.nodes.add_node(ExpressionNode::Grouped {
                            delim_span,
                            inner: node,
                        }),
                    }
                }
                ExpressionStackFrame::Array {
                    mut items,
                    delim_span,
                } => {
                    assert!(matches!(extension, NodeExtension::EndOfStream));
                    items.push(node);
                    self.streams.exit_group();
                    WorkItem::TryParseAndApplyExtension {
                        node: self
                            .nodes
                            .add_node(ExpressionNode::Array { delim_span, items }),
                    }
                }
                ExpressionStackFrame::IncompleteUnaryPrefixOperation { operation } => {
                    let node = self.nodes.add_node(ExpressionNode::UnaryOperation {
                        operation: operation.into(),
                        input: node,
                    });
                    extension.into_post_operation_completion_work_item(node)
                }
                ExpressionStackFrame::IncompleteBinaryOperation { lhs, operation } => {
                    let node = self.nodes.add_node(ExpressionNode::BinaryOperation {
                        operation,
                        left_input: lhs,
                        right_input: node,
                    });
                    extension.into_post_operation_completion_work_item(node)
                }
                ExpressionStackFrame::IncompleteIndex {
                    node: source,
                    access,
                } => {
                    assert!(matches!(extension, NodeExtension::EndOfStream));
                    self.streams.exit_group();
                    let node = self.nodes.add_node(ExpressionNode::Index {
                        node: source,
                        access,
                        index: node,
                    });
                    WorkItem::TryParseAndApplyExtension { node }
                }
                ExpressionStackFrame::IncompleteRange { lhs, range_limits } => {
                    let node = self.nodes.add_node(ExpressionNode::Range {
                        left: lhs,
                        range_limits,
                        right: Some(node),
                    });
                    extension.into_post_operation_completion_work_item(node)
                }
                ExpressionStackFrame::IncompleteAssignment {
                    assignee,
                    equals_token,
                } => {
                    let node = self.nodes.add_node(ExpressionNode::Assignment {
                        assignee,
                        equals_token,
                        value: node,
                    });
                    extension.into_post_operation_completion_work_item(node)
                }
                ExpressionStackFrame::IncompleteCompoundAssignment { place, operation } => {
                    let node = self.nodes.add_node(ExpressionNode::CompoundAssignment {
                        place,
                        operation,
                        value: node,
                    });
                    extension.into_post_operation_completion_work_item(node)
                }
            })
        }
    }

    fn continue_range(
        &mut self,
        lhs: Option<ExpressionNodeId>,
        range_limits: syn::RangeLimits,
    ) -> ParseResult<WorkItem> {
        // See https://doc.rust-lang.org/reference/expressions/range-expr.html
        match &range_limits {
            syn::RangeLimits::Closed(_) => {
                // A closed range requires a right hand side, so we can just
                // go straight to matching a UnaryAtom for it
                return Ok(
                    self.push_stack_frame(ExpressionStackFrame::IncompleteRange {
                        lhs,
                        range_limits,
                    }),
                );
            }
            syn::RangeLimits::HalfOpen(_) => {}
        }
        // Otherwise, we have a half-open range, and need to work out whether
        // we can parse a UnaryAtom to be the right side of the range or whether
        // it will have no right side.
        // Some examples of such ranges include: `[3.., 4]`, `[3..]`, `let x = 3..;`,
        // `(3..).first()` or even `3...first()`
        let can_parse_unary_atom = {
            let forked = self.streams.fork_current();
            let mut forked_stack = ParseStreamStack::new(&forked);
            K::parse_unary_atom(&mut forked_stack).is_ok()
        };
        if can_parse_unary_atom {
            // A unary atom can be parsed so let's attempt to complete the range with it
            Ok(self.push_stack_frame(ExpressionStackFrame::IncompleteRange { lhs, range_limits }))
        } else {
            // No unary atom can be parsed, so let's complete the range
            let node = self.nodes.add_node(ExpressionNode::Range {
                left: lhs,
                range_limits,
                right: None,
            });
            Ok(WorkItem::TryParseAndApplyExtension { node })
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
            .map(|s| s.precedence_to_bind_to_child())
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
    // [PREINTERPRET ADDITION]
    // Assignment should bind more tightly than arrays.
    // e.g. [x = 3, 2] should parse as [(x = 3), 2] rather than [x = (3, 2)]
    NonTerminalComma,
    /// = += -= *= /= %= &= |= ^= <<= >>=
    Assign,
    // [PREINTERPRET ADDITION]
    // By giving assign extensions slightly higher priority than existing assign
    // frames, this effectively makes them right-associative, so that:
    // a = b = c parses as a = (b = c) instead of (a = b) = c
    AssignExtension,
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
pub(super) enum ExpressionStackFrame {
    /// A marker for the root of the expression
    Root,
    /// A marker for the parenthesized or transparent group.
    /// * When the group is opened, we add its inside to the parse stream stack
    /// * When the group is closed, we pop it from the parse stream stack
    Group { delim_span: DelimSpan },
    /// A marker for the bracketed array.
    /// * When the array is opened, we add its inside to the parse stream stack
    /// * When the array is closed, we pop it from the parse stream stack
    Array {
        delim_span: DelimSpan,
        items: Vec<ExpressionNodeId>,
    },
    /// An incomplete unary prefix operation
    /// NB: unary postfix operations such as `as` casting go straight to ExtendableNode
    IncompleteUnaryPrefixOperation { operation: PrefixUnaryOperation },
    /// An incomplete binary operation
    IncompleteBinaryOperation {
        lhs: ExpressionNodeId,
        operation: BinaryOperation,
    },
    /// An incomplete indexing access
    IncompleteIndex {
        node: ExpressionNodeId,
        access: IndexAccess,
    },
    /// An incomplete assignment operation
    /// It's left side is an assignee expression, according to the [rust reference].
    ///
    /// [rust reference]: https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
    IncompleteAssignment {
        assignee: ExpressionNodeId,
        equals_token: Token![=],
    },
    /// An incomplete assignment operation
    /// It's left side is a place expression, according to the [rust reference].
    ///
    /// [rust reference]: https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
    IncompleteCompoundAssignment {
        place: ExpressionNodeId,
        operation: CompoundAssignmentOperation,
    },
    /// A range which will be followed by a rhs
    IncompleteRange {
        lhs: Option<ExpressionNodeId>,
        range_limits: syn::RangeLimits,
    },
}

impl ExpressionStackFrame {
    fn precedence_to_bind_to_child(&self) -> OperatorPrecendence {
        match self {
            ExpressionStackFrame::Root => OperatorPrecendence::MIN,
            ExpressionStackFrame::Group { .. } => OperatorPrecendence::MIN,
            ExpressionStackFrame::Array { .. } => OperatorPrecendence::MIN,
            ExpressionStackFrame::IncompleteIndex { .. } => OperatorPrecendence::MIN,
            ExpressionStackFrame::IncompleteRange { .. } => OperatorPrecendence::Range,
            ExpressionStackFrame::IncompleteAssignment { .. } => OperatorPrecendence::Assign,
            ExpressionStackFrame::IncompleteCompoundAssignment { .. } => {
                OperatorPrecendence::Assign
            }
            ExpressionStackFrame::IncompleteUnaryPrefixOperation { operation, .. } => {
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
    /// The same as [`WorkItem::TryParseAndApplyExtension`], except that we have already
    /// parsed the extension, and we are attempting to apply it again.
    /// This happens if an extension is of lower precedence than an existing unary/binary
    /// operation, and so we complete the existing operation and try to re-apply the extension
    /// on the result. For example, the + in `2 * 3 + 4` fails to apply to 3, but can
    /// then apply to (2 * 3).
    TryApplyAlreadyParsedExtension {
        node: ExpressionNodeId,
        extension: NodeExtension,
    },
    /// Range parsing behaviour is quite unique, so we have a special case for it.
    ContinueRange {
        lhs: Option<ExpressionNodeId>,
        range_limits: syn::RangeLimits,
    },
    Finished {
        root: ExpressionNodeId,
    },
}

pub(super) enum UnaryAtom<K: Expressionable> {
    Leaf(K::Leaf),
    Group(DelimSpan),
    Array {
        delim_span: DelimSpan,
        is_empty: bool,
    },
    PrefixUnaryOperation(PrefixUnaryOperation),
    Range(syn::RangeLimits),
}

pub(super) enum NodeExtension {
    PostfixOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    NonTerminalArrayComma,
    Property(PropertyAccess),
    Index(IndexAccess),
    Range(syn::RangeLimits),
    AssignmentOperation(Token![=]),
    CompoundAssignmentOperation(CompoundAssignmentOperation),
    EndOfStream,
    NoValidExtensionForCurrentParent,
}

impl NodeExtension {
    fn precedence(&self) -> OperatorPrecendence {
        match self {
            NodeExtension::PostfixOperation(op) => OperatorPrecendence::of_unary_operation(op),
            NodeExtension::BinaryOperation(op) => OperatorPrecendence::of_binary_operation(op),
            NodeExtension::NonTerminalArrayComma => OperatorPrecendence::NonTerminalComma,
            NodeExtension::Property { .. } => OperatorPrecendence::Unambiguous,
            NodeExtension::Index { .. } => OperatorPrecendence::Unambiguous,
            NodeExtension::Range(_) => OperatorPrecendence::Range,
            NodeExtension::EndOfStream => OperatorPrecendence::MIN,
            NodeExtension::AssignmentOperation(_) => OperatorPrecendence::AssignExtension,
            NodeExtension::CompoundAssignmentOperation(_) => OperatorPrecendence::AssignExtension,
            NodeExtension::NoValidExtensionForCurrentParent => OperatorPrecendence::MIN,
        }
    }

    fn into_post_operation_completion_work_item(self, node: ExpressionNodeId) -> WorkItem {
        match self {
            // These extensions are valid/correct for any parent,
            // so can be re-used without parsing again.
            extension @ (NodeExtension::PostfixOperation { .. }
            | NodeExtension::BinaryOperation { .. }
            | NodeExtension::Property { .. }
            | NodeExtension::Index { .. }
            | NodeExtension::Range { .. }
            | NodeExtension::AssignmentOperation { .. }
            | NodeExtension::CompoundAssignmentOperation { .. }
            | NodeExtension::EndOfStream) => {
                WorkItem::TryApplyAlreadyParsedExtension { node, extension }
            }
            NodeExtension::NonTerminalArrayComma => {
                unreachable!("Array comma is only possible on array parent")
            }
            NodeExtension::NoValidExtensionForCurrentParent => {
                // We have to reparse in case the extension is valid for the new parent.
                // e.g. consider [2 + 3, 4] - initially the peeked comma has a parent of an
                // incomplete binary operation 2 + 3 which is invalid.
                // When the binary operation is completed, the comma now gets parsed against
                // the parent array, which can succeed.
                WorkItem::TryParseAndApplyExtension { node }
            }
        }
    }
}
