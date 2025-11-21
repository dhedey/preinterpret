#![allow(unused)]
use super::*;

new_key!(pub(crate) ScopeId);
new_key!(pub(crate) VariableDefinitionId);
new_key!(pub(crate) VariableReferenceId);
new_key!(pub(crate) ControlFlowSegmentId);
new_key!(pub(crate) CatchLocationId);

#[cfg(feature = "debug")]
#[derive(Clone, Copy, Debug)]
pub(crate) enum FinalUseAssertion {
    None,
    IsFinal(Span),
    IsNotFinal(Span),
}

#[derive(Debug)]
pub(crate) struct ScopeDefinitions {
    // Scopes
    pub(crate) root_scope: ScopeId,
    pub(crate) scopes: Arena<ScopeId, ScopeData>,
    pub(crate) definitions: Arena<VariableDefinitionId, VariableDefinitionData>,
    pub(crate) references: Arena<VariableReferenceId, VariableReferenceData>,
    // Catch locations
    pub(crate) catch_locations: Arena<CatchLocationId, CatchLocationData>,
    // Segments
    #[cfg(feature = "debug")]
    root_segment: ControlFlowSegmentId,
    #[cfg(feature = "debug")]
    segments: Arena<ControlFlowSegmentId, ControlFlowSegmentData>,
    #[cfg(feature = "debug")]
    final_use_debug: MarkFinalUseOutput,
}

#[allow(unused)]
pub(crate) struct FlowAnalysisState {
    // SCOPE DATA
    scope_id_stack: Vec<ScopeId>,
    scopes: Arena<ScopeId, AllocatedScope>,
    definitions: Arena<VariableDefinitionId, AllocatedVariableDefinition>,
    references: Arena<VariableReferenceId, AllocatedVariableReference>,
    // CATCH LOCATION DATA
    catch_locations: Arena<CatchLocationId, CatchLocationData>,
    labeled_catch_locations: HashMap<String, CatchLocationId>,
    loop_stack: Vec<CatchLocationId>,
    attempt_stack: Vec<CatchLocationId>,
    // CONTROL FLOW DATA
    segments_stack: Vec<ControlFlowSegmentId>,
    segments: Arena<ControlFlowSegmentId, ControlFlowSegmentData>,
}

