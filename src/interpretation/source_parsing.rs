#![allow(unused)]
use super::*;
use std::cell::RefCell;

#[derive(Clone)]
pub(crate) struct ParseContext {
    #[allow(unused)]
    pub(crate) full_state: Rc<RefCell<ParseState>>,
}

impl ParseContext {
    pub(crate) fn new() -> Self {
        Self {
            full_state: Rc::new(RefCell::new(ParseState::new())),
        }
    }

    pub(crate) fn update<R>(&self, f: impl FnOnce(&mut ParseState) -> R) -> R {
        f(&mut self.full_state.borrow_mut())
    }
}

new_key!(pub(crate) ScopeId);
new_key!(pub(crate) VariableDefinitionId);
new_key!(pub(crate) VariableReferenceId);
new_key!(pub(crate) ControlFlowSegmentId);

#[derive(Clone)]
pub(crate) struct ScopeDefinitions {
    root_scope: ScopeId,
    scopes: ReadOnlyArena<ScopeId, ScopeData>,
    definitions: ReadOnlyArena<VariableDefinitionId, VariableDefinitionData>,
    references: ReadOnlyArena<VariableReferenceId, VariableReferenceData>,
}

#[allow(unused)]
pub(crate) struct ParseState {
    // SCOPE DATA
    scope_id_stack: Vec<ScopeId>,
    scopes: AppendOnlyArena<ScopeId, ScopeData>,
    definitions: AppendOnlyArena<VariableDefinitionId, VariableDefinitionData>,
    references: AppendOnlyArena<VariableReferenceId, VariableReferenceData>,
    // CONTROL FLOW DATA
    segments_stack: Vec<ControlFlowSegmentId>,
    segments: AppendOnlyArena<ControlFlowSegmentId, ControlFlowSegmentData>,
}

impl ParseState {
    fn new() -> Self {
        let mut scopes = AppendOnlyArena::new();
        let definitions = AppendOnlyArena::new();
        let references = AppendOnlyArena::new();
        let root_scope = scopes.add(ScopeData {
            parent: None,
            definitions: Vec::new(),
        });
        let mut segments = AppendOnlyArena::new();
        let root_segment = segments.add(ControlFlowSegmentData {
            scope: root_scope,
            parent: None,
            previous_sibling: None,
            segment_kind: SegmentKind::Sequential,
            children: SegmentKind::Sequential.new_children(),
        });
        Self {
            scope_id_stack: vec![root_scope],
            scopes,
            definitions,
            references,
            segments_stack: vec![root_segment],
            segments,
        }
    }

    pub(crate) fn finish(mut self) -> ScopeDefinitions {
        self.mark_last_use_of_variables();

        let root_scope = self.scope_id_stack.pop().expect("No scope to pop");
        assert!(
            self.scope_id_stack.is_empty(),
            "Cannot finish - Unpopped scopes remain"
        );

        ScopeDefinitions {
            root_scope,
            scopes: self.scopes.into_read_only(),
            definitions: self.definitions.into_read_only(),
            references: self.references.into_read_only(),
        }
    }

    fn current_scope_id(&self) -> ScopeId {
        *self.scope_id_stack.last().unwrap()
    }

    fn current_scope(&mut self) -> &mut ScopeData {
        self.scopes.get_mut(self.current_scope_id())
    }

    pub(crate) fn new_scope(&mut self) -> ScopeId {
        let new_scope = self.scopes.add(ScopeData {
            parent: Some(self.current_scope_id()),
            definitions: Vec::new(),
        });
        self.scope_id_stack.push(new_scope);
        new_scope
    }

    pub(crate) fn define_variable(&mut self, name: &Ident) -> VariableDefinitionId {
        let id = self.definitions.add(VariableDefinitionData {
            scope: self.current_scope_id(),
            segment: self.current_segment_id(),
            name: name.to_string(),
            definition_name_span: name.span(),
            references: Vec::new(),
        });
        self.current_scope().definitions.push(id);
        self.current_segment()
            .children
            .push(ControlFlowChild::VariableDefinition(id));
        id
    }

