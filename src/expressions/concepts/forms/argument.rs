use super::*;

pub(crate) enum Argument<L: IsValueLeaf> {
    Owned(L),
    CopyOnWrite(QqqCopyOnWrite<L>),
    Mutable(Mutable<L>),
    Assignee(Assignee<L>),
    Shared(Shared<L>),
}

impl<L: IsValueLeaf> IsValueContent for Argument<L> {
    type Type = L::Type;
    type Form = BeArgument;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for Argument<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for Argument<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeArgument;
impl IsForm for BeArgument {}

impl IsHierarchicalForm for BeArgument {
    type Leaf<'a, T: IsLeafType> = Argument<T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

impl MapFromArgument for BeArgument {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::AsIs;

    fn from_argument_value(
        Spanned(value, _span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        todo!("Argument")
    }
}
