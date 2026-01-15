use super::*;

#[derive(Copy, Clone)]
pub(crate) struct BeArgument;
impl IsForm for BeArgument {}

impl IsHierarchicalForm for BeArgument {
    type Leaf<'a, T: IsLeafType> = Argument<T::Leaf>;
}

pub(crate) enum Argument<T: IsValueLeaf> {
    Owned(T),
    CopyOnWrite(QqqCopyOnWrite<T>),
    Mutable(QqqMutable<T>),
    Assignee(QqqAssignee<T>),
    Shared(QqqShared<T>),
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