    pub(crate) fn reference_variable(&mut self, name: &Ident) -> ParseResult<VariableReferenceId> {
        let name_str = name.to_string();
        let span = name.span();
        for scope_id in self.scope_id_stack.iter().rev() {
            let scope = self.scopes.get(*scope_id);
            for &def_id in scope.definitions.iter().rev() {
                let def = self.definitions.get(def_id);
                if def.name == name_str {
                    let ref_id = self.references.add(VariableReferenceData {
                        definition: def_id,
                        segment: self.current_segment_id(),
                        reference_name_span: span,
                        is_final_reference: true, // This will be fixed up later
                    });
                    self.definitions.get_mut(def_id).references.push(ref_id);
                    self.current_segment()
                        .children
                        .push(ControlFlowChild::VariableReference(ref_id));
                    return Ok(ref_id);
                }
            }
        }
        span.parse_err("A variable must be defined before it is referenced.")
    }

    /// The scope parameter is just to help catch bugs.
    pub(crate) fn pop_scope(&mut self, scope: ScopeId) {
        let id = self.scope_id_stack.pop().expect("No scope to pop");
        assert_eq!(id, scope, "Popped scope is not the current scope");
    }

    // SEGMENTS
    // ========

    fn current_segment_id(&self) -> ControlFlowSegmentId {
        *self.segments_stack.last().unwrap()
    }

    fn current_segment(&mut self) -> &mut ControlFlowSegmentData {
        self.segments.get_mut(self.current_segment_id())
    }

    pub(crate) fn enter_next_segment(
        &mut self,
        scope_id: ScopeId,
        segment_kind: SegmentKind,
    ) -> ControlFlowSegmentId {
        let parent_id = self.current_segment_id();
        let parent = self.segments.get(parent_id);
        let previous_sibling_id = match parent.children {
            SegmentChildren::Sequential { last_cf_child, .. } => last_cf_child,
            SegmentChildren::TreeBased { .. } => {
                panic!("enter_next_segment cannot be called with a tree-based parent")
            }
        };
        self.enter_segment_with_valid_previous(
            scope_id,
            parent_id,
            previous_sibling_id,
            segment_kind,
        )
    }

    pub(crate) fn enter_parented_segment(
        &mut self,
        scope_id: ScopeId,
        previous_sibling_id: Option<ControlFlowSegmentId>,
        segment_kind: SegmentKind,
    ) -> ControlFlowSegmentId {
        let parent_id = self.current_segment_id();
        if let Some(previous_sibling_id) = previous_sibling_id {
            let previous_sibling = self.segments.get(previous_sibling_id);
            // It might be possible if gotos exist to have a non-local parent,
            // But this may impact lots of the algorithms, so forbid it for now.
            assert_eq!(
                previous_sibling.parent,
                Some(parent_id),
                "Previous sibling is not under the current parent"
            );
        }
        let parent = self.segments.get(parent_id);
        if !matches!(parent.children, SegmentChildren::TreeBased { .. }) {
            panic!("enter_parented_segment can only be called with a tree-based parent");
        }
        self.enter_segment_with_valid_previous(
            scope_id,
            parent_id,
            previous_sibling_id,
            segment_kind,
        )
    }

    pub(crate) fn exit_segment(&mut self, segment: ControlFlowSegmentId) {
        let id = self.segments_stack.pop().expect("No segment to pop");
        assert_eq!(id, segment, "Popped segment is not the current segment");
    }

    fn enter_segment_with_valid_previous(
        &mut self,
        scope_id: ScopeId,
        parent_id: ControlFlowSegmentId,
        previous_sibling_id: Option<ControlFlowSegmentId>,
        segment_kind: SegmentKind,
    ) -> ControlFlowSegmentId {
        let child_id = self.segments.add(ControlFlowSegmentData {
            scope: scope_id,
            parent: Some(parent_id),
            previous_sibling: previous_sibling_id,
            children: segment_kind.new_children(),
            segment_kind,
        });
        let parent = self.segments.get_mut(parent_id);
        match &mut parent.children {
            SegmentChildren::Sequential {
                ref mut children,
                last_cf_child,
            } => {
                children.push(ControlFlowChild::Segment(child_id));
                *last_cf_child = Some(child_id);
            }
            SegmentChildren::TreeBased { ref mut all_nodes } => {
                all_nodes.push(child_id);
            }
        }
        child_id
    }

