use super::*;

pub(crate) enum CopyOnWrite<L: 'static> {
    /// An owned value that can be used directly
    Owned(Owned<L>),
    /// For use when the CopyOnWrite value effectively represents the owned value (post-clone).
    /// In this case, returning a Cow is just an optimization and we can always clone infallibly.
    SharedWithInfallibleCloning(Shared<L>),
    /// For use when the CopyOnWrite value represents a pre-cloned read-only value.
    /// A transparent clone may fail in this case at use time.
    SharedWithTransparentCloning(Shared<L>),
}
impl<X: IsValueContent> IsValueContent for CopyOnWrite<X> {
    type Type = X::Type;
    type Form = BeCopyOnWrite;
}

impl<'a, X: Clone + IsValueContent> IntoValueContent<'a> for CopyOnWrite<X>
where
    X: 'static,
    X::Type: IsHierarchicalType<Content<'static, X::Form> = X>,
    X: IsValueContent<Form = BeOwned> + IsSelfValueContent<'static> + for<'b> IntoValueContent<'b>,
{
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        match self {
            CopyOnWrite::Owned(owned) => BeCopyOnWrite::new_owned(owned),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => {
                BeCopyOnWrite::new_shared_in_place_of_owned(shared)
            }
            CopyOnWrite::SharedWithTransparentCloning(shared) => {
                BeCopyOnWrite::new_shared_in_place_of_shared(shared)
            }
        }
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for CopyOnWrite<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

impl<T: 'static> CopyOnWrite<T> {
    pub(crate) fn shared_in_place_of_owned(shared: Shared<T>) -> Self {
        CopyOnWrite::SharedWithInfallibleCloning(shared)
    }

    pub(crate) fn shared_in_place_of_shared(shared: Shared<T>) -> Self {
        CopyOnWrite::SharedWithTransparentCloning(shared)
    }

    pub(crate) fn owned(owned: Owned<T>) -> Self {
        CopyOnWrite::Owned(owned)
    }

    pub(crate) fn acts_as_shared_reference(&self) -> bool {
        match &self {
            CopyOnWrite::Owned { .. } => false,
            CopyOnWrite::SharedWithInfallibleCloning { .. } => false,
            CopyOnWrite::SharedWithTransparentCloning { .. } => true,
        }
    }

    pub(crate) fn map<S>(
        self,
        map_shared: impl FnOnce(Shared<T>) -> FunctionResult<Shared<S>>,
        map_owned: impl FnOnce(Owned<T>) -> FunctionResult<Owned<S>>,
    ) -> FunctionResult<CopyOnWrite<S>> {
        Ok(match self {
            CopyOnWrite::Owned(owned) => CopyOnWrite::Owned(map_owned(owned)?),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => {
                CopyOnWrite::SharedWithInfallibleCloning(map_shared(shared)?)
            }
            CopyOnWrite::SharedWithTransparentCloning(shared) => {
                CopyOnWrite::SharedWithTransparentCloning(map_shared(shared)?)
            }
        })
    }

    pub(crate) fn map_into<U>(
        self,
        map_shared: impl FnOnce(Shared<T>) -> U,
        map_owned: impl FnOnce(Owned<T>) -> U,
    ) -> U {
        match self {
            CopyOnWrite::Owned(owned) => map_owned(owned),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => map_shared(shared),
            CopyOnWrite::SharedWithTransparentCloning(shared) => map_shared(shared),
        }
    }

    /// Deactivates this copy-on-write value, releasing any borrow.
    /// Returns a `InactiveCopyOnWrite` which can be cloned and later re-activated.
    pub(crate) fn deactivate(self) -> InactiveCopyOnWrite<T> {
        match self {
            CopyOnWrite::Owned(owned) => InactiveCopyOnWrite::Owned(owned),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => {
                InactiveCopyOnWrite::SharedWithInfallibleCloning(shared.deactivate())
            }
            CopyOnWrite::SharedWithTransparentCloning(shared) => {
                InactiveCopyOnWrite::SharedWithTransparentCloning(shared.deactivate())
            }
        }
    }
}

