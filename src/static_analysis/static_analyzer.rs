#![allow(unused)]
use super::*;

new_key!(pub(crate) FrameId);
new_key!(pub(crate) ScopeId);
new_key!(pub(crate) VariableDefinitionId);
new_key!(pub(crate) VariableReferenceId);
new_key!(pub(crate) ControlFlowSegmentId);
new_key!(pub(crate) CatchLocationId);

pub(crate) enum InterruptDetails<'a> {
    /// Break statement (targets loops or labeled blocks)
    Break {
        break_token: &'a Token![break],
        label: Option<&'a InterruptLabel>,
    },
    /// Continue statement (targets loops only)
    Continue {
        continue_token: &'a Token![continue],
        label: Option<&'a InterruptLabel>,
    },
    /// Revert statement (targets attempt blocks)
    Revert {
        revert_token: &'a RevertKeyword,
        label: Option<&'a InterruptLabel>,
    },
}

#[cfg(feature = "debug")]
#[derive(Clone, Copy, Debug)]
pub(crate) enum FinalUseAssertion {
    None,
    IsFinal(Span),
    IsNotFinal(Span),
}

#[derive(Debug)]
pub(crate) struct StaticDefinitions {
    pub(crate) root_frame: (FrameId, ScopeId),
    pub(crate) frames: Arena<FrameId, RuntimeFrame>,
    pub(crate) scopes: Arena<ScopeId, ScopeData>,
    pub(crate) definitions: Arena<VariableDefinitionId, VariableDefinitionData>,
    pub(crate) references: Arena<VariableReferenceId, VariableReferenceData>,
    pub(crate) catch_locations: Arena<CatchLocationId, CatchLocationData>,
    // Segment Debugging
    #[cfg(feature = "debug")]
    root_segments: Vec<(FrameId, ControlFlowSegmentId)>,
    #[cfg(feature = "debug")]
    segments: Arena<ControlFlowSegmentId, ControlFlowSegmentData>,
    #[cfg(feature = "debug")]
    final_use_debug: MarkFinalUseOutput,
}

#[allow(unused)]
pub(crate) struct StaticAnalyzer {
    frames_stack: Vec<FrameId>,
    frames: Arena<FrameId, AllocatedFrame>,
    scopes: Arena<ScopeId, AllocatedScope>,
    definitions: Arena<VariableDefinitionId, AllocatedVariableDefinition>,
    references: Arena<VariableReferenceId, AllocatedVariableReference>,
    catch_locations: Arena<CatchLocationId, CatchLocationData>,
    segments: Arena<ControlFlowSegmentId, ControlFlowSegmentData>,

    // >> Current positions (copied for convenience/performance)
    /// This is always frames_stack.last() (else placeholder)
    current_frame_id: FrameId,
    /// This is always frames_stack.last().scopes_stack.last() (else placeholder)
    current_scope_id: ScopeId,
    /// This is always frames_stack.last().segments_stack.last() (else placeholder)
    current_segment_id: ControlFlowSegmentId,
}

impl StaticAnalyzer {
    pub(crate) fn new_empty() -> Self {
        Self {
            current_frame_id: FrameId::new_placeholder(),
            current_scope_id: ScopeId::new_placeholder(),
            current_segment_id: ControlFlowSegmentId::new_placeholder(),
            frames_stack: vec![],
            frames: Arena::new(),
            scopes: Arena::new(),
            definitions: Arena::new(),
            references: Arena::new(),
            catch_locations: Arena::new(),
            segments: Arena::new(),
        }
    }

