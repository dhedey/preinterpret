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

#[derive(Clone, Debug)]
pub(crate) struct ScopeDefinitions {
    // Scopes
    pub(crate) root_scope: ScopeId,
    pub(crate) scopes: ReadOnlyArena<ScopeId, ScopeData>,
    pub(crate) definitions: ReadOnlyArena<VariableDefinitionId, VariableDefinitionData>,
    pub(crate) references: ReadOnlyArena<VariableReferenceId, VariableReferenceData>,
    // Segments
    #[cfg(feature = "debug")]
    root_segment: ControlFlowSegmentId,
    #[cfg(feature = "debug")]
    segments: ReadOnlyArena<ControlFlowSegmentId, ControlFlowSegmentData>,
    #[cfg(feature = "debug")]
    final_use_debug: MarkFinalUseOutput,
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
        #[cfg(feature = "debug")] // If debug is off
        let final_use_debug = self.mark_final_use_of_variables();
        #[cfg(not(feature = "debug"))]
        self.mark_final_use_of_variables();

        let root_scope = self.scope_id_stack.pop().expect("No scope to pop");
        assert!(
            self.scope_id_stack.is_empty(),
            "Cannot finish - Unpopped scopes remain"
        );

        #[allow(unused)] // If debug is off
        let root_segment = self.segments_stack.pop().expect("No segment to pop");
        assert!(
            self.segments_stack.is_empty(),
            "Cannot finish - Unpopped segments remain"
        );

