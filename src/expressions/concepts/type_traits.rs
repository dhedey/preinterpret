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
    // The following is always true, courtesy of the definition of IsFormOf:
    //   <F as form::IsFormOf<Self>>::Content<'a>> := Self::Content<'a, F>
    // So the following where clause can be added where needed to make types line up:
    //   for<'l> T: IsHierarchicalType<Content<'l, F> = <F as form::IsFormOf<T>>::Content<'l>>,
    type Content<'a, F: IsHierarchicalForm>;
    type LeafKind: IsSpecificLeafKind;

    fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
        structure: Self::Content<'a, F>,
    ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>>;

    fn map_ref_with<'r, 'a: 'r, F: IsHierarchicalForm, M: RefLeafMapper<F>>(
        structure: &'r Self::Content<'a, F>,
    ) -> Result<Self::Content<'r, M::OutputForm>, M::ShortCircuit<'a>>;

    fn map_mut_with<'r, 'a: 'r, F: IsHierarchicalForm, M: MutLeafMapper<F>>(
        structure: &'r mut Self::Content<'a, F>,
    ) -> Result<Self::Content<'r, M::OutputForm>, M::ShortCircuit<'a>>;

    fn content_to_leaf_kind<F: IsHierarchicalForm>(
        content: &Self::Content<'_, F>,
    ) -> Self::LeafKind;
}

pub(crate) trait IsLeafType: IsHierarchicalType {
    fn leaf_kind() -> Self::LeafKind;
}

pub(crate) trait IsDynType: IsType<Variant = DynTypeVariant> {
    type DynContent: ?Sized + 'static;
}

pub(crate) trait LeafMapper<F: IsHierarchicalForm> {
    type OutputForm: IsHierarchicalForm;
    type ShortCircuit<'a>;

    fn map_leaf<'a, L: IsValueLeaf>(
        leaf: F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsHierarchicalForm>::Leaf<'a, L>, Self::ShortCircuit<'a>>;
}

pub(crate) trait RefLeafMapper<F: IsHierarchicalForm> {
    type OutputForm: IsHierarchicalForm;
    type ShortCircuit<'a>;

    fn map_leaf<'r, 'a: 'r, L: IsValueLeaf>(
        leaf: &'r F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsHierarchicalForm>::Leaf<'r, L>, Self::ShortCircuit<'a>>;
}

pub(crate) trait MutLeafMapper<F: IsHierarchicalForm> {
    type OutputForm: IsHierarchicalForm;
    type ShortCircuit<'a>;

    fn map_leaf<'r, 'a: 'r, L: IsValueLeaf>(
        leaf: &'r mut F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsHierarchicalForm>::Leaf<'r, L>, Self::ShortCircuit<'a>>;
}

pub(crate) trait UpcastTo<T: IsType, F: IsFormOf<T> + IsFormOf<Self>>: IsType {
    fn upcast_to<'a>(
        content: <F as IsFormOf<Self>>::Content<'a>,
    ) -> <F as IsFormOf<T>>::Content<'a>;
}

pub(crate) trait DowncastFrom<T: IsType, F: IsFormOf<T> + IsFormOf<Self>>: IsType {
    fn downcast_from<'a>(
        content: <F as IsFormOf<T>>::Content<'a>,
    ) -> Option<<F as IsFormOf<Self>>::Content<'a>>;

    fn resolve<'a>(
        actual: Actual<'a, T, F>,
        span_range: SpanRange,
        resolution_target: &str,
    ) -> ExecutionResult<Actual<'a, Self, F>> {
        let content = match Self::downcast_from(actual.0) {
            Some(c) => c,
            None => {
                return span_range.value_err(format!(
                    "{} is expected to be {}, but it is {}",
                    resolution_target,
                    Self::ARTICLED_DISPLAY_NAME,
                    T::ARTICLED_DISPLAY_NAME,
                ))
            }
        };
        Ok(Actual(content))
    }
}

pub(crate) trait IsChildType: IsHierarchicalType {
    type ParentType: IsHierarchicalType;

    fn into_parent<'a, F: IsHierarchicalForm>(
        content: Self::Content<'a, F>,
    ) -> <Self::ParentType as IsHierarchicalType>::Content<'a, F>;

    fn from_parent<'a, F: IsHierarchicalForm>(
        content: <Self::ParentType as IsHierarchicalType>::Content<'a, F>,
    ) -> Option<Self::Content<'a, F>>;
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

            fn resolve_type_property(&self, property_name: &str) -> Option<Value> {
                // Purposefully doesn't resolve parents, but TBC if this is right
                <$type_def as TypeData>::resolve_type_property(property_name)
            }
        }
    };
}

pub(crate) use impl_type_feature_resolver;

