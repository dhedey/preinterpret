use super::*;

pub(crate) type QqqArgumentValue<T> = Content<'static, T, BeArgument>;

#[derive(Copy, Clone)]
pub(crate) struct BeArgument;
impl IsForm for BeArgument {}

impl IsHierarchicalForm for BeArgument {
    type Leaf<'a, T: IsValueLeaf> = ArgumentContent<T>;
}

impl MapFromArgument for BeArgument {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::AsIs;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        todo!()
        // Ok(value)
    }
}

pub(crate) enum ArgumentContent<T: IsValueLeaf> {
    Owned(T),
    CopyOnWrite(CopyOnWriteContent<T, T>),
    Mutable(MutableSubRcRefCell<AnyValue, T>),
    Assignee(MutableSubRcRefCell<AnyValue, T>),
    Shared(SharedSubRcRefCell<AnyValue, T>),
}
