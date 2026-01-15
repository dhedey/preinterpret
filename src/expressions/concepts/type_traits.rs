use super::*;

pub(crate) trait TypeVariant {}

pub(crate) struct HierarchicalTypeVariant;
impl TypeVariant for HierarchicalTypeVariant {}

pub(crate) struct DynTypeVariant;
impl TypeVariant for DynTypeVariant {}

pub(crate) trait IsType: Sized {
    type Variant: TypeVariant;

    const SOURCE_TYPE_NAME: &'static str;
    const ARTICLED_DISPLAY_NAME: &'static str;

    fn type_kind() -> TypeKind;
    fn type_kind_from_source_name(name: &str) -> Option<TypeKind>;
}

pub(crate) trait IsHierarchicalType: IsType<Variant = HierarchicalTypeVariant> {
    type Content<'a, F: IsHierarchicalForm>: IsValueContent<Type = Self, Form = F>
        + IntoValueContent<'a>
        + FromValueContent<'a>;
    type LeafKind: IsLeafKind;

    fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
        mapper: M,
        content: Self::Content<'a, F>,
    ) -> M::Output<'a, Self>;

    fn map_ref_with<'r, 'a: 'r, F: IsHierarchicalForm, M: RefLeafMapper<F>>(
        mapper: M,
        content: &'r Self::Content<'a, F>,
    ) -> M::Output<'r, 'a, Self>;

    fn map_mut_with<'r, 'a: 'r, F: IsHierarchicalForm, M: MutLeafMapper<F>>(
        mapper: M,
        content: &'r mut Self::Content<'a, F>,
    ) -> M::Output<'r, 'a, Self>;

    fn content_to_leaf_kind<F: IsHierarchicalForm>(
        content: &Self::Content<'_, F>,
    ) -> Self::LeafKind;
}

pub(crate) trait IsLeafType:
    IsHierarchicalType
    // NOTE: We can't create a universal bound over F: IsHierarchicalForm in rust
    // If you need a generic interconversion over all such F, we'll need to reintroduce
    // e.g. a fn content_to_leaf(content) method, with body { self } in each impl.
    // For now though, listing out all the forms here is sufficient.
    + for<'a> IsHierarchicalType<Content<'a, BeOwned> = <BeOwned as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeShared> = <BeShared as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeMutable> = <BeMutable as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeRef> = <BeRef as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeMut> = <BeMut as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeAnyRef> = <BeAnyRef as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeAnyMut> = <BeAnyMut as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeReferenceable> = <BeReferenceable as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeAssignee> = <BeAssignee as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeCopyOnWrite> = <BeCopyOnWrite as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeArgument> = <BeArgument as IsHierarchicalForm>::Leaf<'a, Self>>
    + for<'a> IsHierarchicalType<Content<'a, BeLateBound> = <BeLateBound as IsHierarchicalForm>::Leaf<'a, Self>>
{
    type Leaf: IsValueLeaf<LeafType = Self>;

    fn leaf_kind() -> Self::LeafKind;
}

pub(crate) trait IsDynType: IsType<Variant = DynTypeVariant> {
    type DynContent: ?Sized + 'static;
}

pub(crate) trait UpcastTo<T: IsHierarchicalType, F: IsHierarchicalForm>:
    IsHierarchicalType
{
    fn upcast_to<'a>(content: Content<'a, Self, F>) -> Content<'a, T, F>;
}

pub(crate) trait DowncastFrom<T: IsHierarchicalType, F: IsHierarchicalForm>:
    IsHierarchicalType
{
    fn downcast_from<'a>(
        content: Content<'a, T, F>,
    ) -> Result<Content<'a, Self, F>, Content<'a, T, F>>;

    fn resolve<'a>(
        content: Content<'a, T, F>,
        span_range: SpanRange,
        resolution_target: &str,
    ) -> ExecutionResult<Content<'a, Self, F>> {
        let content = match Self::downcast_from(content) {
            Ok(c) => c,
            Err(existing) => {
                let leaf_kind = T::content_to_leaf_kind::<F>(&existing);
                return span_range.value_err(format!(
                    "{} is expected to be {}, but it is {}",
                    resolution_target,
                    Self::ARTICLED_DISPLAY_NAME,
                    leaf_kind.articled_display_name(),
                ));
            }
        };
        Ok(content)
    }
}