    pub(crate) fn finish(
        mut self,
        root_frame: (FrameId, ScopeId),
    ) -> ParseResult<StaticDefinitions> {
        assert!(
            self.frames_stack.is_empty(),
            "Cannot finish - Unpopped frames remain"
        );

        #[cfg(feature = "debug")]
        let root_segments: Vec<(FrameId, ControlFlowSegmentId)> = self
            .frames
            .iter()
            .map(|(frame_id, frame)| (frame_id, frame.defined_ref().root_segment))
            .collect();

        let frames = self.frames.map_all(AllocatedFrame::into_runtime);
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

        Ok(StaticDefinitions {
            root_frame,
            frames,
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

    pub(crate) fn allocate_frame(&mut self) -> FrameId {
        self.frames.add(AllocatedFrame::Allocated)
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

    // FRAMES
    // ======

    pub(crate) fn enter_frame(&mut self, frame_id: FrameId, scope_id: ScopeId, kind: FrameKind) {
        *self.frames.get_mut(frame_id) = AllocatedFrame::Defined(FrameData {
            kind_data: match kind {
                FrameKind::Root => FrameKindData::Root,
                FrameKind::Closure => FrameKindData::Closure {
                    lexical_parent: (self.current_frame_id, self.current_scope_id),
                    closed_variables: BTreeMap::new(),
                },
            },
            root_segment: ControlFlowSegmentId::new_placeholder(),
            scope_stack: Vec::new(),
            segment_stack: Vec::new(),
            catch_location_stack: Vec::new(),
        });
        self.frames_stack.push(frame_id);
        self.current_frame_id = frame_id;
        self.enter_scope(scope_id);
        let segment = self.enter_segment_with_valid_previous(None, None, SegmentKind::Sequential);
        let frame = self.current_frame_mut();
        frame.root_segment = segment;
    }

    /// Updates `current_frame_id` from the frames stack.
    /// Must be called after modifying `frames_stack`.
    fn update_current_frame_id(&mut self) {
        self.current_frame_id = self
            .frames_stack
            .last()
            .copied()
            .unwrap_or_else(FrameId::new_placeholder);
    }

    fn current_frame(&self) -> &FrameData {
        self.frames.get(self.current_frame_id).defined_ref()
    }

    fn current_frame_mut(&mut self) -> &mut FrameData {
        self.frames.get_mut(self.current_frame_id).defined_mut()
    }

    pub(crate) fn exit_frame(&mut self, frame_id: FrameId, scope_id: ScopeId) {
        self.exit_segment(self.current_frame().root_segment);
        assert!(
            self.segments_stack().is_empty(),
            "Segment stack not empty after exiting frame"
        );

        self.exit_scope(scope_id);
        assert!(
            self.scope_id_stack_mut().is_empty(),
            "Scope stack not empty after exiting frame"
        );

        let id = self.frames_stack.pop().expect("No frame to pop");
        assert_eq!(id, frame_id, "Popped frame is not the current frame");
        self.update_current_frame_id();

        // Restore current_scope_id and current_segment_id from the parent frame
        if !self.current_frame_id.is_placeholder() {
            self.update_current_scope_id();
            self.update_current_segment_id();
        }
    }

    // SCOPES
    // ======

    fn scope_id_stack(&self) -> &[ScopeId] {
        &self.current_frame().scope_stack
    }

    fn scope_id_stack_mut(&mut self) -> &mut Vec<ScopeId> {
        &mut self.current_frame_mut().scope_stack
    }

    fn current_scope(&mut self) -> &mut ScopeData {
        self.scopes.get_mut(self.current_scope_id).defined_mut()
    }

    pub(crate) fn enter_scope(&mut self, scope_id: ScopeId) {
        *self.scopes.get_mut(scope_id) = AllocatedScope::Defined(ScopeData {
            definitions: Vec::new(),
            parent: self.scope_id_stack_mut().last().copied(),
            frame: self.current_frame_id,
        });
        self.scope_id_stack_mut().push(scope_id);
        self.current_scope_id = scope_id;
    }

    /// Updates `current_scope_id` from the current frame's scope stack.
    /// Must be called after modifying the scope stack.
    fn update_current_scope_id(&mut self) {
        let scope_id = self
            .current_frame()
            .scope_stack
            .last()
            .copied()
            .unwrap_or_else(ScopeId::new_placeholder);
        self.current_scope_id = scope_id;
    }

    /// The scope parameter is just to help catch bugs.
    pub(crate) fn exit_scope(&mut self, scope: ScopeId) {
        let id = self.scope_id_stack_mut().pop().expect("No scope to pop");
        assert_eq!(id, scope, "Popped scope is not the current scope");
        self.update_current_scope_id();
    }

    pub(crate) fn define_variable(&mut self, id: VariableDefinitionId) {
        let definition = self.definitions.get_mut(id);
        let (name, definition_name_span) = definition.take_allocated();
        *definition = AllocatedVariableDefinition::Defined(VariableDefinitionData {
            scope: self.current_scope_id,
            segment: self.current_segment_id,
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
        let (name, reference_name_span) = self.references.get_mut(id).take_allocated();
        match self.find_definition_or_create_from_parent(self.current_frame_id, name.as_str()) {
            Some((def_id, def_scope_id, segment_id)) => {
                let reference = self.references.get_mut(id);
                *reference = AllocatedVariableReference::Defined(VariableReferenceData {
                    definition: def_id,
                    definition_scope: def_scope_id,
                    segment: segment_id,
                    reference_name_span,
                    is_final_reference: false, // Some will be set to true later
                    #[cfg(feature = "debug")]
                    assertion,
                });
                self.definitions
                    .get_mut(def_id)
                    .defined_mut()
                    .references
                    .push(id);
                self.segments
                    .get_mut(segment_id)
                    .children
                    .push(ControlFlowChild::VariableReference(id, def_id));
                Ok(())
            }
            None => reference_name_span
                .parse_err(format!("Cannot find variable `{}` in this scope", name)),
        }
    }

    // Returns:
    // - The definition id from the given frame
    // - The definition scope id from the given frame
    // - The segment id of the reference in the given frame
    fn find_definition_or_create_from_parent(
        &mut self,
        frame_id: FrameId,
        name: &str,
    ) -> Option<(VariableDefinitionId, ScopeId, ControlFlowSegmentId)> {
        let frame = self.frames.get_mut(frame_id).defined_mut();
        let segment_id = *frame.segment_stack.last().unwrap();
        let scope_id = *frame.scope_stack.last().unwrap();
        if let Some((def_id, scope_id)) =
            Self::find_definition_from_own_frame(&*frame, &self.scopes, &self.definitions, name)
        {
            return Some((def_id, scope_id, segment_id));
        }
        // We didn't find a variable in the current frame -
        // let's see if we can find it in the parent frame (recursively)
        let parent_frame_id = match &frame.kind_data {
            FrameKindData::Root => return None,
            FrameKindData::Closure {
                lexical_parent: (parent_frame_id, _),
                ..
            } => *parent_frame_id,
        };
        // Closed variable definitions must live in the frame's root scope/segment,
        // because at runtime they are defined during invoke() before the body
        // (and any child scopes) are evaluated.
        let frame_root_scope_id = frame.scope_stack[0];
        let frame_root_segment_id = frame.root_segment;
        let (parent_definition, parent_definition_scope, parent_segment_id) =
            self.find_definition_or_create_from_parent(parent_frame_id, name)?;
        // If we find it in the parent frame, then we need to connect it up over 3 steps:
        // 1. We need to create a reference for it in the parent frame
        let parent_reference_id =
            self.references
                .add(AllocatedVariableReference::Defined(VariableReferenceData {
                    definition: parent_definition,
                    definition_scope: parent_definition_scope,
                    segment: parent_segment_id,
                    // This span is only used for ownership resolution errors,
                    // but not for referenced closures where we copy the Referencable into the closure.
                    reference_name_span: Span::call_site(),
                    is_final_reference: false,
                    #[cfg(feature = "debug")]
                    assertion: FinalUseAssertion::None,
                }));
        self.definitions
            .get_mut(parent_definition)
            .defined_mut()
            .references
            .push(parent_reference_id);
        self.segments.get_mut(parent_segment_id).children.push(
            ControlFlowChild::VariableReference(parent_reference_id, parent_definition),
        );

        // 2. We need to create a definition for it in this frame's root scope
        let definition_id = self.definitions.add(AllocatedVariableDefinition::Defined(
            VariableDefinitionData {
                scope: frame_root_scope_id,
                segment: frame_root_segment_id,
                name: name.to_string(),
                // This is unused, so can put in a placeholder
                definition_name_span: Span::call_site(),
                references: vec![],
            },
        ));
        self.scopes
            .get_mut(frame_root_scope_id)
            .defined_mut()
            .definitions
            .push(definition_id);
        // TODO[functions]: This is incorrect - it should really be prepended before other control flow...
        // -- in fact I'm slightly surprised we don't get issues with references before definitions.
        self.segments
            .get_mut(frame_root_segment_id)
            .children
            .push(ControlFlowChild::VariableDefinition(definition_id));

        // 3. We need to add it to the closed variables of this frame
        let frame = self.frames.get_mut(frame_id).defined_mut();
        match &mut frame.kind_data {
            FrameKindData::Root => unreachable!("Already returned above"),
            FrameKindData::Closure {
                closed_variables, ..
            } => {
                closed_variables.insert(name.to_string(), (definition_id, parent_reference_id));
            }
        }

        Some((definition_id, frame_root_scope_id, segment_id))
    }

    fn find_definition_from_own_frame(
        frame: &FrameData,
        scopes: &Arena<ScopeId, AllocatedScope>,
        definitions: &Arena<VariableDefinitionId, AllocatedVariableDefinition>,
        name: &str,
    ) -> Option<(VariableDefinitionId, ScopeId)> {
        let scope_stack = frame.scope_stack.as_slice();
        for scope_id in scope_stack.iter().rev() {
            let scope = scopes.get(*scope_id).defined_ref();
            for &def_id in scope.definitions.iter().rev() {
                let def = definitions.get(def_id).defined_ref();
                if def.name == name {
                    return Some((def_id, def.scope));
                }
            }
        }
        match &frame.kind_data {
            FrameKindData::Root => None,
            FrameKindData::Closure {
                closed_variables, ..
            } => closed_variables
                .get(name)
                .map(|(def_id, _)| (*def_id, frame.scope_stack[0])),
        }
    }

    // SEGMENTS
    // ========

    fn segments_stack(&mut self) -> &mut Vec<ControlFlowSegmentId> {
        &mut self.current_frame_mut().segment_stack
    }

    fn current_segment(&mut self) -> &mut ControlFlowSegmentData {
        self.segments.get_mut(self.current_segment_id)
    }

    pub(crate) fn enter_next_segment(&mut self, segment_kind: SegmentKind) -> ControlFlowSegmentId {
        let parent_id = self.current_segment_id;
        let parent = self.segments.get(parent_id);
        if !matches!(parent.children, SegmentChildren::Sequential { .. }) {
            panic!("enter_next_segment can only be called with a sequential parent");
        }
        self.enter_segment_with_valid_previous(Some(parent_id), None, segment_kind)
    }

    pub(crate) fn enter_path_segment(
        &mut self,
        previous_sibling_id: Option<ControlFlowSegmentId>,
        segment_kind: SegmentKind,
    ) -> ControlFlowSegmentId {
        let parent_id = self.current_segment_id;
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
        self.enter_segment_with_valid_previous(Some(parent_id), previous_sibling_id, segment_kind)
    }

    pub(crate) fn exit_segment(&mut self, segment: ControlFlowSegmentId) {
        let id = self.segments_stack().pop().expect("No segment to pop");
        assert_eq!(id, segment, "Popped segment is not the current segment");
        self.update_current_segment_id();
    }

    /// Updates `current_segment_id` from the current frame's segment stack.
    /// Must be called after modifying the segment stack.
    fn update_current_segment_id(&mut self) {
        let segment_id = self
            .current_frame()
            .segment_stack
            .last()
            .copied()
            .unwrap_or_else(ControlFlowSegmentId::new_placeholder);
        self.current_segment_id = segment_id;
    }

    fn enter_segment_with_valid_previous(
        &mut self,
        parent_id: Option<ControlFlowSegmentId>,
        previous_sibling_id: Option<ControlFlowSegmentId>,
        segment_kind: SegmentKind,
    ) -> ControlFlowSegmentId {
        let child_id = self.segments.add(ControlFlowSegmentData {
            scope: self.current_scope_id,
            parent: parent_id,
            children: segment_kind.new_children(),
            segment_kind,
        });
        self.segments_stack().push(child_id);
        self.current_segment_id = child_id;
        if let Some(parent_id) = parent_id {
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
        }
        child_id
    }

    // CATCH LOCATIONS
    // ===============

    fn catch_location_stack(&self) -> &[CatchLocationId] {
        &self.current_frame().catch_location_stack
    }

    fn catch_location_stack_mut(&mut self) -> &mut Vec<CatchLocationId> {
        &mut self.current_frame_mut().catch_location_stack
    }

    pub(crate) fn register_catch_location(&mut self, data: CatchLocationData) -> CatchLocationId {
        self.catch_locations.add(data)
    }

    pub(crate) fn enter_catch(&mut self, catch_location_id: CatchLocationId) {
        self.catch_location_stack_mut().push(catch_location_id);
    }

    pub(crate) fn exit_catch(&mut self, catch_location_id: CatchLocationId) {
        let popped = self
            .catch_location_stack_mut()
            .pop()
            .expect("No catch to pop");
        assert_eq!(
            popped, catch_location_id,
            "Popped catch location is not the expected catch location"
        );
    }

    pub(crate) fn resolve_catch_for_interrupt(
        &self,
        interrupt_details: InterruptDetails,
    ) -> ParseResult<CatchLocationId> {
        match interrupt_details {
            InterruptDetails::Break {
                label: Some(label), ..
            } => {
                let label_str = label.ident_string();
                for &catch_location_id in self.catch_location_stack().iter().rev() {
                    let catch_location = self.catch_locations.get(catch_location_id);
                    match catch_location {
                        CatchLocationData::Loop {
                            label: Some(loc_label),
                        } => {
                            if loc_label == &label_str {
                                return Ok(catch_location_id);
                            }
                        }
                        CatchLocationData::LabeledBlock { label: loc_label } => {
                            if loc_label == &label_str {
                                return Ok(catch_location_id);
                            }
                        }
                        _ => {}
                    }
                }
                label.parse_err(
                    "A labelled break must be used inside a loop or block with a matching label",
                )
            }
            InterruptDetails::Break {
                label: None,
                break_token,
            } => {
                for &catch_location_id in self.catch_location_stack().iter().rev() {
                    let catch_location = self.catch_locations.get(catch_location_id);
                    if let CatchLocationData::Loop { .. } = catch_location {
                        return Ok(catch_location_id);
                    }
                }
                break_token
                    .span
                    .parse_err("A break must be used inside a loop")
            }
            InterruptDetails::Continue {
                label: Some(label), ..
            } => {
                let label_str = label.ident_string();
                for &catch_location_id in self.catch_location_stack().iter().rev() {
                    let catch_location = self.catch_locations.get(catch_location_id);
                    if let CatchLocationData::Loop {
                        label: Some(loc_label),
                    } = catch_location
                    {
                        if loc_label == &label_str {
                            return Ok(catch_location_id);
                        }
                    }
                }
                label.parse_err(
                    "A labelled continue must be used inside a loop with a matching label",
                )
            }
            InterruptDetails::Continue {
                label: None,
                continue_token,
            } => {
                for &catch_location_id in self.catch_location_stack().iter().rev() {
                    let catch_location = self.catch_locations.get(catch_location_id);
                    if let CatchLocationData::Loop { .. } = catch_location {
                        return Ok(catch_location_id);
                    }
                }
                continue_token
                    .span
                    .parse_err("A continue must be used inside a loop")
            }
            InterruptDetails::Revert {
                revert_token,
                label: Some(label),
            } => {
                let label_str = label.ident_string();
                for &catch_location_id in self.catch_location_stack().iter().rev() {
                    let catch_location = self.catch_locations.get(catch_location_id);
                    if let CatchLocationData::AttemptBlock {
                        label: Some(loc_label),
                    } = catch_location
                    {
                        if loc_label == &label_str {
                            return Ok(catch_location_id);
                        }
                    }
                }
                revert_token
                    .parse_err("A labelled revert must be used inside the left revertible part of an attempt arm, where the attempt has a matching label")
            }
            InterruptDetails::Revert {
                revert_token,
                label: None,
            } => {
                for &catch_location_id in self.catch_location_stack().iter().rev() {
                    let catch_location = self.catch_locations.get(catch_location_id);
                    if let CatchLocationData::AttemptBlock { .. } = catch_location {
                        return Ok(catch_location_id);
                    }
                }
                revert_token.parse_err(
                    "A revert must be used inside the left revertible part of an attempt arm",
                )
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum CatchLocationData {
    /// A loop (can catch unlabeled break/continue, or labeled if this location has a label)
    Loop { label: Option<String> },
    /// A labeled block (can only catch labeled break with matching label)
    LabeledBlock { label: String },
    /// An attempt block (can catch revert)
    AttemptBlock { label: Option<String> },
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

enum AllocatedFrame {
    Allocated,
    Defined(FrameData),
}

impl AllocatedFrame {
    fn defined_mut(&mut self) -> &mut FrameData {
        match self {
            AllocatedFrame::Defined(data) => data,
            _ => panic!("Frame was accessed before it was defined"),
        }
    }

    fn defined_ref(&self) -> &FrameData {
        match self {
            AllocatedFrame::Defined(data) => data,
            _ => panic!("Frame was accessed before it was defined"),
        }
    }

    fn into_runtime(self) -> RuntimeFrame {
        match self {
            AllocatedFrame::Defined(FrameData {
                kind_data:
                    FrameKindData::Closure {
                        closed_variables, ..
                    },
                ..
            }) => RuntimeFrame {
                closed_variables: closed_variables.into_values().collect(),
            },
            AllocatedFrame::Defined(FrameData {
                kind_data: FrameKindData::Root,
                ..
            }) => RuntimeFrame {
                closed_variables: Vec::new(),
            },
            _ => panic!("Frame was not defined"),
        }
    }
}

struct FrameData {
    kind_data: FrameKindData,
    root_segment: ControlFlowSegmentId,
    scope_stack: Vec<ScopeId>,
    catch_location_stack: Vec<CatchLocationId>,
    segment_stack: Vec<ControlFlowSegmentId>,
}

pub(crate) enum FrameKind {
    Root,
    Closure,
}

pub(crate) enum FrameKindData {
    Root,
    Closure {
        lexical_parent: (FrameId, ScopeId),
        // When a variable name matches to a variable defined in an ancestor scope,
        // we need to close over that variable.
        //
        // To do this, at every function boundary between these, we:
        // - Define a closed variable with the same name
        // - Create a variable reference which we can use to capture the variable
        //   from the parent closure when the closure is created.
        closed_variables: BTreeMap<String, (VariableDefinitionId, VariableReferenceId)>,
    },
}

#[derive(Debug)]
pub(crate) struct RuntimeFrame {
    pub(crate) closed_variables: Vec<(VariableDefinitionId, VariableReferenceId)>,
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
    pub(crate) definitions: Vec<VariableDefinitionId>,
    /// Only a None if this is a root scope of a frame
    pub(crate) parent: Option<ScopeId>,
    pub(crate) frame: FrameId,
}

pub(crate) enum ScopeKind {
    Root { root_frame: FrameId },
    Child,
    FunctionBoundary { new_frame: FrameId, span: SpanRange },
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
