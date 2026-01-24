use super::*;

pub(crate) type Referenceable<L> = Rc<RefCell<L>>;

impl<L: IsValueLeaf> IsValueContent for Referenceable<L> {
    type Type = L::Type;
    type Form = BeReferenceable;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for Referenceable<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for Referenceable<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

/// Roughly equivalent to an owned, but wrapped so that it can be turned into a Shared/Mutable easily.
/// This is useful for the content of variables.
///
/// Note that Referenceable form does not support dyn casting, because Rc<RefCell<T>> cannot be
/// directly cast to Rc<RefCell<D>>.
#[derive(Copy, Clone)]
pub(crate) struct BeReferenceable;
impl IsForm for BeReferenceable {}

impl IsHierarchicalForm for BeReferenceable {
    type Leaf<'a, T: IsLeafType> = Referenceable<T::Leaf>;
}