pub(crate) trait DynResolveFrom<T: IsHierarchicalType, F: IsHierarchicalForm + IsDynCompatibleForm>:
    IsDynType
{
    fn downcast_from<'a>(content: Content<'a, T, F>) -> Option<DynContent<'a, Self, F>>;

    fn resolve<'a>(
        content: Content<'a, T, F>,
        span_range: SpanRange,
        resolution_target: &str,
    ) -> ExecutionResult<DynContent<'a, Self, F>> {
        let leaf_kind = T::content_to_leaf_kind::<F>(&content);
        let content = match Self::downcast_from(content) {
            Some(c) => c,
            None => {
                return span_range.value_err(format!(
                    "{} is expected to be {}, but it is {}",
                    resolution_target,
                    Self::ARTICLED_DISPLAY_NAME,
                    leaf_kind.articled_display_name(),
                ))
            }
        };
        Ok(content)
    }
}

pub(crate) trait IsChildType: IsHierarchicalType {
    type ParentType: IsHierarchicalType;

    fn into_parent<'a, F: IsHierarchicalForm>(
        content: Self::Content<'a, F>,
    ) -> Content<'a, Self::ParentType, F>;

    fn from_parent<'a, F: IsHierarchicalForm>(
        content: <Self::ParentType as IsHierarchicalType>::Content<'a, F>,
    ) -> Result<Content<'a, Self, F>, Content<'a, Self::ParentType, F>>;
}

macro_rules! impl_type_feature_resolver {
    (impl TypeFeatureResolver for $type_def:ty: [$($type_defs:ty)+]) => {
        impl TypeFeatureResolver for $type_def {
            fn resolve_method(&self, method_name: &str) -> Option<MethodInterface> {
                $(
                    if let Some(method) = <$type_defs as TypeData>::resolve_own_method(method_name) {
                        return Some(method);
                    };
                )+
                None
            }

            fn resolve_unary_operation(
                &self,
                operation: &UnaryOperation,
            ) -> Option<UnaryOperationInterface> {
                $(
                    if let Some(operation) = <$type_defs as TypeData>::resolve_own_unary_operation(operation) {
                        return Some(operation);
                    };
                )+
                None
            }

            fn resolve_binary_operation(
                &self,
                operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                $(
                    if let Some(operation) = <$type_defs as TypeData>::resolve_own_binary_operation(operation) {
                        return Some(operation);
                    };
                )+
                None
            }

            fn resolve_type_property(&self, property_name: &str) -> Option<AnyValue> {
                // Purposefully doesn't resolve parents, but TBC if this is right
                <$type_def as TypeData>::resolve_type_property(property_name)
            }
        }
    };
}

pub(crate) use impl_type_feature_resolver;

