use super::*;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct ParseContext {
    #[allow(unused)]
    pub(crate) current_scope: ScopeId,
    pub(crate) full_state: Rc<ParseState>,
}

impl ParseContext {
    pub fn new() -> Self {
        let (state, root_scope) = ParseState::new();
        Self {
            current_scope: root_scope,
            full_state: Rc::new(state),
        }
    }
}

new_key!(pub(crate) ScopeId);
new_key!(pub(crate) BindingId);

pub(crate) struct ParseState {
    #[allow(unused)]
    scopes: AppendOnlyArena<ScopeId, ScopeData>,
    #[allow(unused)]
    bindings: AppendOnlyArena<BindingId, BindingData>,
}

impl ParseState {
    fn new() -> (ParseState, ScopeId) {
        let mut scopes = AppendOnlyArena::new();
        let bindings = AppendOnlyArena::new();
        let root = scopes.add(ScopeData {});
        let state = Self { scopes, bindings };
        (state, root)
    }
}

struct ScopeData {}

struct BindingData {}