impl CopyOnWrite<AnyValue> {
    /// Converts to shared reference
    pub(crate) fn into_shared(self, span: SpanRange) -> Shared<AnyValue> {
        match self {
            CopyOnWrite::Owned(owned) => Shared::new_from_owned(owned, None, span),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => shared,
            CopyOnWrite::SharedWithTransparentCloning(shared) => shared,
        }
    }

    /// Converts to owned, using transparent clone for shared values where cloning was not requested
    /// Note - there's a more general `IsSelfValueContent::clone_to_owned_transparently`
    /// but this is re-implemented here as a specialization for efficiency
    pub(crate) fn clone_to_owned_transparently(
        self,
        span: SpanRange,
    ) -> FunctionResult<AnyValueOwned> {
        match self {
            CopyOnWrite::Owned(owned) => Ok(owned),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => Ok(shared.infallible_clone()),
            CopyOnWrite::SharedWithTransparentCloning(shared) => {
                let value = shared.as_ref().try_transparent_clone(span)?;
                Ok(value)
            }
        }
    }
}

/// A disabled copy-on-write value that can be safely cloned and dropped.
pub(crate) enum InactiveCopyOnWrite<T: 'static> {
    Owned(Owned<T>),
    SharedWithInfallibleCloning(InactiveShared<T>),
    SharedWithTransparentCloning(InactiveShared<T>),
}

impl<T: 'static + Clone> Clone for InactiveCopyOnWrite<T> {
    fn clone(&self) -> Self {
        match self {
            InactiveCopyOnWrite::Owned(owned) => InactiveCopyOnWrite::Owned(owned.clone()),
            InactiveCopyOnWrite::SharedWithInfallibleCloning(shared) => {
                InactiveCopyOnWrite::SharedWithInfallibleCloning(shared.clone())
            }
            InactiveCopyOnWrite::SharedWithTransparentCloning(shared) => {
                InactiveCopyOnWrite::SharedWithTransparentCloning(shared.clone())
            }
        }
    }
}

impl<T: 'static> InactiveCopyOnWrite<T> {
    /// Re-activates this inactive copy-on-write value by re-acquiring any borrow.
    pub(crate) fn activate(self, span: SpanRange) -> FunctionResult<CopyOnWrite<T>> {
        Ok(match self {
            InactiveCopyOnWrite::Owned(owned) => CopyOnWrite::Owned(owned),
            InactiveCopyOnWrite::SharedWithInfallibleCloning(inactive) => {
                CopyOnWrite::SharedWithInfallibleCloning(inactive.activate(span)?)
            }
            InactiveCopyOnWrite::SharedWithTransparentCloning(inactive) => {
                CopyOnWrite::SharedWithTransparentCloning(inactive.activate(span)?)
            }
        })
    }
}

impl<T> AsRef<T> for CopyOnWrite<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> Deref for CopyOnWrite<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            CopyOnWrite::Owned(owned) => owned,
            CopyOnWrite::SharedWithInfallibleCloning(shared) => shared.as_ref(),
            CopyOnWrite::SharedWithTransparentCloning(shared) => shared.as_ref(),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeCopyOnWrite;
impl IsForm for BeCopyOnWrite {}

impl IsHierarchicalForm for BeCopyOnWrite {
    type Leaf<'a, T: IsLeafType> = CopyOnWrite<T::Leaf>;

    #[inline]
    fn covariant_leaf<'a, 'b, T: IsLeafType>(leaf: Self::Leaf<'a, T>) -> Self::Leaf<'b, T>
    where
        'a: 'b,
    {
        leaf
    }
}

impl BeCopyOnWrite {
    pub(crate) fn new_owned<'a, C: IntoValueContent<'a, Form = BeOwned>>(
        owned: C,
    ) -> Content<'a, C::Type, BeCopyOnWrite> {
        map_via_leaf! {
            input: (Content<'a, C::Type, BeOwned>) = owned.into_content(),
            fn map_leaf<F = BeOwned, T>(leaf) -> (Content<'a, T, BeCopyOnWrite>) {
                CopyOnWrite::Owned(leaf)
            }
        }
    }