macro_rules! impl_ancestor_chain_conversions {
    ($child:ty $(=> $parent:ident ($parent_content:ident :: $parent_variant:ident) $(=> $ancestor:ty)*)?) => {
        impl<F: IsHierarchicalForm> DowncastFrom<$child, F> for $child
        {
            fn downcast_from<'a>(
                content: Content<'a, Self, F>,
            ) -> Result<Content<'a, Self, F>, Content<'a, Self, F>> {
                Ok(content)
            }
        }

        impl<F: IsHierarchicalForm> UpcastTo<$child, F> for $child
        {
            fn upcast_to<'a>(
                content: Content<'a, Self, F>,
            ) -> Content<'a, Self, F> {
                content
            }
        }

        $(
            impl IsChildType for $child {
                type ParentType = $parent;

                fn into_parent<'a, F: IsHierarchicalForm>(
                    content: Self::Content<'a, F>,
                ) -> <Self::ParentType as IsHierarchicalType>::Content<'a, F> {
                    $parent_content::$parent_variant(content)
                }

                fn from_parent<'a, F: IsHierarchicalForm>(
                    content: <Self::ParentType as IsHierarchicalType>::Content<'a, F>,
                ) -> Result<Content<'a, Self, F>, Content<'a, Self::ParentType, F>> {
                    match content {
                        $parent_content::$parent_variant(i) => Ok(i),
                        other => Err(other),
                    }
                }
            }

            impl<F: IsHierarchicalForm> DowncastFrom<$parent, F> for $child
            {
                fn downcast_from<'a>(
                    content: Content<'a, $parent, F>,
                ) -> Result<Content<'a, Self, F>, Content<'a, $parent, F>> {
                    <$child as IsChildType>::from_parent(content)
                }
            }

            impl<F: IsHierarchicalForm> UpcastTo<$parent, F> for $child
            {
                fn upcast_to<'a>(
                    content: Content<'a, $child, F>,
                ) -> Content<'a, $parent, F> {
                    <$child as IsChildType>::into_parent(content)
                }
            }

            $(
                impl<F: IsHierarchicalForm> DowncastFrom<$ancestor, F> for $child {
                    fn downcast_from<'a>(
                        content: Content<'a, $ancestor, F>,
                    ) -> Result<Content<'a, $child, F>, Content<'a, $ancestor, F>> {
                        let inner = <$parent as DowncastFrom<$ancestor, F>>::downcast_from(content)?;
                        match <$child as DowncastFrom<$parent, F>>::downcast_from(inner) {
                            Ok(c) => Ok(c),
                            Err(existing) => Err(<$parent as UpcastTo<$ancestor, F>>::upcast_to(existing)),
                        }
                    }
                }

                impl<F: IsHierarchicalForm> UpcastTo<$ancestor, F> for $child {
                    fn upcast_to<'a>(
                        content: Content<'a, $child, F>,
                    ) -> Content<'a, $ancestor, F> {
                        <$parent as UpcastTo<$ancestor, F>>::upcast_to(<$child as UpcastTo<$parent, F>>::upcast_to(content))
                    }
                }
            )*
        )?
    };
}

pub(crate) use impl_ancestor_chain_conversions;

pub(crate) trait IsValueLeaf:
    'static
    // This whole creation of IsLeafValueContent and the LeafType bound is a workaround to
    // add an implied bound to IsValueLeaf that Self::Type: IsLeafType<Leaf = Self>
    // - This "use an associated type equality constraint" workaround came from a rust thread
    // related to implied bounds, which of course I can't find now.
    // - The main caveat is that order matters (i.e. I had to set Type == LeafType) and bound the
    // correct one of Type or LeafType in other places to avoid circularity and ensure the one way
    // resolution logic works correctly.
    + IsValueContent<Type = <Self as IsLeafValueContent>::LeafType, Form = BeOwned>
    + for<'a> IntoValueContent<'a>
    + for<'a> FromValueContent<'a>
    + IsLeafValueContent
    + CastDyn<dyn IsIterable>
    + Clone
{
}

pub(crate) trait IsLeafValueContent: IsValueContent {
    type LeafType: IsLeafType<Leaf = Self>;
}

impl<L: IsValueContent> IsLeafValueContent for L
where
    L::Type: IsLeafType<Leaf = L>,
{
    type LeafType = L::Type;
}

pub(crate) trait IsDynLeaf: 'static
where
    DynMapper<Self>: LeafMapper<BeOwned>,
{
    type Type: IsDynType<DynContent = Self>;
}

pub(crate) trait CastDyn<T: ?Sized> {
    fn map_boxed(self: Box<Self>) -> Option<Box<T>> {
        None
    }
    fn map_ref(&self) -> Option<&T> {
        None
    }
    fn map_mut(&mut self) -> Option<&mut T> {
        None
    }
}

pub(crate) trait HasLeafKind {
    type LeafKind: IsLeafKind;

    fn kind(&self) -> Self::LeafKind;

