use super::*;

pub(crate) type QqqArgumentValue<T> = Actual<'static, T, BeArgument>;

pub(crate) struct BeArgument;
impl IsForm for BeArgument {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::AsIs;
}

impl IsHierarchicalForm for BeArgument {
    type Leaf<'a, T: IsValueLeaf> = ArgumentContent<T>;
}

pub(crate) enum ArgumentContent<T: IsValueLeaf> {
    Owned(T),
    CopyOnWrite(CopyOnWriteContent<T, T>),
    Mutable(MutableSubRcRefCell<Value, T>),
    Assignee(MutableSubRcRefCell<Value, T>),
    Shared(SharedSubRcRefCell<Value, T>),
}
