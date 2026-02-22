use super::*;

#[derive(Clone)]
pub(crate) struct Referenceable {
    core: Rc<ReferenceableCore<AnyValue>>,
}

impl Referenceable {
    pub(crate) fn new(root: AnyValue, root_name: String, root_span: SpanRange) -> Self {
        Self {
            core: Rc::new(ReferenceableCore::new(root, root_name, root_span)),
        }
    }

    pub(crate) fn new_inactive_shared(&self) -> InactiveSharedReference<AnyValue> {
        InactiveSharedReference(ReferenceCore::new_root(
            self.core.clone(),
            ReferenceKind::InactiveShared,
        ))
    }

    pub(crate) fn new_inactive_mutable(&self) -> InactiveMutableReference<AnyValue> {
        InactiveMutableReference(ReferenceCore::new_root(
            self.core.clone(),
            ReferenceKind::InactiveMutable,
        ))
    }

    /// Attempts to unwrap the inner value if this is the sole owner (no outstanding references).
    pub(crate) fn try_into_inner(self) -> Result<AnyValue, Self> {
        match Rc::try_unwrap(self.core) {
            Ok(core) => Ok(core.into_inner()),
            Err(core) => Err(Self { core }),
        }
    }
}

pub(super) struct ReferenceableCore<T> {
    // Guaranteed not-null
    root: UnsafeCell<T>,
    data: RefCell<ReferenceableData>,
}