    pub(crate) fn new_shared_in_place_of_owned<'a, C: IntoValueContent<'a, Form = BeShared>>(
        shared: C,
    ) -> Content<'a, C::Type, BeCopyOnWrite> {
        map_via_leaf! {
            input: (Content<'a, C::Type, BeShared>) = shared.into_content(),
            fn map_leaf<F = BeShared, T>(leaf) -> (Content<'a, T, BeCopyOnWrite>) {
                CopyOnWrite::SharedWithInfallibleCloning(leaf)
            }
        }
    }

    pub(crate) fn new_shared_in_place_of_shared<'a, C: IntoValueContent<'a, Form = BeShared>>(
        shared: C,
    ) -> Content<'a, C::Type, BeCopyOnWrite> {
        map_via_leaf! {
            input: (Content<'a, C::Type, BeShared>) = shared.into_content(),
            fn map_leaf<F = BeShared, T>(leaf) -> (Content<'a, T, BeCopyOnWrite>) {
                CopyOnWrite::SharedWithTransparentCloning(leaf)
            }
        }
    }
}

impl MapFromArgument for BeCopyOnWrite {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::CopyOnWrite;

    fn from_argument_value(
        Spanned(value, span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        match value.expect_copy_on_write() {
            CopyOnWrite::Owned(owned) => Ok(BeCopyOnWrite::new_owned(owned)),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => {
                Ok(BeCopyOnWrite::new_shared_in_place_of_owned(shared))
            }
            CopyOnWrite::SharedWithTransparentCloning(shared) => {
                Ok(BeCopyOnWrite::new_shared_in_place_of_shared(shared))
            }
        }
    }
}

impl LeafAsRefForm for BeCopyOnWrite {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        match leaf {
            CopyOnWrite::Owned(owned) => owned,
            CopyOnWrite::SharedWithInfallibleCloning(shared) => shared,
            CopyOnWrite::SharedWithTransparentCloning(shared) => shared,
        }
    }

    fn leaf_clone_to_owned_infallible<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
    ) -> T::Leaf {
        match leaf {
            CopyOnWrite::Owned(owned) => owned.clone(),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => {
                BeShared::leaf_clone_to_owned_infallible::<T>(shared)
            }
            CopyOnWrite::SharedWithTransparentCloning(shared) => {
                BeShared::leaf_clone_to_owned_infallible::<T>(shared)
            }
        }
    }

    fn leaf_clone_to_owned_transparently<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
        error_span: SpanRange,
    ) -> FunctionResult<T::Leaf> {
        match leaf {
            CopyOnWrite::Owned(owned) => Ok(owned.clone()),
            CopyOnWrite::SharedWithInfallibleCloning(shared) => {
                Ok(BeShared::leaf_clone_to_owned_infallible::<T>(shared))
            }
            CopyOnWrite::SharedWithTransparentCloning(shared) => {
                BeShared::leaf_clone_to_owned_transparently::<T>(shared, error_span)
            }
        }
    }
}

