use syn::token;

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
pub(super) struct ExpressionParser<'a> {
    streams: ParseStreamStack<'a, Source>,
    nodes: ExpressionNodes,
    expression_stack: Vec<ExpressionStackFrame>,
}

impl<'a> ExpressionParser<'a> {
    pub(super) fn parse(input: ParseStream<'a, Source>) -> ParseResult<Expression> {
        Self {
            streams: ParseStreamStack::new(input),
            nodes: ExpressionNodes::new(),
            expression_stack: Vec::with_capacity(10),
        }
        .run()
    }

    fn run(mut self) -> ParseResult<Expression> {
        let mut work_item = self.push_stack_frame(ExpressionStackFrame::Root);
        loop {
            work_item = match work_item {
                WorkItem::RequireUnaryAtom => {
                    let unary_atom = Self::parse_unary_atom(&mut self.streams)?;
                    self.extend_with_unary_atom(unary_atom)?
                }
                WorkItem::TryParseAndApplyExtension { node } => {
                    let extension = Self::parse_extension(
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

    fn parse_unary_atom(input: &mut ParseStreamStack<Source>) -> ParseResult<UnaryAtom> {
        Ok(match input.peek_grammar() {
            SourcePeekMatch::Command(_) => UnaryAtom::Leaf(Leaf::Command(input.parse()?)),
            SourcePeekMatch::EmbeddedVariable | SourcePeekMatch::EmbeddedExpression => {
                return input.parse_err(
                    "In an expression, the # variable prefix is not allowed. The # prefix should only be used when embedding a variable into an output stream, e.g. %[#var + #(..expressions..)]",
                )
            }
            SourcePeekMatch::ExplicitTransformStream | SourcePeekMatch::Transformer(_) => {
                return input.parse_err("Destructurings are not supported in an expression")
            }
            SourcePeekMatch::Group(Delimiter::None | Delimiter::Parenthesis) => {
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Group(delim_span)
            }
            SourcePeekMatch::Group(Delimiter::Brace) => {
                let (inner, _, delim_span, _) = input.cursor().any_group().unwrap();
                if let Some((_, next)) = inner.ident() {
                    if next.punct_matching(':').is_some() || next.punct_matching(',').is_some() {
                        return delim_span.open().parse_err("An object literal must be prefixed with %, e.g. `%{ field: 1 }`. Without such a prefix, { .. } defines a block.");
                    }
                }
                UnaryAtom::Leaf(Leaf::Block(input.parse()?))
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
            SourcePeekMatch::Ident(ident) => {
                match ident.to_string().as_str() {
                    "_" => UnaryAtom::Leaf(Leaf::Discarded(input.parse()?)),
                    "true" | "false" => {
                        let bool = input.parse::<syn::LitBool>()?;
                        UnaryAtom::Leaf(Leaf::Value(SharedValue::new_from_owned(
                            ExpressionBoolean::for_litbool(&bool).into_owned_value(),
                        )))
                    }
                    "if" => UnaryAtom::Leaf(Leaf::IfExpression(input.parse()?)),
                    "loop" => UnaryAtom::Leaf(Leaf::LoopExpression(input.parse()?)),
                    "while" => UnaryAtom::Leaf(Leaf::WhileExpression(input.parse()?)),
                    "for" => UnaryAtom::Leaf(Leaf::ForExpression(input.parse()?)),
                    _ => UnaryAtom::Leaf(Leaf::Variable(input.parse()?))
                }
            },
            SourcePeekMatch::Literal(_) => {
                let value = ExpressionValue::for_syn_lit(input.parse()?);
                UnaryAtom::Leaf(Leaf::Value(SharedValue::new_from_owned(value)))
            },
            SourcePeekMatch::StreamLiteral(_) => {
                UnaryAtom::Leaf(Leaf::StreamLiteral(input.parse()?))
            }
            SourcePeekMatch::ObjectLiteral => {
                let _: Token![%] = input.parse()?;
                let (_, delim_span) = input.parse_and_enter_group()?;
                UnaryAtom::Object(Braces { delim_span })
            }
            SourcePeekMatch::End => return input.parse_err("Expected an expression"),
        })
    }

    fn parse_extension(
        input: &mut ParseStreamStack<Source>,
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
            ExpressionStackFrame::NonEmptyArray { .. } => {
                input.parse_err("Expected comma, ], or operator")
            }
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

    fn extend_with_unary_atom(&mut self, unary_atom: UnaryAtom) -> ParseResult<WorkItem> {
        Ok(match unary_atom {
            UnaryAtom::Leaf(leaf) => self.add_leaf(leaf),
            UnaryAtom::Group(delim_span) => {
                self.push_stack_frame(ExpressionStackFrame::Group { delim_span })
            }
            UnaryAtom::Array(brackets) => {
                if self.streams.is_current_empty() {
                    self.streams.exit_group();
                    WorkItem::TryParseAndApplyExtension {
                        node: self.nodes.add_node(ExpressionNode::Array {
                            brackets,
                            items: Vec::new(),
                        }),
                    }
                } else {
                    self.push_stack_frame(ExpressionStackFrame::NonEmptyArray {
                        brackets,
                        items: Vec::new(),
                    })
                }
            }
            UnaryAtom::Object(braces) => self.continue_object(braces, Vec::new())?,
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
                NodeExtension::NonTerminalComma => {
                    let next_item = match self.expression_stack.last_mut().unwrap() {
                        ExpressionStackFrame::NonEmptyMethodCallParametersList { parameters, .. } => {
                            parameters.push(node);
                            Some(WorkItem::RequireUnaryAtom)
                        }
                        ExpressionStackFrame::NonEmptyArray { items, .. } => {
                            items.push(node);
                            Some(WorkItem::RequireUnaryAtom)
                        }
                        ExpressionStackFrame::NonEmptyObject { .. } => {
                            None // Placeholder to indicate separate handling below
                        }
                        _ => unreachable!(
                            "NonTerminalComma is only returned under an Array, Object or MethodCallParametersList parent."
                        ),
                    };
                    match next_item {
                        Some(item) => item,
                        None => {
                            // Indicates object
                            match self.expression_stack.pop().unwrap() {
                                ExpressionStackFrame::NonEmptyObject {
                                    braces,
                                    state: ObjectStackFrameState::EntryValue(key, _),
                                    mut complete_entries,
                                } => {
                                    complete_entries.push((key, node));
                                    self.continue_object(braces, complete_entries)?
                                }
                                _ => unreachable!(
                                    "This None code path is only reachable under an object"
                                ),
                            }
                        }
                    }
                }
                NodeExtension::Property(access) => WorkItem::TryParseAndApplyExtension {
                    node: self
                        .nodes
                        .add_node(ExpressionNode::Property { node, access }),
                },
                NodeExtension::MethodCall(method) => {
                    if self.streams.is_current_empty() {
                        self.streams.exit_group();
                        let node = self.nodes.add_node(ExpressionNode::MethodCall {
                            node,
                            method,
                            parameters: Vec::new(),
                        });
                        WorkItem::TryParseAndApplyExtension { node }
                    } else {
                        self.push_stack_frame(
                            ExpressionStackFrame::NonEmptyMethodCallParametersList {
                                node,
                                method,
                                parameters: Vec::new(),
                            },
                        )
                    }
                }
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
                NodeExtension::EndOfStreamOrGroup
                | NodeExtension::NoValidExtensionForCurrentParent => {
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
                        NodeExtension::EndOfStreamOrGroup
                            | NodeExtension::NoValidExtensionForCurrentParent
                    ));
                    WorkItem::Finished { root: node }
                }
                ExpressionStackFrame::Group { delim_span } => {
                    assert!(matches!(extension, NodeExtension::EndOfStreamOrGroup));
                    self.streams.exit_group();
                    WorkItem::TryParseAndApplyExtension {
                        node: self.nodes.add_node(ExpressionNode::Grouped {
                            delim_span,
                            inner: node,
                        }),
                    }
                }
                ExpressionStackFrame::NonEmptyArray {
                    mut items,
                    brackets,
                } => {
                    assert!(matches!(extension, NodeExtension::EndOfStreamOrGroup));
                    items.push(node);
                    self.streams.exit_group();
                    WorkItem::TryParseAndApplyExtension {
                        node: self
                            .nodes
                            .add_node(ExpressionNode::Array { brackets, items }),
                    }
                }
                ExpressionStackFrame::NonEmptyMethodCallParametersList {
                    node: source,
                    mut parameters,
                    method,
                } => {
                    assert!(matches!(extension, NodeExtension::EndOfStreamOrGroup));
                    parameters.push(node);
                    self.streams.exit_group();
                    let node = self.nodes.add_node(ExpressionNode::MethodCall {
                        node: source,
                        method,
                        parameters,
                    });
                    WorkItem::TryParseAndApplyExtension { node }
                }
                ExpressionStackFrame::NonEmptyObject {
                    braces,
                    complete_entries,
                    state: ObjectStackFrameState::EntryIndex(access),
                } => {
                    assert!(matches!(extension, NodeExtension::EndOfStreamOrGroup));
                    self.streams.exit_group();
                    let colon = self.streams.parse()?;
                    self.expression_stack
                        .push(ExpressionStackFrame::NonEmptyObject {
                            braces,
                            complete_entries,
                            state: ObjectStackFrameState::EntryValue(
                                ObjectKey::Indexed {
                                    access,
                                    index: node,
                                },
                                colon,
                            ),
                        });
                    WorkItem::RequireUnaryAtom
                }
                ExpressionStackFrame::NonEmptyObject {
                    braces,
                    complete_entries: mut entries,
                    state: ObjectStackFrameState::EntryValue(key, _),
                } => {
                    assert!(matches!(extension, NodeExtension::EndOfStreamOrGroup));
                    self.streams.exit_group();
                    entries.push((key, node));
                    let node = self
                        .nodes
                        .add_node(ExpressionNode::Object { braces, entries });
                    WorkItem::TryParseAndApplyExtension { node }
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
                    assert!(matches!(extension, NodeExtension::EndOfStreamOrGroup));
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
            Self::parse_unary_atom(&mut forked_stack).is_ok()
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

    fn continue_object(
        &mut self,
        braces: Braces,
        mut complete_entries: Vec<(ObjectKey, ExpressionNodeId)>,
    ) -> ParseResult<WorkItem> {
        const ERROR_MESSAGE: &str = r##"Expected an object entry (`field,` `field: ..,` or `["field"]: ..,`). If you meant to start a new block, use #{ ... } instead."##;
        let state = loop {
            if self.streams.is_current_empty() {
                self.streams.exit_group();
                let node = self.nodes.add_node(ExpressionNode::Object {
                    braces,
                    entries: complete_entries,
                });
                return Ok(WorkItem::TryParseAndApplyExtension { node });
            } else if self.streams.peek(syn::Ident) {
                let key: Ident = self.streams.parse()?;

                if self.streams.is_current_empty() {
                    // Fall through
                } else if self.streams.peek(token::Comma) {
                    self.streams.parse::<Token![,]>()?;
                    // Fall through
                } else if self.streams.peek(token::Colon) {
                    let colon = self.streams.parse()?;
                    break ObjectStackFrameState::EntryValue(ObjectKey::Identifier(key), colon);
                } else {
                    return self.streams.parse_err(ERROR_MESSAGE);
                }

                let node =
                    self.nodes
                        .add_node(ExpressionNode::Leaf(Leaf::Variable(
                            VariableIdentifier { ident: key.clone() },
                        )));
                complete_entries.push((ObjectKey::Identifier(key), node));
                continue;
            } else if self.streams.peek(token::Bracket) {
                let (_, delim_span) = self.streams.parse_and_enter_group()?;
                break ObjectStackFrameState::EntryIndex(IndexAccess {
                    brackets: Brackets { delim_span },
                });
            } else {
                return self.streams.parse_err(ERROR_MESSAGE);
            }
        };
        Ok(self.push_stack_frame(ExpressionStackFrame::NonEmptyObject {
            braces,
            complete_entries,
            state,
        }))
    }

    fn add_leaf(&mut self, leaf: Leaf) -> WorkItem {
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

    pub(super) fn complete(self, root: ExpressionNodeId) -> Expression {
        Expression::new(root, self.nodes)
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
    NonEmptyArray {
        brackets: Brackets,
        items: Vec<ExpressionNodeId>,
    },
    /// A marker for an object literal.
    /// * When the object is opened, we add its inside to the parse stream stack
    /// * When the object is closed, we pop it from the parse stream stack
    NonEmptyObject {
        braces: Braces,
        complete_entries: Vec<(ObjectKey, ExpressionNodeId)>,
        state: ObjectStackFrameState,
    },
    /// A method call with a possibly incomplete list of parameters.
    /// * When the method parameters list is opened, we add its inside to the parse stream stack
    /// * When the method parameters list is closed, we pop it from the parse stream stack
    NonEmptyMethodCallParametersList {
        node: ExpressionNodeId,
        method: MethodAccess,
        parameters: Vec<ExpressionNodeId>,
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

#[derive(Clone)]
pub(super) enum ObjectKey {
    Identifier(Ident),
    Indexed {
        access: IndexAccess,
        index: ExpressionNodeId,
    },
}

pub(super) enum ObjectStackFrameState {
    EntryIndex(IndexAccess),
    #[allow(unused)]
    EntryValue(ObjectKey, Token![:]),
}

impl ExpressionStackFrame {
    fn precedence_to_bind_to_child(&self) -> OperatorPrecendence {
        match self {
            ExpressionStackFrame::Root => OperatorPrecendence::MIN,
            ExpressionStackFrame::Group { .. } => OperatorPrecendence::MIN,
            ExpressionStackFrame::NonEmptyArray { .. } => OperatorPrecendence::MIN,
            ExpressionStackFrame::NonEmptyObject { .. } => OperatorPrecendence::MIN,
            ExpressionStackFrame::NonEmptyMethodCallParametersList { .. } => {
                OperatorPrecendence::MIN
            }
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

pub(super) enum UnaryAtom {
    Leaf(Leaf),
    Group(DelimSpan),
    Array(Brackets),
    Object(Braces),
    PrefixUnaryOperation(PrefixUnaryOperation),
    Range(syn::RangeLimits),
}

pub(super) enum NodeExtension {
    PostfixOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    /// Used under Arrays, Objects, and Method Parameter List parents
    NonTerminalComma,
    Property(PropertyAccess),
    MethodCall(MethodAccess),
    Index(IndexAccess),
    Range(syn::RangeLimits),
    AssignmentOperation(Token![=]),
    CompoundAssignmentOperation(CompoundAssignmentOperation),
    EndOfStreamOrGroup,
    NoValidExtensionForCurrentParent,
}

impl NodeExtension {
    fn precedence(&self) -> OperatorPrecendence {
        match self {
            NodeExtension::PostfixOperation(op) => OperatorPrecendence::of_unary_operation(op),
            NodeExtension::BinaryOperation(op) => OperatorPrecendence::of_binary_operation(op),
            NodeExtension::NonTerminalComma => OperatorPrecendence::NonTerminalComma,
            NodeExtension::Property { .. } => OperatorPrecendence::Unambiguous,
            NodeExtension::MethodCall { .. } => OperatorPrecendence::Unambiguous,
            NodeExtension::Index { .. } => OperatorPrecendence::Unambiguous,
            NodeExtension::Range(_) => OperatorPrecendence::Range,
            NodeExtension::EndOfStreamOrGroup => OperatorPrecendence::MIN,
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
            | NodeExtension::MethodCall { .. }
            | NodeExtension::Index { .. }
            | NodeExtension::Range { .. }
            | NodeExtension::AssignmentOperation { .. }
            | NodeExtension::CompoundAssignmentOperation { .. }
            | NodeExtension::EndOfStreamOrGroup) => {
                WorkItem::TryApplyAlreadyParsedExtension { node, extension }
            }
            NodeExtension::NonTerminalComma => {
                unreachable!(
                    "Comma is only possible on method parameter list or array or object parent"
                )
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