impl<T> ReferenceableCore<T> {
    pub(super) fn new(root: T, root_name: String, root_span: SpanRange) -> Self {
        Self {
            root: UnsafeCell::new(root),
            data: RefCell::new(ReferenceableData {
                root_name,
                root_span,
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

    pub(super) fn into_inner(self) -> T {
        self.root.into_inner()
    }
}

new_key_type! {
    pub(crate) struct LocalReferenceId;
}

pub(super) struct ReferenceableData {
    // Could be in Referencable Core, but having it here makes the error message API easier
    root_name: String,
    // Could be in Referencable Core, but having it here makes the error message API easier
    root_span: SpanRange,
    arena: SlotMap<LocalReferenceId, TrackedReference>,
}

impl ReferenceableData {
    pub(super) fn root_span(&self) -> SpanRange {
        self.root_span
    }

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
        let data = self.for_reference_mut(id);
        data.reference_kind = match data.reference_kind {
            ReferenceKind::ActiveMutable => ReferenceKind::InactiveMutable,
            ReferenceKind::ActiveShared => ReferenceKind::InactiveShared,
            _ => panic!("cannot deactivate an inactive reference"),
        }
    }

    pub(super) fn make_shared(&mut self, id: LocalReferenceId) {
        let data = self.for_reference_mut(id);
        data.reference_kind = match data.reference_kind {
            ReferenceKind::InactiveMutable | ReferenceKind::InactiveShared => {
                ReferenceKind::InactiveShared
            }
            ReferenceKind::ActiveMutable | ReferenceKind::ActiveShared => {
                ReferenceKind::ActiveShared
            }
        }
    }

    pub(super) fn activate_mutable_reference(
        &mut self,
        id: LocalReferenceId,
    ) -> FunctionResult<()> {
        let data = self.for_reference(id);
        // Perform checks as per the module doc on `dynamic_references`
        for (other_id, other_data) in self.arena.iter() {
            if other_id != id {
                let error_reason = data
                    .path
                    .compare(&other_data.path)
                    .error_comparing_mutable_with_other(other_data.reference_kind.is_active());
                let error_reason = match error_reason {
                    Some(reason) => reason,
                    None => continue,
                };
                let mut error_message =
                    "Cannot create mutable reference because it clashes with another reference: "
                        .to_string();
                error_message.push_str(error_reason);
                let _ = write!(error_message, "This reference-: ");
                self.display_path(&mut error_message, id, Some(ReferenceKind::ActiveMutable));
                let _ = write!(error_message, "Other reference: ");
                self.display_path(&mut error_message, other_id, None);
                return data.creation_span.ownership_err(error_message);
            }
        }

        let data = self.for_reference_mut(id);
        data.reference_kind = match data.reference_kind {
            ReferenceKind::InactiveMutable => ReferenceKind::ActiveMutable,
            ReferenceKind::InactiveShared => {
                panic!("cannot mut-activate an inactive shared reference")
            }
            _ => panic!("cannot mut-activate an active reference"),
        };

        Ok(())
    }

    pub(super) fn activate_shared_reference(&mut self, id: LocalReferenceId) -> FunctionResult<()> {
        let data = self.for_reference(id);
        // Perform checks as per the module doc on `dynamic_references`
        for (other_id, other_data) in self.arena.iter() {
            if other_id != id && other_data.reference_kind == ReferenceKind::ActiveMutable {
                let error_reason = other_data
                    .path
                    .compare(&data.path)
                    .error_comparing_mutable_with_other(true);
                let error_reason = match error_reason {
                    Some(reason) => reason,
                    None => continue,
                };
                let mut error_message =
                    "Cannot create shared reference because it clashes with a mutable reference: "
                        .to_string();
                error_message.push_str(error_reason);
                let _ = write!(error_message, "This reference-: ");
                self.display_path(&mut error_message, id, Some(ReferenceKind::ActiveShared));
                let _ = write!(error_message, "Other reference: ");
                self.display_path(&mut error_message, other_id, None);
                return data.creation_span.ownership_err(error_message);
            }
        }
        let data = self.for_reference_mut(id);
        data.reference_kind = match data.reference_kind {
            ReferenceKind::InactiveShared | ReferenceKind::InactiveMutable => {
                ReferenceKind::ActiveShared
            }
            _ => panic!("cannot shared-activate an active reference"),
        };
        Ok(())
    }

    pub(super) fn derive_reference(
        &mut self,
        id: LocalReferenceId,
        path_extension: PathExtension,
        new_span: SpanRange,
    ) {
        let data = self.for_reference_mut(id);
        data.creation_span = new_span;
        let last_path_part = data.path.parts.last_mut().expect("path is non-empty");
        match (last_path_part, path_extension) {
            (last_path_part, PathExtension::Child(specifier, child_bound_as)) => {
                let parent_type = specifier.bound_type_kind();
                match last_path_part {
                    PathPart::Value { bound_as } => {
                        if !bound_as.is_tightening_to(&parent_type) {
                            panic!(
                                "Invalid path extension: cannot derive {} from {}",
                                parent_type.source_name(),
                                bound_as.source_name()
                            );
                        }
                    }
                    PathPart::Child(_) => {
                        panic!("Invalid path extension: paths are expected to end in a value")
                    }
                }
                *last_path_part = PathPart::Child(specifier);
                data.path.parts.push(PathPart::Value {
                    bound_as: child_bound_as,
                });
            }
            (PathPart::Value { bound_as }, PathExtension::Tightened(new_bound_as)) => {
                if bound_as.is_tightening_to(&new_bound_as) {
                    *bound_as = new_bound_as;
                } else {
                    panic!(
                        "Invalid path extension: cannot derive {} from {}",
                        new_bound_as.source_name(),
                        bound_as.source_name()
                    );
                }
            }
            (PathPart::Child(_), _) => {
                panic!("Invalid path extension: paths are expected to end in a value");
            }
        }
    }

    fn display_path(
        &self,
        mut f: impl std::fmt::Write,
        id: LocalReferenceId,
        override_kind: Option<ReferenceKind>,
    ) -> std::fmt::Result {
        let data = self.for_reference(id);
        let kind = override_kind.unwrap_or(data.reference_kind);
        match kind {
            // These are the same width to allow alignment in the error message
            ReferenceKind::InactiveShared => f.write_str("[inactive]     &")?,
            ReferenceKind::InactiveMutable => f.write_str("[inactive] &mut ")?,
            ReferenceKind::ActiveShared => f.write_str("[*active*]     &")?,
            ReferenceKind::ActiveMutable => f.write_str("[*active*] &mut ")?,
        }
        f.write_str(&self.root_name)?;
        for part in data.path.parts.iter() {
            match part {
                PathPart::Value { bound_as } => {
                    f.write_str(&format!(" (of type {})", bound_as.source_name()))?;
                }
                PathPart::Child(child) => match child {
                    ChildSpecifier::ArrayChild(i) => {
                        f.write_char('[')?;
                        write!(f, "{}", i)?;
                        f.write_char(']')?;
                    }
                    ChildSpecifier::ObjectChild(key) => {
                        if syn::parse_str::<Ident>(key).is_ok() {
                            f.write_char('.')?;
                            f.write_str(key)?;
                        } else {
                            f.write_char('[')?;
                            write!(f, "{:?}", key)?;
                            f.write_char(']')?;
                        }
                    }
                },
            }
        }
        Ok(())
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
    InactiveShared,
    InactiveMutable,
    ActiveShared,
    ActiveMutable,
}

impl ReferenceKind {
    fn is_active(&self) -> bool {
        match self {
            ReferenceKind::InactiveShared | ReferenceKind::InactiveMutable => false,
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

enum PathComparison {
    /// The paths have diverged in a manner which permits mutual mutability
    /// e.g. left = x.a and right = x.b
    Divergent,
    /// The paths parts overlap in a manner which forbids mutual mutability,
    /// but they are not identical or in an ancestor/descendant relationship.
    /// e.g. left = x[0..10], right = x[5..15]
    Overlapping,
    /// The right path is a descendent of the left path.
    /// e.g. right = left.x or right = left[0]["key"]
    RightIsDescendent,
    /// The left path is a descendent of the right path.
    /// e.g. left = right.x or left = right[0]["key"]
    LeftIsDescendent,
    /// The left and right path refer to the same leaf value.
    /// But they may be in different forms. e.g. left = &any and right = &integer
    ReferencesEqual(TypeBindingComparison),
    /// Represents an impossible comparison which indicates a broken invariant
    /// (e.g. left: &integer and right: &string)
    Incompatible,
}

impl PathComparison {
    fn error_comparing_mutable_with_other(self, other_is_active: bool) -> Option<&'static str> {
        Some(match self {
            PathComparison::Divergent => return None,
            PathComparison::Overlapping => {
                "they overlap, so mutation may invalidate the other reference"
            }
            PathComparison::RightIsDescendent => {
                "mutation may invalidate the other descendent reference"
            }
            PathComparison::ReferencesEqual(TypeBindingComparison::RightDerivesFromLeft) => {
                "mutation may invalidate the other reference with more specific type"
            }
            PathComparison::ReferencesEqual(TypeBindingComparison::Incomparable) => {
                "mutation may invalidate the other reference with an incompatible type"
            }
            // Activated reference is descendent of existing reference
            PathComparison::ReferencesEqual(TypeBindingComparison::Equal)
            | PathComparison::ReferencesEqual(TypeBindingComparison::LeftDerivesFromRight)
            | PathComparison::LeftIsDescendent => {
                if other_is_active {
                    "the mutable reference is observable from the other reference, which breaks aliasing rules"
                } else {
                    return None;
                }
            }
            PathComparison::ReferencesEqual(TypeBindingComparison::Incompatible) => {
                panic!("Unexpected incompatible type comparison. This indicates a bug in preinterpret.")
            }
            PathComparison::Incompatible => {
                panic!("Unexpected incompatible reference comparison. This indicates a bug in preinterpret.")
            }
        })
    }
}

impl ReferencePath {
    pub(super) fn leaf(bound_as: TypeKind) -> Self {
        Self {
            parts: vec![PathPart::Value { bound_as }],
        }
    }

    fn compare(&self, other: &Self) -> PathComparison {
        for (own, other) in self.parts.iter().zip(&other.parts) {
            return match own.compare(other) {
                PathPartComparison::Divergent => PathComparison::Divergent,
                PathPartComparison::IdenticalChildReference => continue,
                PathPartComparison::OverlappingChildReference => PathComparison::Overlapping,
                PathPartComparison::RightIsDescendent => PathComparison::RightIsDescendent,
                PathPartComparison::LeftIsDescendent => PathComparison::LeftIsDescendent,
                PathPartComparison::ReferencesEqual(inner) => {
                    PathComparison::ReferencesEqual(inner)
                }
                PathPartComparison::Incompatible => PathComparison::Incompatible,
            };
        }
        unreachable!("BUG: PathParts should be [Child* Value] and so can't end with a comparison of PathPartComparison::IdenticalDeeperReference")
    }
}

pub(crate) enum PathExtension {
    /// Extends the path with a child reference (e.g. .x or [0])
    Child(ChildSpecifier, TypeKind),
    /// Extends the path with a value of a certain type (e.g. dereferencing a pointer)
    Tightened(TypeKind),
}

#[derive(PartialEq, Eq, Clone)]
enum PathPart {
    Value { bound_as: TypeKind },
    Child(ChildSpecifier),
}

#[derive(PartialEq, Eq, Clone)]
pub(crate) enum ChildSpecifier {
    ArrayChild(usize),
    ObjectChild(String),
}

impl ChildSpecifier {
    fn bound_type_kind(&self) -> TypeKind {
        match self {
            ChildSpecifier::ArrayChild(_) => ArrayType::type_kind(),
            ChildSpecifier::ObjectChild(_) => ObjectType::type_kind(),
        }
    }
}

enum PathPartComparison {
    /// The paths have diverged in a manner which permits mutual mutability
    /// e.g. left = x.a and right = x.b
    Divergent,
    /// The path parts match.
    /// e.g. left = root.x.?? and right = root.x.??
    IdenticalChildReference,
    /// The paths parts overlap in a manner which forbids mutual mutability,
    /// but they are not identical or in an ancestor/descendant relationship.
    /// e.g. left = x[0..10], right = x[5..15]
    #[allow(unused)] // Kept for future, and to ensure we have the correct abstraction
    OverlappingChildReference,
    /// The right path is a descendent of the left path.
    /// e.g. right = left.x or right = left[0]["key"]
    RightIsDescendent,
    /// The left path is a descendent of the right path.
    /// e.g. left = right.x or left = right[0]["key"]
    LeftIsDescendent,
    /// The left and right path refer to the same leaf value.
    /// But they may be in different forms. e.g. left = &any and right = &integer
    ReferencesEqual(TypeBindingComparison),
    /// Represents an impossible comparison which indicates a broken invariant
    /// (e.g. left: &integer and right: &string)
    Incompatible,
}

impl PathPart {
    fn compare(&self, other: &Self) -> PathPartComparison {
        match (self, other) {
            (PathPart::Child(a), PathPart::Child(b)) => match (a, b) {
                (ChildSpecifier::ArrayChild(i), ChildSpecifier::ArrayChild(j)) if i == j => {
                    PathPartComparison::IdenticalChildReference
                }
                (ChildSpecifier::ArrayChild(_), ChildSpecifier::ArrayChild(_)) => {
                    PathPartComparison::Divergent
                }
                (ChildSpecifier::ObjectChild(a), ChildSpecifier::ObjectChild(b)) if a == b => {
                    PathPartComparison::IdenticalChildReference
                }
                (ChildSpecifier::ObjectChild(_), ChildSpecifier::ObjectChild(_)) => {
                    PathPartComparison::Divergent
                }
                _ => PathPartComparison::Incompatible,
            },
            (PathPart::Child(a), PathPart::Value { bound_as }) => {
                if a.bound_type_kind() == *bound_as {
                    PathPartComparison::LeftIsDescendent
                } else {
                    PathPartComparison::Incompatible
                }
            }
            (PathPart::Value { bound_as }, PathPart::Child(b)) => {
                if b.bound_type_kind() == *bound_as {
                    PathPartComparison::RightIsDescendent
                } else {
                    PathPartComparison::Incompatible
                }
            }
            (
                PathPart::Value { bound_as },
                PathPart::Value {
                    bound_as: other_bound_as,
                },
            ) => PathPartComparison::ReferencesEqual(bound_as.compare_bindings(other_bound_as)),
        }
    }
}
