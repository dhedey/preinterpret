use super::*;

type QqqReferenceable<T> = Actual<'static, T, BeReferenceable>;

/// Roughly equivalent to an owned, but wrapped so that it can be turned into a Shared/Mutable easily.
/// This is useful for the content of variables.
///
/// Note that Referenceable form does not support dyn casting, because Rc<RefCell<T>> cannot be
/// directly cast to Rc<RefCell<D>>.
#[derive(Copy, Clone)]
pub(crate) struct BeReferenceable;
impl IsForm for BeReferenceable {}

impl IsHierarchicalForm for BeReferenceable {
    type Leaf<'a, T: IsValueLeaf> = Rc<RefCell<T>>;
}

impl MapFromArgument for BeReferenceable {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument_value(
        _value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        // Rc::new(RefCell::new(value.expect_owned()))
        todo!()
    }
}

pub(crate) struct OwnedToReferenceableMapper;

impl LeafMapper<BeOwned> for OwnedToReferenceableMapper {
    type OutputForm = BeReferenceable;
    type ShortCircuit<'a> = Infallible;

    fn map_leaf<'a, L: IsValueLeaf>(leaf: L) -> Result<Rc<RefCell<L>>, Self::ShortCircuit<'a>> {
        Ok(Rc::new(RefCell::new(leaf)))
    }
}
