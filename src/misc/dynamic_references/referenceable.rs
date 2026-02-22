use super::*;

pub(crate) struct Referenceable {
    core: Rc<ReferenceableCore<AnyValue>>,
}

impl Referenceable {
    pub(crate) fn new_inactive_shared(&self) -> InactiveSharedReference<AnyValue> {
        InactiveSharedReference(ReferenceCore::new_root(
            self.core.clone(),
            ReferenceKind::Inactive,
        ))
    }

    pub(crate) fn new_inactive_mutable(&self) -> InactiveMutableReference<AnyValue> {
        InactiveMutableReference(ReferenceCore::new_root(
            self.core.clone(),
            ReferenceKind::Inactive,
        ))
    }
}

pub(super) struct ReferenceableCore<T> {
    // Guaranteed not-null
    root: UnsafeCell<T>,
    root_name: String,
    pub(super) root_span: SpanRange,
    data: RefCell<ReferenceableData>,
}

impl<T> ReferenceableCore<T> {
    pub(super) fn new(root: T, root_name: String, root_span: SpanRange) -> Self {
        Self {
            root: UnsafeCell::new(root),
            root_name,
            root_span,
            data: RefCell::new(ReferenceableData {
                arena: SlotMap::with_key(),
            }),
        }
    }

    pub(super) fn root(&self) -> NonNull<T> {
        unsafe {
            // SAFETY: This is guaranteed non-null
            NonNull::new_unchecked(self.root.get())
        }
    }

    pub(super) fn data(&self) -> Ref<'_, ReferenceableData> {
        self.data.borrow()
    }

    pub(super) fn data_mut(&self) -> RefMut<'_, ReferenceableData> {
        self.data.borrow_mut()
    }

    pub(super) fn display_path(
        &self,
        mut f: impl std::fmt::Write,
        id: LocalReferenceId,
    ) -> std::fmt::Result {
        f.write_str(&self.root_name)?;
        let data = self.data();
        let data = data.for_reference(id);
        for part in data.path.parts.iter() {
            match part {
                PathPart::Value { bound_as } => {
                    f.write_str(&format!(" (of type {})", bound_as.source_name()))?;
                }
                PathPart::ArrayChild(i) => {
                    f.write_char('[')?;
                    write!(f, "{}", i)?;
                    f.write_char(']')?;
                }
                PathPart::ObjectChild(key) => {
                    if syn::parse_str::<Ident>(key).is_ok() {
                        f.write_char('.')?;
                        f.write_str(key)?;
                    } else {
                        f.write_char('[')?;
                        write!(f, "{:?}", key)?;
                        f.write_char(']')?;
                    }
                }
            }
        }
        Ok(())
    }
}

new_key_type! {
    pub(crate) struct LocalReferenceId;
}

pub(super) struct ReferenceableData {
    arena: SlotMap<LocalReferenceId, TrackedReference>,
}

impl ReferenceableData {
    pub(super) fn new_reference(&mut self, data: TrackedReference) -> LocalReferenceId {
        self.arena.insert(data)
    }

    pub(super) fn for_reference(&self, id: LocalReferenceId) -> &TrackedReference {
        self.arena.get(id).expect("reference id not found in map")
    }

    fn for_reference_mut(&mut self, id: LocalReferenceId) -> &mut TrackedReference {
        self.arena
            .get_mut(id)
            .expect("reference id not found in map")
    }

    pub(super) fn drop_reference(&mut self, id: LocalReferenceId) {
        self.arena.remove(id);
    }

    pub(super) fn deactivate_reference(&mut self, id: LocalReferenceId) {
        self.for_reference_mut(id).reference_kind = ReferenceKind::Inactive;
    }

    pub(super) fn activate_mutable_reference(
        &mut self,
        id: LocalReferenceId,
    ) -> FunctionResult<()> {
        let data = self.for_reference(id);
        // Perform checks as per the module doc on `dynamic_references`
        for (other_id, other_data) in self.arena.iter() {
            if other_id != id {
                match other_data.path.partial_cmp(&data.path) {
                    None => continue,
                    Some(Ordering::Equal | Ordering::Less) => {
                        if other_data.reference_kind.is_active() {
                            // TODO[references]: Fix this
                            todo!("// BETTER ERROR: Safety invariant break")
                        }
                    }
                    Some(Ordering::Greater) => {
                        // TODO[references]: Fix this
                        todo!("// BETTER ERROR: Validity invariant break")
                    }
                }
            }
        }
        let data = self.for_reference_mut(id);
        data.reference_kind = match data.reference_kind {
            ReferenceKind::Inactive => ReferenceKind::ActiveMutable,
            _ => panic!("cannot mut-activate an active reference"),
        };
        Ok(())
    }

