use super::*;

pub(super) struct ControlFlowAnalyzer<'a> {
    definitions: &'a Arena<VariableDefinitionId, VariableDefinitionData>,
    references: &'a mut Arena<VariableReferenceId, VariableReferenceData>,
    scopes: &'a Arena<ScopeId, ScopeData>,
    segments: &'a Arena<ControlFlowSegmentId, ControlFlowSegmentData>,
}

impl<'a> ControlFlowAnalyzer<'a> {
    pub(super) fn new(
        definitions: &'a Arena<VariableDefinitionId, VariableDefinitionData>,
        references: &'a mut Arena<VariableReferenceId, VariableReferenceData>,
        scopes: &'a Arena<ScopeId, ScopeData>,
        segments: &'a Arena<ControlFlowSegmentId, ControlFlowSegmentData>,
    ) -> ControlFlowAnalyzer<'a> {
        ControlFlowAnalyzer {
            definitions,
            references,
            scopes,
            segments,
        }
    }

    pub(super) fn analyze_and_update_references(&mut self) -> MarkFinalUseOutput {
        self.mark_last_reference_uses()
    }

    fn mark_last_reference_uses(&mut self) -> MarkFinalUseOutput {
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
            let ancestor_scopes = self.create_ancestor_scopes(definition.scope);

            let (last_use_candidates, markers) = self.last_reference_first_phase(
                definition_id,
                &definition.references,
                &ancestor_scopes,
            );

            self.mark_valid_last_references(
                definition_id,
                &ancestor_scopes,
                &markers,
                &last_use_candidates,
            );

            #[cfg(feature = "debug")]
            output.insert(definition_id, markers);
        }

        #[cfg(feature = "debug")]
        return output;
    }

    fn create_ancestor_scopes(&self, scope: ScopeId) -> HashSet<ScopeId> {
        let mut scopes = HashSet::new();
        let mut current = self.scopes.get(scope).parent;
        while let Some(scope) = current {
            scopes.insert(scope);
            current = self.scopes.get(scope).parent;
        }
        scopes
    }

    /// Phase 1:
    /// * Creates a shortlist of possible last-use references
    /// * Marks segments as NotFinal
    ///
    /// In phase 2, we will validate each candidate.
    fn last_reference_first_phase(
        &mut self,
        definition_id: VariableDefinitionId,
        references: &[VariableReferenceId],
        ancestor_scopes: &HashSet<ScopeId>,
    ) -> (
        Vec<VariableReferenceId>,
        HashMap<ControlFlowChild, SegmentMarker>,
    ) {
        let mut last_use_candidates = Vec::new();
        let mut markers = HashMap::new();
        let mut segments_with_references = HashSet::new();
        for reference_id in references.iter() {
            let reference = self.references.get(*reference_id);
            segments_with_references.insert(reference.segment);
        }

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
        (last_use_candidates, markers)
    }

    fn mark_valid_last_references(
        &mut self,
        definition_id: VariableDefinitionId,
        ancestor_scopes: &HashSet<ScopeId>,
        markers: &HashMap<ControlFlowChild, SegmentMarker>,
        last_use_candidates: &[VariableReferenceId],
    ) {
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
    }
}

#[cfg(feature = "debug")]
pub(super) type MarkFinalUseOutput =
    HashMap<VariableDefinitionId, HashMap<ControlFlowChild, SegmentMarker>>;
#[cfg(not(feature = "debug"))]
pub(super) type MarkFinalUseOutput = ();

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub(super) enum SegmentMarker {
    AlreadyHandled,
    NotFinal,
}