    fn mark_last_use_of_variables(&mut self) {
        // todo!();
        /*
        During this algorithm, we'll record a piece of state against segments:
        * (no tag yet) = Unhandled
        * "Handled" = This segment might have last uses under it, all previous segments have
          (or one of their ancestors) has been marked NotFinal.
        * "NotFinal" indicates that all a segment and all its descendents are not a last use.

        For a given variable definition, create a set from its ancestor scopes, then:
        - Form a set of the segment ids of all the references
        - For each such segment id:
          - Mark all but the last reference in its segment as not final
          - Set `possibly_final = true` for this final reference
          - Check own segment and all parents in its segment stack:
            - If the segment has a scope above the variable definition’s scope, break
            - If the segment is marked dirty or a Loop, set possibly_final = false
            - If the segment is unmarked (i.e. not yet handled or dirty):
              - Mark itself as "handled"
              - Mark all previous siblings at its level as “dirty”
            - (Repeat with parent segment)
          - If possibly_final = true, add it to a candidates list, else mark it as not final
        - Go through all candidates again:
          - Check all its parents in the segment stack, if any dirty, mark candidate as not final
        */
    }
}

/// A control flow segment captures a section of code which executes in order.
///
/// A segment may have children, either:
/// * Sequential: Children are instructions and segments, which have a fixed order
/// * Tree-based: Only has segment children; these form multiple possible execution paths.
///
/// ## Standard Segment
/// * Children defined with `enter_new_sibling_segment`.
/// * Linear history
///
/// ## If Expression Segment
/// * Children defined with  `enter_new_child_segment`
/// * Example children tree
/// ```text
/// IfCondA           < BlockA
/// ^< ElseIfCondB    < BlockB
///    ^< ElseIfCondC < BlockC
///       ^<----------- ElseBlock
/// ```
///
/// ## For loop
/// - Will have `ExecutionCount::Multiple`
/// - Children defined with `enter_new_child_segment`
/// - Children are the pattern match segment, and then the body segment/s
struct ControlFlowSegmentData {
    scope: ScopeId,
    parent: Option<ControlFlowSegmentId>,
    previous_sibling: Option<ControlFlowSegmentId>,
    children: SegmentChildren,
    segment_kind: SegmentKind,
}

pub(crate) enum SegmentKind {
    Sequential,
    TreeBased,
    LoopingSequential,
}

impl SegmentKind {
    fn new_children(&self) -> SegmentChildren {
        match self {
            SegmentKind::Sequential | SegmentKind::LoopingSequential => {
                SegmentChildren::Sequential {
                    children: vec![],
                    last_cf_child: None,
                }
            }
            SegmentKind::TreeBased => SegmentChildren::TreeBased { all_nodes: vec![] },
        }
    }
}

enum SegmentChildren {
    Sequential {
        children: Vec<ControlFlowChild>,
        last_cf_child: Option<ControlFlowSegmentId>,
    },
    TreeBased {
        all_nodes: Vec<ControlFlowSegmentId>,
    },
}

impl SegmentChildren {
    fn push(&mut self, child: ControlFlowChild) {
        match self {
            SegmentChildren::Sequential { children, .. } => children.push(child),
            SegmentChildren::TreeBased { .. } => panic!("Cannot push instruction under a tree-based segment. It needs a sequential segment underneath it."),
        }
    }
}

enum ControlFlowChild {
    Segment(ControlFlowSegmentId),
    VariableDefinition(VariableDefinitionId),
    VariableReference(VariableReferenceId),
}

struct ScopeData {
    parent: Option<ScopeId>,
    definitions: Vec<VariableDefinitionId>,
}

struct VariableDefinitionData {
    scope: ScopeId,
    segment: ControlFlowSegmentId,
    name: String,
    definition_name_span: Span,
    references: Vec<VariableReferenceId>,
}

struct VariableReferenceData {
    definition: VariableDefinitionId,
    segment: ControlFlowSegmentId,
    reference_name_span: Span,
    /// If a reference is the last use of a variable in any execution path
    /// then it is able to be taken by value as Owned by the interpreter.
    /// This avoids needing to prompt the code writer from lots of clones.
    ///
    /// This value is calculated during [ParseState::mark_last_use_of_variables].
    is_final_reference: bool,
}