        ScopeDefinitions {
            root_scope,
            scopes: self.scopes.into_read_only(),
            definitions: self.definitions.into_read_only(),
            references: self.references.into_read_only(),
            #[cfg(feature = "debug")]
            root_segment,
            #[cfg(feature = "debug")]
            segments: self.segments.into_read_only(),
            #[cfg(feature = "debug")]
            final_use_debug,
        }
    }

    fn current_scope_id(&self) -> ScopeId {
        *self.scope_id_stack.last().unwrap()
    }

    fn current_scope(&mut self) -> &mut ScopeData {
        self.scopes.get_mut(self.current_scope_id())
    }

    pub(crate) fn enter_scope(&mut self) -> ScopeId {
        let new_scope = self.scopes.add(ScopeData {
            parent: Some(self.current_scope_id()),
            definitions: Vec::new(),
        });
        self.scope_id_stack.push(new_scope);
        new_scope
    }

    pub(crate) fn define_inactive_variable(&mut self, name: &Ident) -> VariableDefinitionId {
        let id = self.definitions.add(VariableDefinitionData {
            scope: self.current_scope_id(),
            segment: self.current_segment_id(),
            name: name.to_string(),
            definition_name_span: name.span(),
            references: Vec::new(),
            active: false,
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
                if def.name == name_str && def.active {
                    let ref_id = self.references.add(VariableReferenceData {
                        definition: def_id,
                        definition_scope: def.scope,
                        segment: self.current_segment_id(),
                        reference_name_span: span,
                        is_final_reference: false, // Some will be set to true later
                    });
                    self.definitions.get_mut(def_id).references.push(ref_id);
                    self.current_segment()
                        .children
                        .push(ControlFlowChild::VariableReference(ref_id, def_id));
                    return Ok(ref_id);
                }
            }
        }
        span.parse_err("A variable must be defined before it is referenced.")
    }

    pub(crate) fn activate_pending_variable_definitions(&mut self) {
        for def_id in self
            .scopes
            .get_mut(self.current_scope_id())
            .definitions
            .iter()
        {
            let def = self.definitions.get_mut(*def_id);
            def.active = true;
        }
    }

    pub(crate) fn reenter_scope(&mut self, scope: ScopeId) {
        let new_scope = self.scopes.get(scope);
        let parent = new_scope.parent.expect("Cannot re-enter root scope");
        assert_eq!(
            parent,
            self.current_scope_id(),
            "Cannot re-enter a scope which is not a child of the current scope"
        );
        self.scope_id_stack.push(scope);
    }

    /// The scope parameter is just to help catch bugs.
    pub(crate) fn exit_scope(&mut self, scope: ScopeId) {
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

    pub(crate) fn enter_next_segment(&mut self, segment_kind: SegmentKind) -> ControlFlowSegmentId {
        let parent_id = self.current_segment_id();
        let parent = self.segments.get(parent_id);
        if !matches!(parent.children, SegmentChildren::Sequential { .. }) {
            panic!("enter_next_segment can only be called with a sequential parent");
        }
        self.enter_segment_with_valid_previous(parent_id, None, segment_kind)
    }

    pub(crate) fn enter_path_segment(
        &mut self,
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
        if !matches!(parent.children, SegmentChildren::PathBased { .. }) {
            panic!("enter_path_segment can only be called with a path-based parent");
        }
        self.enter_segment_with_valid_previous(parent_id, previous_sibling_id, segment_kind)
    }

    pub(crate) fn reenter_segment(&mut self, segment: ControlFlowSegmentId) {
        let new_segment = self.segments.get(segment);
        let parent = new_segment.parent.expect("Cannot re-enter root segment");
        assert_eq!(
            parent,
            self.current_segment_id(),
            "Cannot re-enter a segment which is not a child of the current segment"
        );
        self.segments_stack.push(segment);
    }

    pub(crate) fn exit_segment(&mut self, segment: ControlFlowSegmentId) {
        let id = self.segments_stack.pop().expect("No segment to pop");
        assert_eq!(id, segment, "Popped segment is not the current segment");
    }

    fn enter_segment_with_valid_previous(
        &mut self,
        parent_id: ControlFlowSegmentId,
        previous_sibling_id: Option<ControlFlowSegmentId>,
        segment_kind: SegmentKind,
    ) -> ControlFlowSegmentId {
        let child_id = self.segments.add(ControlFlowSegmentData {
            scope: self.current_scope_id(),
            parent: Some(parent_id),
            children: segment_kind.new_children(),
            segment_kind,
        });
        self.segments_stack.push(child_id);
        let parent = self.segments.get_mut(parent_id);
        match &mut parent.children {
            SegmentChildren::Sequential { ref mut children } => {
                children.push(ControlFlowChild::Segment(child_id));
            }
            SegmentChildren::PathBased {
                node_previous_map: ref mut node_parent_map,
            } => {
                node_parent_map.insert(child_id, previous_sibling_id);
            }
        }
        child_id
    }

    fn mark_final_use_of_variables(&mut self) -> MarkFinalUseOutput {
        // ALGORITHM OVERVIEW
        // ==================
        //
        // The goal of this algorithm is to efficiently flag variable references as
        // "is_final_reference" if they are definitely the last use of a variable
        // in any possible execution path.
        //
        // This is quite subtle because of branching control flow, and loops.
        //
        // For each variable definition, this algorithm works in two phases:
        // 1. Identify segments where references occur, and for each:
        //    - Add the last reference in the segment as a candidate for last use
        //    - Mark all previous segments/references as not final.
        //      This is recursive as "previous siblings" of self and each ancestor segment
        //      We mark nodes as Handled | NotFinal to save repeated work.
        // 2. For each candidate, check if it's actually a last use:
        //    - Check it and all its relevant ancestor segments are:
        //      - Not marked NotFinal
        //      - Not loops
        //    - If all these checks pass, then mark it as a final reference.

        #[cfg(feature = "debug")]
        let mut output = HashMap::new();

        for (definition_id, definition) in self.definitions.iter() {
            let mut last_use_candidates = Vec::new();
            let mut markers = HashMap::new();
            let ancestor_scopes = {
                let mut scopes = HashSet::new();
                let mut current = self.scopes.get(definition.scope).parent;
                while let Some(scope) = current {
                    scopes.insert(scope);
                    current = self.scopes.get(scope).parent;
                }
                scopes
            };
            let mut segments_with_references = HashSet::new();
            for reference_id in definition.references.iter() {
                let reference = self.references.get(*reference_id);
                segments_with_references.insert(reference.segment);
            }

            // ======================================
            // PHASE 1 - We create a shortlist, and mark NotFinal segments
            // ======================================
            for segment_id in segments_with_references {
                let segment = self.segments.get(segment_id);
                let mut reference_ids = match segment.children {
                    SegmentChildren::Sequential { ref children, .. } => children
                        .iter()
                        .filter_map(|c| match c {
                            ControlFlowChild::VariableReference(ref_id, def_id)
                                if *def_id == definition_id =>
                            {
                                Some(*ref_id)
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    SegmentChildren::PathBased { .. } => {
                        panic!("Segment was marked as having references, but is path based")
                    }
                };
                // The last reference in the segment is a candidate for last use
                assert!(
                    !reference_ids.is_empty(),
                    "Segment was marked as having references, but none were found"
                );
                let last_reference_id = reference_ids.pop().unwrap();
                last_use_candidates.push(last_reference_id);

                // For each segment level between current up to the scope of the variable definition:
                // - Mark all previous segments at its level as NotFinal
                let mut parent_segment_id = Some(segment_id);
                let mut own_child_id =
                    ControlFlowChild::VariableReference(last_reference_id, definition_id);

                while let Some(parent_seg_id) = parent_segment_id {
                    let parent = self.segments.get(parent_seg_id);
                    let marker = markers.get(&own_child_id);
                    match marker {
                        Some(SegmentMarker::AlreadyHandled | SegmentMarker::NotFinal) => {
                            break;
                        }
                        None => {
                            markers.insert(own_child_id, SegmentMarker::AlreadyHandled);
                        }
                    }

                    // Mark all previous siblings as NotFinal
                    match parent.children {
                        SegmentChildren::PathBased {
                            ref node_previous_map,
                        } => {
                            // Walk up the tree of previous siblings, marking all as NotFinal
                            let own_seg_id = match own_child_id {
                                ControlFlowChild::Segment(id) => id,
                                _ => panic!("The child of a path-based segment must be a segment"),
                            };
                            let mut previous = node_previous_map.get(&own_seg_id).unwrap();
                            while let Some(prev_id) = *previous {
                                markers.insert(
                                    ControlFlowChild::Segment(prev_id),
                                    SegmentMarker::NotFinal,
                                );
                                previous = node_previous_map.get(&prev_id).unwrap();
                            }
                        }
                        SegmentChildren::Sequential { ref children } => {
                            let mut before_current_segment = false;
                            for sibling in children.iter().rev() {
                                if sibling == &own_child_id {
                                    before_current_segment = true;
                                    continue;
                                }
                                if before_current_segment {
                                    markers.insert(*sibling, SegmentMarker::NotFinal);
                                }
                            }
                        }
                    }
                    if ancestor_scopes.contains(&parent.scope) {
                        break;
                    }
                    own_child_id = ControlFlowChild::Segment(parent_seg_id);
                    parent_segment_id = parent.parent;
                }
            }

            // ======================================
            // PHASE 2 - We validate each candidate
            // ======================================
            for candidate_id in last_use_candidates.iter() {
                if let Some(SegmentMarker::NotFinal) = markers.get(
                    &ControlFlowChild::VariableReference(*candidate_id, definition_id),
                ) {
                    continue;
                }
                let candidate = self.references.get_mut(*candidate_id);
                let mut possibly_final = true;
                let mut current_segment_id = Some(candidate.segment);
                while let Some(seg_id) = current_segment_id {
                    let current_segment = self.segments.get(seg_id);
                    if ancestor_scopes.contains(&current_segment.scope) {
                        break;
                    }
                    if current_segment.segment_kind.is_looping() {
                        possibly_final = false;
                        break;
                    }
                    if let Some(SegmentMarker::NotFinal) =
                        markers.get(&ControlFlowChild::Segment(seg_id))
                    {
                        possibly_final = false;
                        break;
                    }
                    current_segment_id = current_segment.parent;
                }
                if possibly_final {
                    candidate.is_final_reference = true;
                }
            }
            #[cfg(feature = "debug")]
            output.insert(definition_id, markers);
        }

        #[cfg(feature = "debug")]
        return output;
    }
}

#[cfg(feature = "debug")]
type MarkFinalUseOutput = HashMap<VariableDefinitionId, HashMap<ControlFlowChild, SegmentMarker>>;
#[cfg(not(feature = "debug"))]
type MarkFinalUseOutput = ();

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum SegmentMarker {
    AlreadyHandled,
    NotFinal,
}

/// A control flow segment captures a section of code which executes in order.
///
/// A segment may have children, either:
/// * Sequential: Children are instructions and segments, which have a fixed order
/// * Tree-based: Only has segment children; these form multiple possible execution paths.
///
/// See `expressions/control_flow.rs` for some examples of how various segments are created.
#[derive(Debug)]
pub(crate) struct ControlFlowSegmentData {
    scope: ScopeId,
    parent: Option<ControlFlowSegmentId>,
    children: SegmentChildren,
    segment_kind: SegmentKind,
}

#[derive(Debug)]
pub(crate) enum SegmentKind {
    Sequential,
    PathBased,
    LoopingSequential,
}

impl SegmentKind {
    fn is_looping(&self) -> bool {
        match self {
            SegmentKind::Sequential => false,
            SegmentKind::PathBased => false,
            SegmentKind::LoopingSequential => true,
        }
    }

    fn new_children(&self) -> SegmentChildren {
        match self {
            SegmentKind::Sequential | SegmentKind::LoopingSequential => {
                SegmentChildren::Sequential { children: vec![] }
            }
            SegmentKind::PathBased => SegmentChildren::PathBased {
                node_previous_map: HashMap::new(),
            },
        }
    }
}

#[derive(Debug)]
enum SegmentChildren {
    Sequential {
        children: Vec<ControlFlowChild>,
    },
    PathBased {
        node_previous_map: HashMap<ControlFlowSegmentId, Option<ControlFlowSegmentId>>,
    },
}

impl SegmentChildren {
    fn push(&mut self, child: ControlFlowChild) {
        match self {
            SegmentChildren::Sequential { children, .. } => children.push(child),
            SegmentChildren::PathBased { .. } => panic!("Cannot push instruction under a tree-based segment. It needs a sequential segment underneath it."),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ControlFlowChild {
    Segment(ControlFlowSegmentId),
    VariableDefinition(VariableDefinitionId),
    VariableReference(VariableReferenceId, VariableDefinitionId),
}

#[derive(Debug)]
pub(crate) struct ScopeData {
    pub(crate) parent: Option<ScopeId>,
    pub(crate) definitions: Vec<VariableDefinitionId>,
}

#[derive(Debug)]
pub(crate) struct VariableDefinitionData {
    pub(crate) scope: ScopeId,
    pub(crate) segment: ControlFlowSegmentId,
    pub(crate) name: String,
    pub(crate) definition_name_span: Span,
    pub(crate) references: Vec<VariableReferenceId>,
    /// In a `let x = x + 1` statement, the RHS is executed against previous bindings.
    /// We allow the `let x` to create a binding, but only activate it for matching after the control flow completes.
    pub(crate) active: bool,
}

#[derive(Debug)]
pub(crate) struct VariableReferenceData {
    pub(crate) definition: VariableDefinitionId,
    pub(crate) definition_scope: ScopeId,
    pub(crate) segment: ControlFlowSegmentId,
    pub(crate) reference_name_span: Span,
    /// If a reference is the last use of a variable in any execution path
    /// then it is able to be taken by value as Owned by the interpreter.
    /// This avoids needing to prompt the code writer from lots of clones.
    ///
    /// This value is calculated during [ParseState::mark_final_use_of_variables].
    pub(crate) is_final_reference: bool,
}
