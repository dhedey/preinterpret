use super::*;
use std::rc::Rc;

pub(crate) fn wrap_parser<T, E>(
    parser: impl FnOnce(SourceParser) -> Result<T, E>,
) -> impl FnOnce(ParseStream<Source>) -> Result<(T, ParseState), E> {
    move |stream: ParseStream<Source>| {
        let context = ParseContext::new();
        let state = Rc::clone(&context.full_state);
        let output = parser(&SourceParseBuffer {
            buffer: OwnedOrRef::Ref(stream),
            context,
        })?;
        let state = match Rc::into_inner(state) {
            Some(state) => state,
            None => {
                panic!("Something held onto a parser state after the parser returned");
            }
        };
        Ok((output, state))
    }
}

pub(crate) type SourceParser<'a> = &'a SourceParseBuffer<'a>;

pub(crate) struct SourceParseBuffer<'a> {
    pub(crate) buffer: OwnedOrRef<'a, ParseBuffer<'a, Source>>,
    pub(crate) context: ParseContext,
}

impl<'a> Deref for SourceParseBuffer<'a> {
    type Target = ParseBuffer<'a, Source>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

#[derive(Clone)]
pub(crate) struct ParseContext {
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

new_marker!(pub(crate) Scopes);
type ScopeId = Key<Scopes>;

new_marker!(pub(crate) Bindings);
type BindingId = Key<Bindings>;

pub(crate) struct ParseState {
    scopes: WriteOnlyArena<Scopes, ScopeData>,
    bindings: WriteOnlyArena<Bindings, BindingData>,
}

impl ParseState {
    fn new() -> (ParseState, ScopeId) {
        let mut scopes = WriteOnlyArena::new();
        let bindings = WriteOnlyArena::new();
        let root = scopes.insert(ScopeData {});
        let state = Self {
            scopes,
            bindings,
        };
        (state, root)
    }
}

struct ScopeData {

}

struct BindingData {

}