    pub(super) fn activate_shared_reference(&mut self, id: LocalReferenceId) -> FunctionResult<()> {
        let data = self.for_reference(id);
        // Perform checks as per the module doc on `dynamic_references`
        for (other_id, other_data) in self.arena.iter() {
            if other_id != id && other_data.reference_kind == ReferenceKind::ActiveMutable {
                match other_data.path.partial_cmp(&data.path) {
                    Some(Ordering::Equal | Ordering::Greater) => {
                        // TODO[references]
                        todo!("// BETTER ERROR: Safety invariant break")
                    }
                    Some(Ordering::Less) => {
                        panic!("Unexpected mutability invariant break: shared-activating a reference with an active mutable parent. This should already be prevented by the mutable reference's activation checks, so this likely indicates a bug in the implementation.")
                    }
                    _ => continue,
                }
            }
        }
        let data = self.for_reference_mut(id);
        data.reference_kind = match data.reference_kind {
            ReferenceKind::Inactive => ReferenceKind::ActiveShared,
            _ => panic!("cannot shared-activate an active reference"),
        };
        Ok(())
    }

    pub(super) fn derive_reference(
        &mut self,
        id: LocalReferenceId,
        path_extension: ReferencePathExtension,
        new_span: SpanRange,
    ) {
        let data = self.for_reference_mut(id);
        data.creation_span = new_span;
        // TODO[references]: Extend the path
    }
}

#[derive(Clone)]
pub(super) struct TrackedReference {
    pub(super) path: ReferencePath,
    pub(super) reference_kind: ReferenceKind,
    pub(super) creation_span: SpanRange,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub(super) enum ReferenceKind {
    Inactive,
    ActiveShared,
    ActiveMutable,
}

impl ReferenceKind {
    fn is_active(&self) -> bool {
        match self {
            ReferenceKind::Inactive => false,
            ReferenceKind::ActiveShared | ReferenceKind::ActiveMutable => true,
        }
    }
}

/// ## Partial Order
///
/// Has a partial order defined which says all of the following:
/// - P1 and P2 are incomparable if they can exist as distinct mutable reference
/// - P1 < P2 if P2 is "deeper" than P1, i.e. that:
///   - A mutation of P1 could invalidate a reference to P2 so must be banned
///   - P1 could observe a mutation of P2, but would not invalidate it
/// - P1 == P2 if they are equivalent to the same reference, i.e. that:
///   - A mutation of P1 is possible without invalidating a reference to P2
///   - P1 could observe a mutation of P2 (and vice versa)
///
/// If any of these assumptions are wrong, we'll need to revisit this.
#[derive(PartialEq, Eq, Clone)]
pub(super) struct ReferencePath {
    parts: Vec<PathPart>,
}

impl ReferencePath {
    pub(super) fn leaf(bound_as: TypeKind) -> Self {
        Self {
            parts: vec![PathPart::Value { bound_as }],
        }
    }
}

impl PartialOrd for ReferencePath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        for (own, other) in self.parts.iter().zip(&other.parts) {
            match own.partial_cmp(other) {
                // If incomparable, the paths have diverged so are incomparable
                None => return None,
                Some(Ordering::Equal) => continue,
                Some(ordered) => return Some(ordered),
            }
        }
        Some(self.parts.len().cmp(&other.parts.len()))
    }
}

pub(crate) struct ReferencePathExtension;

#[derive(PartialEq, Eq, Clone)]
enum PathPart {
    Value { bound_as: TypeKind },
    ArrayChild(usize),
    ObjectChild(String),
}

impl PathPart {
    fn bound_type_kind(&self) -> TypeKind {
        match self {
            PathPart::Value { bound_as } => *bound_as,
            PathPart::ArrayChild(_) => ArrayType::type_kind(),
            PathPart::ObjectChild(_) => ObjectType::type_kind(),
        }
    }
}

impl PartialOrd for PathPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (PathPart::ArrayChild(i), PathPart::ArrayChild(j)) if i == j => Some(Ordering::Equal),
            (PathPart::ArrayChild(_), PathPart::ArrayChild(_)) => None,
            (PathPart::ObjectChild(a), PathPart::ObjectChild(b)) if a == b => Some(Ordering::Equal),
            (PathPart::ObjectChild(_), PathPart::ObjectChild(_)) => None,
            // TODO[references]: I'm not sure this is right
            // - A path being incomparable is a divergence
            // - A type being incomparable is:
            //  - A broken invariant if they are incompatible (e.g. String and Int)
            //  - ?? if it's compatible dyn and a concrete type (e.g. dyn Iterator and Array)
            // - So we probably don't want to make PathPart and TypeKind implement PartialOrd,
            //   we probably want a more senamtically meaningful output enum which we can handle
            //   correctly in upstream logic
            (this, other) => this.bound_type_kind().partial_cmp(&other.bound_type_kind()),
        }
    }
}