macro_rules! impl_ancestor_chain_conversions {
    ($child:ty $(=> $parent:ident($parent_content:ident :: $parent_variant:ident) $(=> $ancestor:ty)*)?) => {
        impl<F: IsHierarchicalForm> DowncastFrom<$child, F> for $child
        {
            fn downcast_from<'a>(
                content: <F as IsFormOf<Self>>::Content<'a>,
            ) -> Option<<F as IsFormOf<Self>>::Content<'a>> {
                Some(content)
            }
        }

        impl<F: IsHierarchicalForm> UpcastTo<$child, F> for $child
        {
            fn upcast_to<'a>(
                content: <F as IsFormOf<Self>>::Content<'a>,
            ) -> <F as IsFormOf<Self>>::Content<'a> {
                content
            }
        }

        $(
            impl IsChildType for $child {
                type ParentType = $parent;

                fn into_parent<'a, F: IsHierarchicalForm>(
                    content: Self::Content<'a, F>,
                ) -> <Self::ParentType as IsHierarchicalType>::Content<'a, F> {
                    $parent_content::$parent_variant(Actual(content))
                }

                fn from_parent<'a, F: IsHierarchicalForm>(
                    content: <Self::ParentType as IsHierarchicalType>::Content<'a, F>,
                ) -> Option<Self::Content<'a, F>> {
                    match content {
                        $parent_content::$parent_variant(i) => Some(i.0),
                        _ => None,
                    }
                }
            }

            impl<F: IsHierarchicalForm> DowncastFrom<$parent, F> for $child
            {
                fn downcast_from<'a>(
                    content: <F as IsFormOf<$parent>>::Content<'a>,
                ) -> Option<<F as IsFormOf<Self>>::Content<'a>> {
                    <$child as IsChildType>::from_parent(content)
                }
            }

            impl<F: IsHierarchicalForm> UpcastTo<$parent, F> for $child
            {
                fn upcast_to<'a>(
                    content: <F as IsFormOf<$child>>::Content<'a>,
                ) -> <F as IsFormOf<$parent>>::Content<'a> {
                    <$child as IsChildType>::into_parent(content)
                }
            }

            $(
                impl<F: IsHierarchicalForm> DowncastFrom<$ancestor, F> for $child {
                    fn downcast_from<'a>(
                        content: <F as IsFormOf<$ancestor>>::Content<'a>,
                    ) -> Option<<F as IsFormOf<$child>>::Content<'a>> {
                        <$child as DowncastFrom<$parent, F>>::downcast_from(<$parent as DowncastFrom<$ancestor, F>>::downcast_from(content)?)
                    }
                }

                impl<F: IsHierarchicalForm> UpcastTo<$ancestor, F> for $child {
                    fn upcast_to<'a>(
                        content: <F as IsFormOf<$child>>::Content<'a>,
                    ) -> <F as IsFormOf<$ancestor>>::Content<'a> {
                        <$parent as UpcastTo<$ancestor, F>>::upcast_to(<$child as UpcastTo<$parent, F>>::upcast_to(content))
                    }
                }
            )*
        )?
    };
}

pub(crate) use impl_ancestor_chain_conversions;

pub(crate) trait IsValueLeaf:
    'static + IntoValueContent<'static, Form = BeOwned> + CastDyn<dyn IsIterable>
{
}

