use super::*;

pub(crate) enum QqqCopyOnWrite<L: 'static> {
    /// An owned value that can be used directly
    Owned(Owned<L>),
    /// For use when the CopyOnWrite value effectively represents the owned value (post-clone).
    /// In this case, returning a Cow is just an optimization and we can always clone infallibly.
    SharedWithInfallibleCloning(QqqShared<L>),
    /// For use when the CopyOnWrite value represents a pre-cloned read-only value.
    /// A transparent clone may fail in this case at use time.
    SharedWithTransparentCloning(QqqShared<L>),
}

impl<L: IsValueLeaf> IsValueContent for QqqCopyOnWrite<L> {
    type Type = L::Type;
    type Form = BeCopyOnWrite;
}

impl<'a, L: IsValueLeaf> IntoValueContent<'a> for QqqCopyOnWrite<L> {
    fn into_content(self) -> Content<'a, Self::Type, Self::Form> {
        self
    }
}

impl<'a, L: IsValueLeaf> FromValueContent<'a> for QqqCopyOnWrite<L> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self {
        content
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BeCopyOnWrite;
impl IsForm for BeCopyOnWrite {}

impl IsHierarchicalForm for BeCopyOnWrite {
    type Leaf<'a, T: IsLeafType> = QqqCopyOnWrite<T::Leaf>;

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
                QqqCopyOnWrite::Owned(leaf)
            }
        }
    }

    pub(crate) fn new_shared_in_place_of_owned<'a, C: IntoValueContent<'a, Form = BeShared>>(
        shared: C,
    ) -> Content<'a, C::Type, BeCopyOnWrite> {
        map_via_leaf! {
            input: (Content<'a, C::Type, BeShared>) = shared.into_content(),
            fn map_leaf<F = BeShared, T>(leaf) -> (Content<'a, T, BeCopyOnWrite>) {
                QqqCopyOnWrite::SharedWithInfallibleCloning(leaf)
            }
        }
    }

    pub(crate) fn new_shared_in_place_of_shared<'a, C: IntoValueContent<'a, Form = BeShared>>(
        shared: C,
    ) -> Content<'a, C::Type, BeCopyOnWrite> {
        map_via_leaf! {
            input: (Content<'a, C::Type, BeShared>) = shared.into_content(),
            fn map_leaf<F = BeShared, T>(leaf) -> (Content<'a, T, BeCopyOnWrite>) {
                QqqCopyOnWrite::SharedWithTransparentCloning(leaf)
            }
        }
    }
}

impl MapFromArgument for BeCopyOnWrite {
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::CopyOnWrite;

    fn from_argument_value(
        Spanned(value, span): Spanned<ArgumentValue>,
    ) -> FunctionResult<Content<'static, AnyType, Self>> {
        match value.expect_copy_on_write().inner {
            CopyOnWriteInner::Owned(owned) => Ok(BeCopyOnWrite::new_owned(owned)),
            CopyOnWriteInner::SharedWithInfallibleCloning(shared) => {
                Ok(BeCopyOnWrite::new_shared_in_place_of_owned(shared))
            }
            CopyOnWriteInner::SharedWithTransparentCloning(shared) => {
                Ok(BeCopyOnWrite::new_shared_in_place_of_shared(shared))
            }
        }
    }
}

impl LeafAsRefForm for BeCopyOnWrite {
    fn leaf_as_ref<'r, 'a: 'r, T: IsLeafType>(leaf: &'r Self::Leaf<'a, T>) -> &'r T::Leaf {
        match leaf {
            QqqCopyOnWrite::Owned(owned) => owned,
            QqqCopyOnWrite::SharedWithInfallibleCloning(shared) => shared,
            QqqCopyOnWrite::SharedWithTransparentCloning(shared) => shared,
        }
    }