pub(crate) trait IsSelfCopyOnWriteContent<'a>: IsSelfValueContent<'a>
where
    Self: IsValueContent<Form = BeCopyOnWrite>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
    fn into_any_level_copy_on_write(self) -> AnyLevelCopyOnWrite<'a, Self::Type>
    where
        Self: Sized,
    {
        map_via_leaf! {
            input: (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F = BeCopyOnWrite, T>(leaf) -> (AnyLevelCopyOnWrite<'a, T>) {
                match leaf {
                    CopyOnWrite::Owned(owned) => AnyLevelCopyOnWrite::Owned(owned),
                    CopyOnWrite::SharedWithInfallibleCloning(shared) => AnyLevelCopyOnWrite::SharedWithInfallibleCloning(shared),
                    CopyOnWrite::SharedWithTransparentCloning(shared) => AnyLevelCopyOnWrite::SharedWithTransparentCloning(shared),
                }
            }
        }
    }

    fn into_owned_infallible(self) -> Content<'static, Self::Type, BeOwned>
    where
        Self: Sized,
    {
        map_via_leaf! {
            input: (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F = BeCopyOnWrite, T>(leaf) -> (Content<'static, T, BeOwned>) {
                match leaf {
                    CopyOnWrite::Owned(owned) => owned,
                    CopyOnWrite::SharedWithInfallibleCloning(shared) => BeShared::leaf_clone_to_owned_infallible::<T>(&shared),
                    CopyOnWrite::SharedWithTransparentCloning(shared) => BeShared::leaf_clone_to_owned_infallible::<T>(&shared),
                }
            }
        }
    }

    fn into_owned_transparently(
        self,
        error_span: SpanRange,
    ) -> ExecutionResult<Content<'static, Self::Type, BeOwned>>
    where
        Self: Sized,
    {
        map_via_leaf! {
            input: (Content<'a, Self::Type, Self::Form>) = self,
            state: SpanRange | let error_span = error_span,
            fn map_leaf<F = BeCopyOnWrite, T>(leaf) -> (ExecutionResult<Content<'static, T, BeOwned>>) {
                Ok(match leaf {
                    CopyOnWrite::Owned(owned) => owned,
                    CopyOnWrite::SharedWithInfallibleCloning(shared) => BeShared::leaf_clone_to_owned_infallible::<T>(&shared),
                    CopyOnWrite::SharedWithTransparentCloning(shared) => BeShared::leaf_clone_to_owned_transparently::<T>(&shared, error_span)?,
                })
            }
        }
    }

    fn acts_as_shared_reference(&self) -> bool {
        map_via_leaf! {
            input: &'r (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F = BeCopyOnWrite, T>(leaf) -> (bool) {
                match leaf {
                    CopyOnWrite::Owned(_) => false,
                    CopyOnWrite::SharedWithInfallibleCloning(_) => false,
                    CopyOnWrite::SharedWithTransparentCloning(_) => true,
                }
            }
        }
    }
    // // TODO - Find alternative implementation or replace
    // fn map<M>(self, mapper: M) -> ExecutionResult<Content<'static, M::TTo, BeCopyOnWrite>>
    // where
    //     'a: 'static,
    //     M: OwnedTypeMapper + RefTypeMapper,
    //     M: TypeMapper<TFrom = Self::Type, TTo = AnyType>, // u32 is temporary to see if we can make it work without adding generics to `map_via_leaf!`
    //     Self: Sized,
    //     Self: IsValueContent<Type = AnyType>, // Temporary to see if we can make it work without adding generics to `map_via_leaf!`
    // {
    //     Ok(match self.into_any_level_copy_on_write() {
    //         AnyLevelCopyOnWrite::Owned(owned) => BeCopyOnWrite::new_owned(mapper.map_owned(owned)?),
    //         AnyLevelCopyOnWrite::SharedWithInfallibleCloning(shared) => {
    //             // // Apply map, get Shared<Content<'a, M::TTo, BeOwned>>
    //             // let shared = map_via_leaf! {
    //             //     input: (Content<'a, Self::Type, BeShared>) = shared,
    //             //     fn map_leaf<F = BeShared, T>(leaf) -> (ExecutionResult<Shared<Content<'static, AnyType, BeOwned>>>) {
    //             //         leaf.try_map(|value_ref| {
    //             //             let from_ref = value_ref.into_any();
    //             //             inner_map_ref(from_ref) // &Content<M::TTo, BeOwned>

    //             //             // If inner_map_ref returned a Content<'a, M::TTo, BeRef>
    //             //             // then we'd need to move the leaf map inside here, but it'd work the same
    //             //         })
    //             //     }
    //             // }?;
    //             // // Migrate Shared into leaf
    //             // let shared_content = shared.replace(|content, emplacer| {
    //             //     // TODO - use two lifetimes here
    //             //     // ----
    //             //     map_via_leaf! {
    //             //         input: &'r (Content<'a, AnyType, BeOwned>) = content,
    //             //         state: | <'e> SharedEmplacer<'e, AnyValue, AnyValue> | let emplacer = emplacer,
    //             //         fn map_leaf<F = BeOwned, T>(leaf) -> (Content<'static, T, BeShared>) {
    //             //             emplacer.emplace(leaf)
    //             //         }
    //             //     }
    //             // });
    //             // BeCopyOnWrite::new_shared_in_place_of_owned::<M::TTo>(shared_content)
    //             todo!()
    //         }
    //         AnyLevelCopyOnWrite::SharedWithTransparentCloning(shared) => {
    //             todo!()
    //         }
    //     })
    // }
}

fn inner_map_ref<'a>(
    ref_value: Content<'a, AnyType, BeRef>,
) -> ExecutionResult<&'a Content<'static, AnyType, BeOwned>> {
    unimplemented!()
}