    fn value_kind(&self) -> AnyValueLeafKind {
        self.kind().into()
    }

    fn articled_kind(&self) -> &'static str {
        self.kind().articled_display_name()
    }
}

// TODO[concepts]: Remove when we get rid of impl_resolvable_argument_for
impl<T: HasLeafKind> HasLeafKind for &T {
    type LeafKind = T::LeafKind;

    fn kind(&self) -> Self::LeafKind {
        (**self).kind()
    }
}

// TODO[concepts]: Remove when we get rid of impl_resolvable_argument_for
impl<T: HasLeafKind> HasLeafKind for &mut T {
    type LeafKind = T::LeafKind;

    fn kind(&self) -> Self::LeafKind {
        (**self).kind()
    }
}

macro_rules! define_parent_type {
    (
        $type_def_vis:vis $type_def:ident $(=> $parent:ident($parent_content:ident :: $parent_variant:ident) $(=> $ancestor:ty)*)?,
        content: $content_vis:vis $content:ident,
        leaf_kind: $leaf_kind_vis:vis $leaf_kind:ident,
        type_kind: ParentTypeKind::$parent_kind:ident($type_kind_vis:vis $type_kind:ident),
        variants: {
            $($variant:ident => $variant_type:ty,)*
        },
        type_name: $source_type_name:literal,
        articled_display_name: $articled_display_name:literal,
    ) => {
        #[derive(Copy, Clone)]
        $type_def_vis struct $type_def;

        impl IsType for $type_def {
            type Variant = HierarchicalTypeVariant;

            const SOURCE_TYPE_NAME: &'static str = $source_type_name;
            const ARTICLED_DISPLAY_NAME: &'static str = $articled_display_name;

            fn type_kind() -> TypeKind {
                TypeKind::Parent(ParentTypeKind::$parent_kind($type_kind))
            }

            #[inline(always)]
            fn type_kind_from_source_name(name: &str) -> Option<TypeKind> {
                if name == Self::SOURCE_TYPE_NAME {
                    return Some(Self::type_kind());
                }
                $(
                    if let Some(type_kind) = <$variant_type as IsType>::type_kind_from_source_name(name) {
                        return Some(type_kind);
                    }
                )*
                None
            }
        }

        impl_type_feature_resolver! {
            impl TypeFeatureResolver for $type_def: [
                $type_def $( $parent $( $ancestor )* )?
            ]
        }

        impl IsHierarchicalType for $type_def {
            type Content<'a, F: IsHierarchicalForm> = $content<'a, F>;
            type LeafKind = $leaf_kind;

            fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
                mapper: M,
                content: Self::Content<'a, F>,
            ) -> M::Output<'a, Self> {
                match content {
                    $( $content::$variant(x) => M::to_parent_output::<'a, $variant_type>(
                        <$variant_type>::map_with::<'a, F, M>(mapper, x)
                    ), )*
                }
            }

            fn map_ref_with<'r, 'a: 'r, F: IsHierarchicalForm, M: RefLeafMapper<F>>(
                mapper: M,
                content: &'r Self::Content<'a, F>,
            ) -> M::Output<'r, 'a, Self> {
                match content {
                    $( $content::$variant(x) => M::to_parent_output::<'r, 'a, $variant_type>(
                        <$variant_type>::map_ref_with::<'r, 'a, F, M>(mapper, x)
                    ), )*
                }
            }

            fn map_mut_with<'r, 'a: 'r, F: IsHierarchicalForm, M: MutLeafMapper<F>>(
                mapper: M,
                content: &'r mut Self::Content<'a, F>,
            ) -> M::Output<'r, 'a, Self> {
                match content {
                    $( $content::$variant(x) => M::to_parent_output::<'r, 'a, $variant_type>(
                        <$variant_type>::map_mut_with::<'r, 'a, F, M>(mapper, x)
                    ), )*
                }
            }

            fn content_to_leaf_kind<F: IsHierarchicalForm>(
                content: &Self::Content<'_, F>,
            ) -> Self::LeafKind {
                content.kind()
            }
        }

        $content_vis enum $content<'a, F: IsHierarchicalForm> {
            $( $variant(Content<'a, $variant_type, F>), )*
        }

        impl<'a, F: IsHierarchicalForm> Clone for $content<'a, F>
        where
            $( Content<'a, $variant_type, F>: Clone ),*
        {
            fn clone(&self) -> Self {
                match self {
                    $( $content::$variant(x) => $content::$variant(x.clone()), )*
                }
            }
        }


        impl<'a, F: IsHierarchicalForm> Copy for $content<'a, F>
        where
            $( Content<'a, $variant_type, F>: Copy ),*
        {}

        impl_value_content_traits!(parent: $type_def, $content);

        impl<'a, F: IsHierarchicalForm> HasLeafKind for $content<'a, F> {
            type LeafKind = $leaf_kind;

            fn kind(&self) -> Self::LeafKind {
                match self {
                    $($content::$variant(x) => $leaf_kind::$variant(
                        <$variant_type as IsHierarchicalType>::content_to_leaf_kind::<F>(x)
                    ),)*
                }
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        $type_kind_vis struct $type_kind;

        impl $type_kind {
            pub(crate) fn articled_display_name(&self) -> &'static str {
                $articled_display_name
            }

            pub(crate) fn source_type_name(&self) -> &'static str {
                $source_type_name
            }

            pub(crate) fn feature_resolver(&self) -> &'static dyn TypeFeatureResolver {
                &$type_def
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        $leaf_kind_vis enum $leaf_kind {
            $( $variant(<$variant_type as IsHierarchicalType>::LeafKind), )*
        }

        $(
            impl From<$leaf_kind> for AnyValueLeafKind {
                fn from(kind: $leaf_kind) -> Self {
                    let as_parent_kind = <$parent as IsHierarchicalType>::LeafKind::$parent_variant(kind);
                    AnyValueLeafKind::from(as_parent_kind)
                }
            }
        )?

        impl IsLeafKind for $leaf_kind {
            fn source_type_name(&self) -> &'static str {
                match self {
                    $( Self::$variant(x) => x.source_type_name(), )*
                }
            }

            fn articled_display_name(&self) -> &'static str {
                match self {
                    $( Self::$variant(x) => x.articled_display_name(), )*
                }
            }

            fn feature_resolver(&self) -> &'static dyn TypeFeatureResolver {
                match self {
                    $( Self::$variant(x) => x.feature_resolver(), )*
                }
            }
        }

        impl_ancestor_chain_conversions!(
            $type_def $(=> $parent($parent_content :: $parent_variant) $(=> $ancestor)*)?
        );
    };
}

