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

    pub(crate) fn update(&self, f: impl FnOnce(&mut ParseState)) {
        f(&mut self.full_state.borrow_mut());
    }
}

new_key!(pub(crate) ScopeId);
new_key!(pub(crate) VariableDefinitionId);
new_key!(pub(crate) VariableReferenceId);

#[derive(Clone)]
pub(crate) struct ScopeDefinitions {
    root_scope: ScopeId,
    scopes: ReadOnlyArena<ScopeId, ScopeData>,
    definitions: ReadOnlyArena<VariableDefinitionId, VariableDefinitionData>,
    references: ReadOnlyArena<VariableReferenceId, VariableReferenceData>,
}

pub(crate) struct ParseState {
    #[allow(unused)]
    current_scope_id: ScopeId,
    #[allow(unused)]
    scopes: AppendOnlyArena<ScopeId, ScopeData>,
    #[allow(unused)]
    definitions: AppendOnlyArena<VariableDefinitionId, VariableDefinitionData>,
    #[allow(unused)]
    references: AppendOnlyArena<VariableReferenceId, VariableReferenceData>,
}

impl ParseState {
    fn new() -> Self {
        let mut scopes = AppendOnlyArena::new();
        let definitions = AppendOnlyArena::new();
        let references = AppendOnlyArena::new();
        let root = scopes.add(ScopeData {
            parent: None,
            definitions: Vec::new(),
        });
        Self { current_scope_id: root, scopes, definitions, references }
    }

    pub(crate) fn finish(mut self) -> ScopeDefinitions {
        assert!(self.current_scope().parent.is_none(), "Cannot finish - Unpopped scopes remain");

        ScopeDefinitions {
            root_scope: self.current_scope_id,
            scopes: self.scopes.into_read_only(),
            definitions: self.definitions.into_read_only(),
            references: self.references.into_read_only(),
        }
    }

    fn current_scope(&mut self) -> &mut ScopeData {
        self.scopes.get_mut(self.current_scope_id)
    }

    pub(crate) fn new_scope(&mut self) -> ScopeId {
        let new_scope = self.scopes.add(ScopeData {
            parent: Some(self.current_scope_id),
            definitions: Vec::new(),
        });
        self.current_scope_id = new_scope;
        new_scope
    }

    pub(crate) fn define_variable(&mut self, name: Spanned<&str>) -> VariableDefinitionId {
        let id = self.definitions
            .add(VariableDefinitionData {
                scope: self.current_scope_id,
                name: name.to_string(),
                definition_name_span: name.span_range.start(),
                references: Vec::new(),
            });
        self.current_scope().definitions.push(id);
        id
    }

    /// The scope parameter is just to help catch bugs.
    pub(crate) fn pop_scope(&mut self, scope: ScopeId) {
        assert_eq!(self.current_scope_id, scope, "Popped scope is not the current scope");
        let parent = self.current_scope().parent;
        match parent {
            None => panic!("Cannot pop the root scope"),
            Some(parent) => {
                self.current_scope_id = parent;
            }
        }   
    }
}

struct ScopeData {
    parent: Option<ScopeId>,
    definitions: Vec<VariableDefinitionId>,
}

struct VariableDefinitionData {
    scope: ScopeId,
    name: String,
    definition_name_span: Span,
    references: Vec<VariableReferenceId>,
}

struct VariableReferenceData {
    definition: VariableDefinitionId,
    reference_name_span: Span,
}

// // LAST USE RESOLUTION
// // => Effectively we can define a partial order across references, based on "X can execute before Y"
// // => If Y is a leaf, it can be marked as a last use
// // => As we discover a new reference, we can look at existing leaves and see if should be set as not last use 
// // 
// // SCOPE A: ChildScopes::Sequential
// let x = 1;
// attempt { // SCOPE B: ChildScopes::AtMostOneUnilateral
//  // NOTE: We can't take x as value in B.1.LHS because it might be used in a later RHS
//  // You can define a partial ordering on the sub-scopes:
//  // SCOPE B.1.LHS <= SCOPE B.1.RHS
//  // SCOPE B.1.LHS <= SCOPE B.2.LHS
//  // SCOPE B.2.LHS <= SCOPE B.2.RHS
//  // Each scope can define a parent check scope: SCOPE B.N.RHS => SCOPE B.N.LHS and SCOPE B.N.LHS => SCOPE B.(N-1).LHS
//  // If a use is the last use on any of the reverse paths then it can be marked as a final use
//  // This can be done in O(Scopes) if we're careful
//  //-------
//  { let y = x; panic!() } => { 1 } // SCOPE B.1
//  { let y = 2; } => {x}            // SCOPE B.2
// }