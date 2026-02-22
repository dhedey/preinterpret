use super::*;

pub(super) struct ReferenceCore<T: ?Sized> {
    pub(super) pointer: NonNull<T>,
    pub(super) id: LocalReferenceId,
    pub(super) core: Rc<ReferenceableCore<AnyValue>>,
}

impl ReferenceCore<AnyValue> {
    pub(super) fn new_root(
        core: Rc<ReferenceableCore<AnyValue>>,
        reference_kind: ReferenceKind,
    ) -> Self {
        let new_id = core.data_mut().new_reference(TrackedReference {
            path: ReferencePath::leaf(AnyType::type_kind()),
            reference_kind,
            creation_span: core.root_span,
        });
        Self {
            pointer: core.root(),
            id: new_id,
            core,
        }
    }
}

impl<T: ?Sized> ReferenceCore<T> {
    pub(super) fn activate_mutable_reference(&self) -> FunctionResult<()> {
        self.core.data_mut().activate_mutable_reference(self.id)
    }

    pub(super) fn activate_shared_reference(&self) -> FunctionResult<()> {
        self.core.data_mut().activate_shared_reference(self.id)
    }

    pub(super) fn deactivate_reference(&self) {
        self.core.data_mut().deactivate_reference(self.id);
    }

    pub(super) fn into_emplacer<'e>(self) -> EmplacerCore<'e, T> {
        // NOTE: The clean-up duty is delegated to the emplacer
        let reference = ManuallyDrop::new(self);
        let manual_drop = ManualDropReferenceCore {
            pointer: reference.pointer,
            id: reference.id,
            // SAFETY: read's invariants are upheld, and we are manually dropping the reference,
            // so we won't run the drop implementation that would mess with the reference count.
            core: unsafe { std::ptr::read(&reference.core) },
        };
        EmplacerCore {
            inner: Some(manual_drop),
            encapsulation_lifetime: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> Clone for ReferenceCore<T> {
    fn clone(&self) -> Self {
        let mut data = self.core.data_mut();
        let copied_data = data.for_reference(self.id).clone();
        if copied_data.reference_kind == ReferenceKind::ActiveMutable {
            panic!("Cannot clone an active mutable reference");
        }
        Self {
            pointer: self.pointer,
            id: data.new_reference(copied_data),
            core: Rc::clone(&self.core),
        }
    }
}

impl<T: ?Sized> Drop for ReferenceCore<T> {
    fn drop(&mut self) {
        self.core.data_mut().drop_reference(self.id);
    }
}

/// Essentially the same as a [ReferenceCore], but doesn't have the drop implementation.
/// Must be a temporary, whose lifetime exists only as long as the pointer is valid in the given context.
pub(super) struct ManualDropReferenceCore<T: ?Sized> {
    pub(super) pointer: NonNull<T>,
    pub(super) id: LocalReferenceId,
    pub(super) core: Rc<ReferenceableCore<AnyValue>>,
}

pub(super) struct EmplacerCore<'e, T: ?Sized> {
    inner: Option<ManualDropReferenceCore<T>>,
    encapsulation_lifetime: std::marker::PhantomData<&'e ()>,
}

impl<'e, T: ?Sized> EmplacerCore<'e, T> {
    pub(crate) fn revert(&mut self) -> ReferenceCore<T> {
        let core = self.take();
        ReferenceCore {
            pointer: core.pointer,
            id: core.id,
            core: core.core,
        }
    }

    /// SAFETY:
    /// - The caller must ensure that the pointer is derived from the original content
    /// - The caller must ensure that the ReferencePathExtension is correct
    pub(crate) unsafe fn emplace_unchecked<V: 'static + ?Sized>(
        &mut self,
        pointer: NonNull<V>,
        path_extension: ReferencePathExtension,
        new_span: SpanRange,
    ) -> ReferenceCore<V> {
        let emplacer_core = self.take();
        let id = emplacer_core.id;
        let core = emplacer_core.core;
        core.data_mut()
            .derive_reference(id, path_extension, new_span);
        ReferenceCore { pointer, id, core }
    }

    fn take(&mut self) -> ManualDropReferenceCore<T> {
        self.inner
            .take()
            .expect("You may only emplace or revert once from an emplacer")
    }
}

impl<'e, T: ?Sized> Drop for EmplacerCore<'e, T> {
    fn drop(&mut self) {
        if let Some(core) = self.inner.take() {
            // If the emplacer is still around, we need to manually drop it
            core.core.data_mut().drop_reference(core.id);
        };
    }
}