pub(crate) use define_parent_type;

macro_rules! impl_fallback_is_iterable {
    ({IsIterable $($haystack:tt)*} for $content_type:ty) => {
        // Already implemented, ignoring
    };
    ({$discard:tt $($haystack:tt)*} for $content_type:ty) => {
        // Recurse to check the rest
        impl_is_iterable_if_missing!({$($haystack)*} for $content_type)
    };
    ({} for $content_type:ty) => {
        // Implement fallback
        impl CastDyn<dyn IsIterable> for $content_type {}
    };
}

pub(crate) use impl_fallback_is_iterable;

macro_rules! define_leaf_type {
    (
        $type_def_vis:vis $type_def:ident => $parent:ident($parent_content:ident :: $parent_variant:ident) $(=> $ancestor:ty)*,
        content: $content_type:ty,
        kind: $kind_vis:vis $kind:ident,
        type_name: $source_type_name:literal,
        articled_display_name: $articled_display_name:literal,
        dyn_impls: {
            $($dyn_type:ty: impl $dyn_trait:ident { $($dyn_trait_impl:tt)* })*
        },
    ) => {
        #[derive(Copy, Clone)]
        $type_def_vis struct $type_def;

        impl IsType for $type_def {
            type Variant = HierarchicalTypeVariant;

            const SOURCE_TYPE_NAME: &'static str = $source_type_name;
            const ARTICLED_DISPLAY_NAME: &'static str = $articled_display_name;

            fn type_kind() -> TypeKind {
                TypeKind::Leaf(AnyValueLeafKind::from($kind))
            }

            // This is called for every leaf in a row as part of parsing
            // Use inline(always) to avoid function call overhead, even in Debug builds
            // which are used when macros are run during development
            #[inline(always)]
            fn type_kind_from_source_name(name: &str) -> Option<TypeKind> {
                if name == Self::SOURCE_TYPE_NAME {
                    Some(Self::type_kind())
                } else {
                    None
                }
            }
        }

        impl IsHierarchicalType for $type_def {
            type Content<'a, F: IsHierarchicalForm> = F::Leaf<'a, Self>;
            type LeafKind = $kind;

            fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
                mapper: M,
                content: Self::Content<'a, F>,
            ) -> M::Output<'a, Self> {
                mapper.map_leaf::<Self>(content)
            }

            fn map_ref_with<'r, 'a: 'r, F: IsHierarchicalForm, M: RefLeafMapper<F>>(
                mapper: M,
                content: &'r Self::Content<'a, F>,
            ) -> M::Output<'r, 'a, Self> {
                mapper.map_leaf::<Self>(content)
            }

            fn map_mut_with<'r, 'a: 'r, F: IsHierarchicalForm, M: MutLeafMapper<F>>(
                mapper: M,
                content: &'r mut Self::Content<'a, F>,
            ) -> M::Output<'r, 'a, Self> {
                mapper.map_leaf::<Self>(content)
            }

            fn content_to_leaf_kind<F: IsHierarchicalForm>(
                _content: &Self::Content<'_, F>,
            ) -> Self::LeafKind {
                $kind
            }
        }

        impl IsLeafType for $type_def {
            type Leaf = $content_type;

            fn leaf_kind() -> $kind {
                $kind
            }
        }

        impl_type_feature_resolver! {
            impl TypeFeatureResolver for $type_def: [
                $type_def $($dyn_type)* $parent $( $ancestor )*
            ]
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        $kind_vis struct $kind;

        impl From<$kind> for AnyValueLeafKind {
            fn from(kind: $kind) -> Self {
                let as_parent_kind = <$parent as IsHierarchicalType>::LeafKind::$parent_variant(kind);
                AnyValueLeafKind::from(as_parent_kind)
            }
        }

        impl IsLeafKind for $kind {
            fn source_type_name(&self) -> &'static str {
                $source_type_name
            }

            fn articled_display_name(&self) -> &'static str {
                $articled_display_name
            }

            fn feature_resolver(&self) -> &'static dyn TypeFeatureResolver {
                &$type_def
            }
        }

        impl HasLeafKind for $content_type {
            type LeafKind = $kind;

            fn kind(&self) -> Self::LeafKind {
                $kind
            }
        }

        impl_value_content_traits!(leaf: $type_def, $content_type);

        impl IsValueLeaf for $content_type {}

        $(
            impl $dyn_trait for $content_type {
                $($dyn_trait_impl)*
            }
            impl CastDyn<dyn $dyn_trait> for $content_type {
                fn map_boxed(self: Box<Self>) -> Option<Box<dyn $dyn_trait>> {
                    Some(self)
                }
                fn map_ref(&self) -> Option<&dyn $dyn_trait> {
                    Some(self)
                }
                fn map_mut(&mut self) -> Option<&mut dyn $dyn_trait> {
                    Some(self)
                }
            }
        )*
        impl_fallback_is_iterable!({$($dyn_trait)*} for $content_type);

        impl_ancestor_chain_conversions!(
            $type_def => $parent($parent_content :: $parent_variant) $(=> $ancestor)*
        );
    };
}