impl FlowAnalysisState {
    pub(crate) fn new() -> Self {
        let mut scopes = Arena::new();
        let definitions = Arena::new();
        let references = Arena::new();
        let root_scope = scopes.add(AllocatedScope::Defined(ScopeData {
            parent: None,
            definitions: Vec::new(),
        }));
        let mut segments = Arena::new();
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
            catch_locations: Arena::new(),
            labeled_catch_locations: HashMap::new(),
            loop_stack: Vec::new(),
            attempt_stack: Vec::new(),
            segments_stack: vec![root_segment],
            segments,
        }
    }

    pub(crate) fn finish(mut self) -> ParseResult<ScopeDefinitions> {
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

        let scopes = self.scopes.map_all(AllocatedScope::into_defined);
        let definitions = self
            .definitions
            .map_all(AllocatedVariableDefinition::into_defined);
        let mut references = self
            .references
            .map_all(AllocatedVariableReference::into_defined);

        let mut analyzer =
            ControlFlowAnalyzer::new(&definitions, &mut references, &scopes, &self.segments);

        #[cfg(feature = "debug")] // If debug is off
        let final_use_debug = analyzer.analyze_and_update_references();
        #[cfg(not(feature = "debug"))]
        analyzer.analyze_and_update_references();

        #[cfg(feature = "debug")]
        {
            for (_, data) in references.iter() {
                match data.assertion {
                    FinalUseAssertion::IsFinal(span) if !data.is_final_reference => {
                        return span.parse_err(
                            "Assertion failed. Reference was calculated to be non-final.",
                        );
                    }
                    FinalUseAssertion::IsNotFinal(span) if data.is_final_reference => {
                        return span
                            .parse_err("Assertion failed. Reference was calculated to be final.");
                    }
                    _ => {}
                }
            }
        }

        Ok(ScopeDefinitions {
            root_scope,
            scopes,
            definitions,
            references,
            catch_locations: self.catch_locations,
            #[cfg(feature = "debug")]
            root_segment,
            #[cfg(feature = "debug")]
            segments: self.segments,
            #[cfg(feature = "debug")]
            final_use_debug,
        })
    }

    pub(crate) fn allocate_scope(&mut self) -> ScopeId {
        self.scopes.add(AllocatedScope::Allocated)
    }

    pub(crate) fn allocate_variable_reference(&mut self, name: &Ident) -> VariableReferenceId {
        self.references.add(AllocatedVariableReference::Allocated {
            name: name.to_string(),
            span: name.span(),
        })
    }

    pub(crate) fn allocate_variable_definition(&mut self, name: &Ident) -> VariableDefinitionId {
        self.definitions
            .add(AllocatedVariableDefinition::Allocated {
                name: name.to_string(),
                span: name.span(),
            })
    }

    fn current_scope_id(&self) -> ScopeId {
        *self.scope_id_stack.last().unwrap()
    }

    fn current_scope(&mut self) -> &mut ScopeData {
        self.scopes.get_mut(self.current_scope_id()).defined_mut()
    }

    pub(crate) fn enter_scope(&mut self, scope_id: ScopeId) {
        *self.scopes.get_mut(scope_id) = AllocatedScope::Defined(ScopeData {
            parent: Some(self.current_scope_id()),
            definitions: Vec::new(),
        });
        self.scope_id_stack.push(scope_id);
    }

    pub(crate) fn define_variable(&mut self, id: VariableDefinitionId) {
        let scope = self.current_scope_id();
        let segment = self.current_segment_id();
        let definition = self.definitions.get_mut(id);
        let (name, definition_name_span) = definition.take_allocated();
        *definition = AllocatedVariableDefinition::Defined(VariableDefinitionData {
            scope,
            segment,
            name,
            definition_name_span,
            references: Vec::new(),
        });
        self.current_scope().definitions.push(id);
        self.current_segment()
            .children
            .push(ControlFlowChild::VariableDefinition(id));
    }

    pub(crate) fn reference_variable(
        &mut self,
        id: VariableReferenceId,
        #[cfg(feature = "debug")] assertion: FinalUseAssertion,
    ) -> ParseResult<()> {
        let segment = self.current_segment_id();
        let reference = self.references.get_mut(id);
        let (name, reference_name_span) = reference.take_allocated();
        for scope_id in self.scope_id_stack.iter().rev() {
            let scope = self.scopes.get(*scope_id).defined_ref();
            for &def_id in scope.definitions.iter().rev() {
                let def = self.definitions.get_mut(def_id).defined_mut();
                if def.name == name {
                    *reference = AllocatedVariableReference::Defined(VariableReferenceData {
                        definition: def_id,
                        definition_scope: def.scope,
                        segment,
                        reference_name_span,
                        is_final_reference: false, // Some will be set to true later
                        #[cfg(feature = "debug")]
                        assertion,
                    });
                    def.references.push(id);
                    self.current_segment()
                        .children
                        .push(ControlFlowChild::VariableReference(id, def_id));
                    return Ok(());
                }
            }
        }
        reference_name_span.parse_err(format!("Cannot find variable `{}` in this scope", name))
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

    pub(crate) fn register_catch_location(&mut self, kind: CatchLocationKind) -> CatchLocationId {
        self.catch_locations.add(CatchLocationData { kind })
    }

    pub(crate) fn register_labeled_catch_location(
        &mut self,
        label: &str,
        location_id: CatchLocationId,
    ) {
        self.labeled_catch_locations
            .insert(label.to_string(), location_id);
    }

    pub(crate) fn resolve_label_to_catch_location(&self, label: &str) -> Option<CatchLocationId> {
        self.labeled_catch_locations.get(label).copied()
    }

    pub(crate) fn enter_loop(&mut self, catch_location_id: CatchLocationId) {
        self.loop_stack.push(catch_location_id);
    }

    pub(crate) fn exit_loop(&mut self, catch_location_id: CatchLocationId) {
        let popped = self.loop_stack.pop().expect("No loop to pop");
        assert_eq!(
            popped, catch_location_id,
            "Popped loop is not the expected loop"
        );
    }

    pub(crate) fn current_loop_catch_location(&self) -> Option<CatchLocationId> {
        self.loop_stack.last().copied()
    }

    pub(crate) fn enter_attempt(&mut self, catch_location_id: CatchLocationId) {
        self.attempt_stack.push(catch_location_id);
    }

    pub(crate) fn exit_attempt(&mut self, catch_location_id: CatchLocationId) {
        let popped = self.attempt_stack.pop().expect("No attempt to pop");
        assert_eq!(
            popped, catch_location_id,
            "Popped attempt is not the expected attempt"
        );
    }

    pub(crate) fn current_attempt_catch_location(&self) -> Option<CatchLocationId> {
        self.attempt_stack.last().copied()
    }
}

