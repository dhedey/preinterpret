use super::*;

/// A temporary, flexible form that can resolve to any concrete ownership type.
///
/// Sometimes, a value can be accessed, but we don't yet know *how* we need to access it.
/// In this case, we attempt to load it with the most powerful access we can have, and convert it later.
///
/// ## Example of requirement
/// For example, if we have `x[a].method()`, we first need to resolve the type of `x[a]` to know
/// whether the method needs `x[a]` to be a shared reference, mutable reference or an owned value.
///
/// So instead, we take the most powerful access we can have for `x[a]`, and convert it later.
pub(crate) enum LateBound<T: 'static> {
    /// An owned value that can be converted to any ownership type
    Owned(LateBoundOwned<T>),
    /// A copy-on-write value that can be converted to an owned value
    CopyOnWrite(CopyOnWrite<T>),
    /// A mutable reference
    Mutable(Mutable<T>),
    /// A shared reference where mutable access failed for a specific reason
    Shared(LateBoundShared<T>),
}

impl<T: 'static> LateBound<T> {
    pub(crate) fn new_shared(shared: Shared<T>, reason_not_mutable: syn::Error) -> Self {
        LateBound::Shared(LateBoundShared {
            shared,
            reason_not_mutable,
        })
    }

    /// Maps the late-bound value through the appropriate accessor function.
    ///
    /// If the mutable mapping fails with a retryable reason, we fall back to `map_shared`.
    /// This allows operations that don't need mutable access (like reading a non-existent
    /// key from an object) to still work, with the mutable error preserved as `reason_not_mutable`.
    pub(crate) fn map_any(
        self,
        map_shared: impl FnOnce(Shared<T>) -> FunctionResult<Shared<T>>,
        map_mutable: impl FnOnce(Mutable<T>) -> Result<Mutable<T>, (FunctionError, Mutable<T>)>,
        map_owned: impl FnOnce(T) -> FunctionResult<T>,
    ) -> FunctionResult<Self> {
        Ok(match self {
            LateBound::Owned(owned) => LateBound::Owned(LateBoundOwned {
                owned: map_owned(owned.owned)?,
                is_from_last_use: owned.is_from_last_use,
            }),
            LateBound::CopyOnWrite(copy_on_write) => {
                LateBound::CopyOnWrite(copy_on_write.map(map_shared, map_owned)?)
            }
            LateBound::Mutable(mutable) => match map_mutable(mutable) {
                Ok(mapped) => LateBound::Mutable(mapped),
                Err((error, recovered_mutable)) => {
                    // Check if this error can be caught for fallback to shared access
                    let reason_not_mutable = error.into_caught_mutable_map_attempt_error()?;
                    LateBound::new_shared(
                        map_shared(recovered_mutable.into_shared())?,
                        reason_not_mutable,
                    )
                }
            },
            LateBound::Shared(LateBoundShared {
                shared,
                reason_not_mutable,
            }) => LateBound::new_shared(map_shared(shared)?, reason_not_mutable),
        })
    }
}

impl Spanned<AnyValueLateBound> {
    pub(crate) fn resolve(self, ownership: ArgumentOwnership) -> FunctionResult<ArgumentValue> {
        ownership.map_from_late_bound(self)
    }
}

impl<T: 'static> Deref for LateBound<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            LateBound::Owned(owned) => &owned.owned,
            LateBound::CopyOnWrite(cow) => cow.as_ref(),
            LateBound::Mutable(mutable) => mutable.as_ref(),
            LateBound::Shared(shared) => shared.shared.as_ref(),
        }
    }
}

impl<L: IsValueLeaf> IsValueContent for LateBound<L> {
    type Type = L::Type;
    type Form = BeLateBound;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for LateBound<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for LateBound<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

/// A temporary, flexible form that can resolve to any concrete ownership type.
///
/// See [`LateBound`] for more details.
#[derive(Copy, Clone)]
pub(crate) struct BeLateBound;
impl IsForm for BeLateBound {}

impl IsHierarchicalForm for BeLateBound {
    type Leaf<'a, T: IsLeafType> = LateBound<T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

pub(crate) struct LateBoundOwned<O: 'static> {
    pub(crate) owned: O,
    pub(crate) is_from_last_use: bool,
}

/// A shared value where mutable access failed for a specific reason
pub(crate) struct LateBoundShared<T: 'static> {
    pub(crate) shared: Shared<T>,
    pub(crate) reason_not_mutable: syn::Error,
}