pub(crate) use define_leaf_type;

pub(crate) struct DynMapper<D: ?Sized>(std::marker::PhantomData<D>);

impl<D: ?Sized> DynMapper<D> {
    pub(crate) const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// Implements `IsValueContent`, `IntoValueContent`, and `FromValueContent` for various
/// form wrappers of a content type. This macro deduplicates code across `define_leaf_type`,
/// `define_parent_type`, and `define_dyn_type`.
macro_rules! impl_value_content_traits {
    // For leaf types - implements for all common form wrappers
    (leaf: $type_def:ty, $content_type:ty) => {
        // NB - we can't blanket implement this for all L: IsValueLeaf (unlike all the other forms)
        // because it potentially conflicts with the &'a L and &'a mut L blanket implementations.

        // BeOwned: content is X
        impl IsValueContent for $content_type {
            type Type = $type_def;
            type Form = BeOwned;
        }
        impl<'a> IntoValueContent<'a> for $content_type {
            fn into_content(self) -> Self {
                self
            }
        }
        impl<'a> FromValueContent<'a> for $content_type {
            fn from_content(content: Self) -> Self {
                content
            }
        }
    };

    // For parent types - $content<'a, F> where F: IsHierarchicalForm
    (parent: $type_def:ty, $content:ident) => {
        impl<'a, F: IsHierarchicalForm> IsValueContent for $content<'a, F> {
            type Type = $type_def;
            type Form = F;
        }
        impl<'a, F: IsHierarchicalForm> IntoValueContent<'a> for $content<'a, F> {
            fn into_content(self) -> Self {
                self
            }
        }
        impl<'a, F: IsHierarchicalForm> FromValueContent<'a> for $content<'a, F> {
            fn from_content(content: Self) -> Self {
                content
            }
        }
    };
}

