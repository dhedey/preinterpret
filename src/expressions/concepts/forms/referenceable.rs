use super::*;

type QqqReferenceable<T> = Content<'static, T, BeReferenceable>;

/// Roughly equivalent to an owned, but wrapped so that it can be turned into a Shared/Mutable easily.
/// This is useful for the content of variables.
///
/// Note that Referenceable form does not support dyn casting, because Rc<RefCell<T>> cannot be
/// directly cast to Rc<RefCell<D>>.
#[derive(Copy, Clone)]
pub(crate) struct BeReferenceable;
impl IsForm for BeReferenceable {}

impl IsHierarchicalForm for BeReferenceable {
    type Leaf<'a, T: IsLeafType> = Rc<RefCell<T::Leaf>>;
}

impl MapFromArgument for BeReferenceable {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Content<'static, AnyType, Self>> {
        // Rc::new(RefCell::new(value.expect_owned()))
        todo!()
    }
}