    fn leaf_clone_to_owned_infallible<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
    ) -> T::Leaf {
        match leaf {
            QqqCopyOnWrite::Owned(owned) => owned.clone(),
            QqqCopyOnWrite::SharedWithInfallibleCloning(shared) => {
                BeShared::leaf_clone_to_owned_infallible::<T>(shared)
            }
            QqqCopyOnWrite::SharedWithTransparentCloning(shared) => {
                BeShared::leaf_clone_to_owned_infallible::<T>(shared)
            }
        }
    }

    fn leaf_clone_to_owned_transparently<'r, 'a: 'r, T: IsLeafType>(
        leaf: &'r Self::Leaf<'a, T>,
        error_span: SpanRange,
    ) -> ExecutionResult<T::Leaf> {
        match leaf {
            QqqCopyOnWrite::Owned(owned) => Ok(owned.clone()),
            QqqCopyOnWrite::SharedWithInfallibleCloning(shared) => {
                Ok(BeShared::leaf_clone_to_owned_infallible::<T>(shared))
            }
            QqqCopyOnWrite::SharedWithTransparentCloning(shared) => {
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
                    QqqCopyOnWrite::Owned(owned) => AnyLevelCopyOnWrite::Owned(owned),
                    QqqCopyOnWrite::SharedWithInfallibleCloning(shared) => AnyLevelCopyOnWrite::SharedWithInfallibleCloning(shared),
                    QqqCopyOnWrite::SharedWithTransparentCloning(shared) => AnyLevelCopyOnWrite::SharedWithTransparentCloning(shared),
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
                    QqqCopyOnWrite::Owned(owned) => owned,
                    QqqCopyOnWrite::SharedWithInfallibleCloning(shared) => BeShared::leaf_clone_to_owned_infallible::<T>(&shared),
                    QqqCopyOnWrite::SharedWithTransparentCloning(shared) => BeShared::leaf_clone_to_owned_infallible::<T>(&shared),
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
                    QqqCopyOnWrite::Owned(owned) => owned,
                    QqqCopyOnWrite::SharedWithInfallibleCloning(shared) => BeShared::leaf_clone_to_owned_infallible::<T>(&shared),
                    QqqCopyOnWrite::SharedWithTransparentCloning(shared) => BeShared::leaf_clone_to_owned_transparently::<T>(&shared, error_span)?,
                })
            }
        }
    }

    fn acts_as_shared_reference(&self) -> bool {
        map_via_leaf! {
            input: &'r (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F = BeCopyOnWrite, T>(leaf) -> (bool) {
                match leaf {
                    QqqCopyOnWrite::Owned(_) => false,
                    QqqCopyOnWrite::SharedWithInfallibleCloning(_) => false,
                    QqqCopyOnWrite::SharedWithTransparentCloning(_) => true,
                }
            }
        }
    }

    // TODO - Find alternative implementation or replace
    fn map<M>(self, mapper: M) -> ExecutionResult<Content<'static, M::TTo, BeCopyOnWrite>>
    where
        'a: 'static,
        M: OwnedTypeMapper + RefTypeMapper,
        M: TypeMapper<TFrom = Self::Type, TTo = AnyType>, // u32 is temporary to see if we can make it work without adding generics to `map_via_leaf!`
        Self: Sized,
        Self: IsValueContent<Type = AnyType>, // Temporary to see if we can make it work without adding generics to `map_via_leaf!`
    {
        Ok(match self.into_any_level_copy_on_write() {
            AnyLevelCopyOnWrite::Owned(owned) => BeCopyOnWrite::new_owned(mapper.map_owned(owned)?),
            AnyLevelCopyOnWrite::SharedWithInfallibleCloning(shared) => {
                // // Apply map, get Shared<Content<'a, M::TTo, BeOwned>>
                // let shared = map_via_leaf! {
                //     input: (Content<'a, Self::Type, BeShared>) = shared,
                //     fn map_leaf<F = BeShared, T>(leaf) -> (ExecutionResult<QqqShared<Content<'static, AnyType, BeOwned>>>) {
                //         leaf.try_map(|value_ref| {
                //             let from_ref = value_ref.into_any();
                //             inner_map_ref(from_ref) // &Content<M::TTo, BeOwned>

                //             // If inner_map_ref returned a Content<'a, M::TTo, BeRef>
                //             // then we'd need to move the leaf map inside here, but it'd work the same
                //         })
                //     }
                // }?;
                // // Migrate Shared into leaf
                // let shared_content = shared.replace(|content, emplacer| {
                //     // TODO - use two lifetimes here
                //     // ----
                //     map_via_leaf! {
                //         input: &'r (Content<'a, AnyType, BeOwned>) = content,
                //         state: | <'e> SharedEmplacer<'e, AnyValue, AnyValue> | let emplacer = emplacer,
                //         fn map_leaf<F = BeOwned, T>(leaf) -> (Content<'static, T, BeShared>) {
                //             emplacer.emplace(leaf)
                //         }
                //     }
                // });
                // BeCopyOnWrite::new_shared_in_place_of_owned::<M::TTo>(shared_content)
                todo!()
            }
            AnyLevelCopyOnWrite::SharedWithTransparentCloning(shared) => {
                todo!()
            }
        })
    }
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
pub(crate) trait OwnedTypeMapper: TypeMapper {
    fn map_owned<'a>(
        self,
        owned_value: Content<'a, Self::TFrom, BeOwned>,
    ) -> ExecutionResult<Content<'a, Self::TTo, BeOwned>>;
}

pub(crate) trait RefTypeMapper: TypeMapper {
    fn map_ref<'a>(
        self,
        ref_value: Content<'a, Self::TFrom, BeRef>,
    ) -> ExecutionResult<&'a Content<'a, Self::TTo, BeOwned>>;
}

pub(crate) trait MutTypeMapper: TypeMapper {
    fn map_mut<'a>(
        self,
        mut_value: Content<'a, Self::TFrom, BeMut>,
    ) -> ExecutionResult<&'a mut Content<'a, Self::TTo, BeOwned>>;
}

impl<'a, C: IsSelfValueContent<'a>> IsSelfCopyOnWriteContent<'a> for C
where
    Self: IsValueContent<Form = BeCopyOnWrite>,
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
{
}