pub(crate) use impl_value_content_traits;

macro_rules! define_dyn_type {
    (
        $type_def_vis:vis $type_def:ident,
        content: $dyn_type:ty,
        dyn_kind: DynTypeKind::$dyn_kind:ident,
        type_name: $source_type_name:literal,
        articled_display_name: $articled_display_name:literal,
    ) => {
        #[derive(Copy, Clone)]
        $type_def_vis struct $type_def;

        impl IsType for $type_def {
            type Variant = DynTypeVariant;

            const SOURCE_TYPE_NAME: &'static str = $source_type_name;
            const ARTICLED_DISPLAY_NAME: &'static str = $articled_display_name;

            fn type_kind() -> TypeKind {
                TypeKind::Dyn(DynTypeKind::$dyn_kind)
            }

            // This is called for every leaf in a row as part of parsing
            // Use inline(always) to avoid function call overhead, even in Debug builds
            // which are used when macros are run during development
            #[inline(always)]
            fn type_kind_from_source_name(name: &str) -> Option<TypeKind> {
                if name == Self::SOURCE_TYPE_NAME {
                    Some(Self::type_kind())
                } else {
                    None
                }
            }
        }

        impl IsDynType for $type_def {
            type DynContent = $dyn_type;
        }

        impl_type_feature_resolver! {
            impl TypeFeatureResolver for $type_def: [$type_def]
        }

        impl IsDynLeaf for $dyn_type {
            type Type = $type_def;
        }

        impl<T: IsHierarchicalType, F: IsHierarchicalForm + IsDynCompatibleForm + IsDynMappableForm> DynResolveFrom<T, F> for $type_def
        {
            fn downcast_from<'a>(content: Content<'a, T, F>) -> Option<DynContent<'a, Self, F>> {
                T::map_with::<'a, F, _>(DynMapper::<$dyn_type>::new(), content)
            }
        }

        impl<F: IsDynMappableForm> LeafMapper<F> for DynMapper<$dyn_type> {
            type Output<'a, T: IsHierarchicalType> =  Option<F::DynLeaf<'a, $dyn_type>>;

            fn to_parent_output<'a, T: IsChildType>(
                output: Self::Output<'a, T>,
            ) -> Self::Output<'a, T::ParentType> {
                output
            }

            fn map_leaf<'a, T: IsLeafType>(
                self,
                leaf: F::Leaf<'a, T>,
            ) -> Self::Output<'a, T> {
                F::leaf_to_dyn(leaf)
            }
        }
    };
}

pub(crate) use define_dyn_type;