pub(crate) trait IsDynLeaf: 'static + IsValueContent<'static>
where
    DynMapper<Self>: LeafMapper<BeOwned>,
    Self::Type: IsDynType<DynContent = Self>,
{
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
    type LeafKind: IsSpecificLeafKind;

    fn kind(&self) -> Self::LeafKind;

    fn value_kind(&self) -> ValueLeafKind {
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
                content: Self::Content<'a, F>,
            ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>> {
                Ok(match content {
                    $( $content::$variant(x) => $content::$variant(x.map_with::<M>()?), )*
                })
            }

            fn map_ref_with<'r, 'a: 'r, F: IsHierarchicalForm, M: RefLeafMapper<F>>(
                content: &'r Self::Content<'a, F>,
            ) -> Result<Self::Content<'r, M::OutputForm>, M::ShortCircuit<'a>> {
                Ok(match content {
                    $( $content::$variant(x) => $content::$variant(x.map_ref_with::<M>()?), )*
                })
            }

            fn map_mut_with<'r, 'a: 'r, F: IsHierarchicalForm, M: MutLeafMapper<F>>(
                content: &'r mut Self::Content<'a, F>,
            ) -> Result<Self::Content<'r, M::OutputForm>, M::ShortCircuit<'a>> {
                Ok(match content {
                    $( $content::$variant(x) => $content::$variant(x.map_mut_with::<M>()?), )*
                })
            }

            fn content_to_leaf_kind<F: IsHierarchicalForm>(
                content: &Self::Content<'_, F>,
            ) -> Self::LeafKind {
                content.kind()
            }
        }

        $content_vis enum $content<'a, F: IsHierarchicalForm> {
            $( $variant(Actual<'a, $variant_type, F>), )*
        }

        impl<'a, F: IsHierarchicalForm> HasLeafKind for $content<'a, F> {
            type LeafKind = $leaf_kind;

            fn kind(&self) -> Self::LeafKind {
                match self {
                    $($content::$variant(x) => $leaf_kind::$variant(
                        <$variant_type as IsHierarchicalType>::content_to_leaf_kind::<F>(&x.0)
                    ),)*
                }
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        $type_kind_vis struct $type_kind;

        impl $type_kind {
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
            impl From<$leaf_kind> for ValueLeafKind {
                fn from(kind: $leaf_kind) -> Self {
                    let as_parent_kind = <$parent as IsHierarchicalType>::LeafKind::$parent_variant(kind);
                    ValueLeafKind::from(as_parent_kind)
                }
            }
        )?

        impl IsSpecificLeafKind for $leaf_kind {
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
        $type_def_vis struct $type_def;

        impl IsType for $type_def {
            type Variant = HierarchicalTypeVariant;

            const SOURCE_TYPE_NAME: &'static str = $source_type_name;
            const ARTICLED_DISPLAY_NAME: &'static str = $articled_display_name;

            fn type_kind() -> TypeKind {
                TypeKind::Leaf(ValueLeafKind::from($kind))
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
            type Content<'a, F: IsHierarchicalForm> = F::Leaf<'a, $content_type>;
            type LeafKind = $kind;

            fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
                content: Self::Content<'a, F>,
            ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>> {
                M::map_leaf::<$content_type>(content)
            }

            fn map_ref_with<'r, 'a: 'r, F: IsHierarchicalForm, M: RefLeafMapper<F>>(
                content: &'r Self::Content<'a, F>,
            ) -> Result<Self::Content<'r, M::OutputForm>, M::ShortCircuit<'a>> {
                M::map_leaf::<$content_type>(content)
            }

            fn map_mut_with<'r, 'a: 'r, F: IsHierarchicalForm, M: MutLeafMapper<F>>(
                content: &'r mut Self::Content<'a, F>,
            ) -> Result<Self::Content<'r, M::OutputForm>, M::ShortCircuit<'a>> {
                M::map_leaf::<$content_type>(content)
            }

            fn content_to_leaf_kind<F: IsHierarchicalForm>(
                _content: &Self::Content<'_, F>,
            ) -> Self::LeafKind {
                $kind
            }
        }

        impl IsLeafType for $type_def {
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

        impl From<$kind> for ValueLeafKind {
            fn from(kind: $kind) -> Self {
                let as_parent_kind = <$parent as IsHierarchicalType>::LeafKind::$parent_variant(kind);
                ValueLeafKind::from(as_parent_kind)
            }
        }

        impl IsSpecificLeafKind for $kind {
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

        impl<'a> IsValueContent<'a> for $content_type {
            type Type = $type_def;
            type Form = BeOwned;
        }

        impl HasLeafKind for $content_type {
            type LeafKind = $kind;

            fn kind(&self) -> Self::LeafKind {
                $kind
            }
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

macro_rules! define_dyn_type {
    (
        $type_def_vis:vis $type_def:ident,
        content: $dyn_type:ty,
        dyn_kind: DynTypeKind::$dyn_kind:ident,
        type_name: $source_type_name:literal,
        articled_display_name: $articled_display_name:literal,
    ) => {
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

        impl<'a> IsValueContent<'a> for $dyn_type {
            type Type = $type_def;
            type Form = BeOwned;
        }

        impl IsDynLeaf for $dyn_type {}

        impl<T: IsHierarchicalType, F: IsFormOf<T> + IsFormOf<$type_def> + IsDynMappableForm> DowncastFrom<T, F> for $type_def
            where
                for<'a> T: IsHierarchicalType<Content<'a, F> = <F as IsFormOf<T>>::Content<'a>>,
                for<'a> F: IsDynCompatibleForm<DynLeaf<'a, $dyn_type> = <F as IsFormOf<Self>>::Content<'a>>,
        {
            fn downcast_from<'a>(content: <F as IsFormOf<T>>::Content<'a>) -> Option<<F as IsFormOf<Self>>::Content<'a>> {
                match T::map_with::<'a, F, DynMapper<$dyn_type>>(content) {
                    Ok(_) => panic!("DynMapper is expected to always short-circuit"),
                    Err(dyn_leaf) => dyn_leaf,
                }
            }
        }

        impl<F: IsDynMappableForm> LeafMapper<F> for DynMapper<$dyn_type> {
            type OutputForm = BeOwned; // Unused
            type ShortCircuit<'a> = Option<F::DynLeaf<'a, $dyn_type>>;

            fn map_leaf<'a, L: IsValueLeaf>(
                leaf: F::Leaf<'a, L>,
            ) -> Result<<Self::OutputForm as IsHierarchicalForm>::Leaf<'a, L>, Self::ShortCircuit<'a>>
            {
                Err(F::leaf_to_dyn(leaf))
            }
        }
    };
}

pub(crate) use define_dyn_type;
