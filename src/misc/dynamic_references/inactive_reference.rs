use super::*;

#[derive(Clone)]
pub(crate) struct InactiveReference<T>(ReferenceCore<T>);

impl InactiveReference<AnyValue> {
    pub(super) fn new_root(core: Rc<ReferenceableCore<AnyValue>>) -> Self {
        let new_id = core.data_mut()
            .new_reference(TrackedReference {
                path: ReferencePath::leaf(AnyType::type_kind()),
                reference_kind: ReferenceKind::Inactive,
                creation_span: core.root_span,
            });
        Self(ReferenceCore {
            pointer: core.root.get(),
            id: new_id,
            core,
        })
    }

    pub(super) fn activate_mutable(self) -> FunctionResult<MutableReference<AnyValue>> {
        self.0.activate_mutable_reference()?;
        Ok(MutableReference(self.0))
    }

    pub(super) fn activate_shared(self) -> FunctionResult<SharedReference<AnyValue>> {
        self.0.activate_shared_reference()?;
        Ok(SharedReference(self.0))
    }
}

#[derive(Clone)]
pub(crate) struct InactiveSharedReference<T>(pub(super) ReferenceCore<T>);

#[derive(Clone)]
pub(crate) struct InactiveMutableReference<T>(pub(super) ReferenceCore<T>);