use super::*;

pub(super) struct ReferenceCore<T> {
    pub(super) pointer: *mut T,
    pub(super) id: LocalReferenceId,
    pub(super) core: Rc<ReferenceableCore<AnyValue>>,
}

impl<T> ReferenceCore<T> {
    pub(super) fn activate_mutable_reference(&self) -> FunctionResult<()> {
        self.core.data_mut().activate_mutable_reference(self.id)
    }

    pub(super) fn activate_shared_reference(&self) -> FunctionResult<()> {
        self.core.data_mut().activate_shared_reference(self.id)
    }

    pub(super) fn deactivate_reference(&self) {
        self.core.data_mut().deactivate_reference(self.id);
    }
}

impl<T> Clone for ReferenceCore<T> {
    fn clone(&self) -> Self {
        let mut data = self.core.data_mut();
        let copied_data = data.for_reference(self.id).clone();
        if copied_data.reference_kind == ReferenceKind::ActiveMutable {
            panic!("Cannot clone a mutable reference");
        }
        Self {
            pointer: self.pointer,
            id: data.new_reference(copied_data),
            core: Rc::clone(&self.core),
        }
    }
}

impl<T> Drop for ReferenceCore<T> {
    fn drop(&mut self) {
        self.core.data_mut().drop_reference(self.id);
    }
}