/// A control flow segment captures a section of code which executes in order.
///
/// A segment may have children, either:
/// Represents a location where control flow interrupts (break, continue, revert) can be caught.
#[derive(Debug)]
pub(crate) struct CatchLocationData {
    pub(crate) kind: CatchLocationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CatchLocationKind {
    /// A loop (can catch unlabeled break/continue, or labeled if this location has a label)
    Loop,
    /// A labeled block (can only catch labeled break with matching label)
    LabeledBlock,
    /// An attempt block (can catch revert)
    AttemptBlock,
}

/// * Sequential: Children are instructions and segments, which have a fixed order
/// * Tree-based: Only has segment children; these form multiple possible execution paths.
///
/// See `expressions/control_flow.rs` for some examples of how various segments are created.
#[derive(Debug)]
pub(crate) struct ControlFlowSegmentData {
    pub(super) scope: ScopeId,
    pub(super) parent: Option<ControlFlowSegmentId>,
    pub(super) children: SegmentChildren,
    pub(super) segment_kind: SegmentKind,
}

#[derive(Debug)]
pub(crate) enum SegmentKind {
    Sequential,
    PathBased,
    LoopingSequential,
    RevertibleSequential,
}

impl SegmentKind {
    pub(crate) fn is_looping(&self) -> bool {
        match self {
            SegmentKind::Sequential => false,
            SegmentKind::PathBased => false,
            SegmentKind::LoopingSequential => true,
            SegmentKind::RevertibleSequential => false,
        }
    }

    fn new_children(&self) -> SegmentChildren {
        match self {
            SegmentKind::Sequential
            | SegmentKind::LoopingSequential
            | SegmentKind::RevertibleSequential => SegmentChildren::Sequential { children: vec![] },
            SegmentKind::PathBased => SegmentChildren::PathBased {
                node_previous_map: HashMap::new(),
            },
        }
    }
}

#[derive(Debug)]
pub(super) enum SegmentChildren {
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
pub(super) enum ControlFlowChild {
    Segment(ControlFlowSegmentId),
    VariableDefinition(VariableDefinitionId),
    VariableReference(VariableReferenceId, VariableDefinitionId),
}

enum AllocatedScope {
    Allocated,
    Defined(ScopeData),
}

impl AllocatedScope {
    fn into_defined(self) -> ScopeData {
        match self {
            AllocatedScope::Defined(data) => data,
            _ => panic!("Scope was not defined"),
        }
    }

    fn defined_mut(&mut self) -> &mut ScopeData {
        match self {
            AllocatedScope::Defined(data) => data,
            _ => panic!("Scope was accessed before it was defined"),
        }
    }

    fn defined_ref(&self) -> &ScopeData {
        match self {
            AllocatedScope::Defined(data) => data,
            _ => panic!("Scope was accessed before it was defined"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ScopeData {
    pub(crate) parent: Option<ScopeId>,
    pub(crate) definitions: Vec<VariableDefinitionId>,
}

enum AllocatedVariableDefinition {
    Allocated { name: String, span: Span },
    Updating,
    Defined(VariableDefinitionData),
}

impl AllocatedVariableDefinition {
    fn into_defined(self) -> VariableDefinitionData {
        match self {
            AllocatedVariableDefinition::Defined(data) => data,
            _ => {
                panic!("Variable definition was allocated but not instantiated during control flow")
            }
        }
    }

    fn take_allocated(&mut self) -> (String, Span) {
        match core::mem::replace(self, AllocatedVariableDefinition::Updating) {
            AllocatedVariableDefinition::Allocated { name, span } => (name, span),
            _ => panic!("Variable was already defined"),
        }
    }

    fn defined_mut(&mut self) -> &mut VariableDefinitionData {
        match self {
            AllocatedVariableDefinition::Defined(data) => data,
            _ => panic!("Variable was not defined"),
        }
    }

    fn defined_ref(&self) -> &VariableDefinitionData {
        match self {
            AllocatedVariableDefinition::Defined(data) => data,
            _ => panic!("Variable was not defined"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct VariableDefinitionData {
    pub(crate) scope: ScopeId,
    pub(crate) segment: ControlFlowSegmentId,
    pub(crate) name: String,
    pub(crate) definition_name_span: Span,
    pub(crate) references: Vec<VariableReferenceId>,
}

enum AllocatedVariableReference {
    Allocated { name: String, span: Span },
    Updating,
    Defined(VariableReferenceData),
}

impl AllocatedVariableReference {
    fn take_allocated(&mut self) -> (String, Span) {
        match core::mem::replace(self, AllocatedVariableReference::Updating) {
            AllocatedVariableReference::Allocated { name, span } => (name, span),
            _ => panic!("Variable was already defined"),
        }
    }

    fn into_defined(self) -> VariableReferenceData {
        match self {
            AllocatedVariableReference::Defined(data) => data,
            _ => panic!("Variable reference was allocated but not defined during control flow"),
        }
    }

    fn defined_mut(&mut self) -> &mut VariableReferenceData {
        match self {
            AllocatedVariableReference::Defined(data) => data,
            _ => panic!("Variable was not defined"),
        }
    }
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
    #[cfg(feature = "debug")]
    pub(crate) assertion: FinalUseAssertion,
}