/// Using the leaf-based type [`Content<'a, T, BeCopyOnWrite>`] is usually preferred,
/// but in some instances (particularly around supporting old code), we may want the
/// partitioning to be at a higher level (e.g. an `Owned(OwnedValue)` or `Shared(SharedValue)`).
///
/// That is what this type represents.
pub(crate) enum AnyLevelCopyOnWrite<'a, T: IsHierarchicalType> {
    Owned(Content<'a, T, BeOwned>),
    SharedWithInfallibleCloning(Content<'a, T, BeShared>),
    SharedWithTransparentCloning(Content<'a, T, BeShared>),
}

impl<'a, T: IsHierarchicalType> AnyLevelCopyOnWrite<'a, T> {
    pub fn into_copy_on_write(self) -> Content<'a, T, BeCopyOnWrite> {
        match self {
            AnyLevelCopyOnWrite::Owned(owned) => BeCopyOnWrite::new_owned(owned),
            AnyLevelCopyOnWrite::SharedWithInfallibleCloning(shared) => {
                BeCopyOnWrite::new_shared_in_place_of_owned(shared)
            }
            AnyLevelCopyOnWrite::SharedWithTransparentCloning(shared) => {
                BeCopyOnWrite::new_shared_in_place_of_shared(shared)
            }
        }
    }
}

// TODO[concepts]: COPY ON WRITE MAPPING
// How map can work:
// - Create AnyLevelCopyOnWrite<'a, T>:
//   * AnyLevelCopyOnWrite::<'a, T>::Owned(Content<'a, T, BeOwned>)
//   * AnyLevelCopyOnWrite::<'a, T>::SharedAsX(Content<'a, T, BeShared>)
// - Use a leaf mapper to convert Content<'a, TFrom, BeCopyOnWrite> to AnyLevelCopyOnWrite::<'a, TFrom>
// - Then map the inner Content via TFrom::map_to::<TTo, Form>()
// - Then convert AnyLevelCopyOnWrite<'a, TTo> back to Content<'a, TTo, BeCopyOnWrite> via leaf mapping `Owned` / `Shared` *to* copy on write (see e.g. as_referencable)
// - Create an AnyValuePropertyAccessor and an AnyValueIndexer
pub(crate) trait TypeMapper {
    type TFrom: IsHierarchicalType;
    type TTo: IsHierarchicalType;
}
// pub(crate) trait OwnedTypeMapper: TypeMapper {
//     fn map_owned<'a>(
//         self,
//         owned_value: Content<'a, Self::TFrom, BeOwned>,
//     ) -> ExecutionResult<Content<'a, Self::TTo, BeOwned>>;
// }

// pub(crate) trait RefTypeMapper: TypeMapper {
//     fn map_ref<'a>(
//         self,
//         ref_value: Content<'a, Self::TFrom, BeRef>,
//     ) -> ExecutionResult<&'a Content<'a, Self::TTo, BeOwned>>;
// }

// pub(crate) trait MutTypeMapper: TypeMapper {
//     fn map_mut<'a>(
//         self,
//         mut_value: Content<'a, Self::TFrom, BeMut>,
//     ) -> ExecutionResult<&'a mut Content<'a, Self::TTo, BeOwned>>;
// }

impl<'a, C: IsSelfValueContent<'a>> IsSelfCopyOnWriteContent<'a> for C
where
    Self: IsValueContent<Form = BeCopyOnWrite>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
}
