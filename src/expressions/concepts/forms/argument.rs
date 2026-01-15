use super::*;

pub(crate) enum Argument<L: IsValueLeaf> {
    Owned(L),
    CopyOnWrite(QqqCopyOnWrite<L>),
    Mutable(QqqMutable<L>),
    Assignee(QqqAssignee<L>),
    Shared(QqqShared<L>),
}

impl<L: IsValueLeaf> IsValueContent for Argument<L> {
    type Type = L::Type;
    type Form = BeArgument;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for Argument<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        <L::LeafType as IsLeafType>::leaf_to_content(self)
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for Argument<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        <L::LeafType as IsLeafType>::content_to_leaf(content)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeArgument;
impl IsForm for BeArgument {}

impl IsHierarchicalForm for BeArgument {
    type Leaf<'a, T: IsLeafType> = Argument<T::Leaf>;
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
