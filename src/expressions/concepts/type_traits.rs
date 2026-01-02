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
}

pub(crate) trait IsHierarchicalType: IsType<Variant = HierarchicalTypeVariant> {
    // <F as form::IsFormOf<Self>>::Content<'a>> := Self::Content<'a, F>
    type Content<'a, F: IsHierarchicalForm>;
    type LeafKind: IsSpecificLeafKind;

    fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
        structure: Self::Content<'a, F>,
    ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>>;
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
    type LeafKindType: IsSpecificLeafKind;

    const KIND: Self::LeafKindType;

    fn kind(&self) -> Self::LeafKindType {
        Self::KIND
    }

    fn value_kind(&self) -> ValueKind {
        self.kind().into()
    }

    fn articled_kind(&self) -> &'static str {
        self.kind().articled_display_name()
    }
}

impl<T: IntoValueContent<'static>> HasLeafKind for T
where
    T::Type: HasLeafKind,
{
    type LeafKindType = <T::Type as HasLeafKind>::LeafKindType;

    const KIND: Self::LeafKindType = T::Type::KIND;
}

macro_rules! define_parent_type {
    (
        $type_def_vis:vis $type_def:ident $(=> $parent:ident($parent_content:ident :: $parent_variant:ident) $(=> $ancestor:ty)*)?,
        content: $content_vis:vis $content:ident,
        kind: $kind_vis:vis $kind:ident,
        parent_kind: ParentTypeKind::$parent_kind:ident,
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
                TypeKind::Parent(ParentTypeKind::$parent_kind)
            }
        }

        impl MethodResolver for $type_def {
            fn resolve_method(&self, _method_name: &str) -> Option<MethodInterface> {
                unimplemented!()
            }

            fn resolve_unary_operation(
                &self,
                _operation: &UnaryOperation,
            ) -> Option<UnaryOperationInterface> {
                unimplemented!()
            }

            fn resolve_binary_operation(
                &self,
                _operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                unimplemented!()
            }

            fn resolve_type_property(&self, _property_name: &str) -> Option<Value> {
                unimplemented!()
            }
        }

        impl IsHierarchicalType for $type_def {
            type Content<'a, F: IsHierarchicalForm> = $content<'a, F>;
            type LeafKind = $kind;

            fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
                content: Self::Content<'a, F>,
            ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>> {
                Ok(match content {
                    $( $content::$variant(x) => $content::$variant(x.map_with::<M>()?), )*
                })
            }
        }

        $content_vis enum $content<'a, F: IsHierarchicalForm> {
            $( $variant(Actual<'a, $variant_type, F>), )*
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        $kind_vis enum $kind {
            $( $variant(<$variant_type as IsHierarchicalType>::LeafKind), )*
        }

        $(
            impl From<$kind> for ValueKind {
                fn from(kind: $kind) -> Self {
                    let as_parent_kind = <$parent as IsHierarchicalType>::LeafKind::$parent_variant(kind);
                    ValueKind::from(as_parent_kind)
                }
            }
        )?

        impl IsSpecificLeafKind for $kind {
            fn articled_display_name(&self) -> &'static str {
                match self {
                    $( Self::$variant(x) => x.articled_display_name(), )*
                }
            }

            fn method_resolver(&self) -> &'static dyn MethodResolver {
                &$type_def
            }
        }

        impl_ancestor_chain_conversions!(
            $type_def $(=> $parent($parent_content :: $parent_variant) $(=> $ancestor)*)?
        );
    };
}

pub(crate) use define_parent_type;

macro_rules! define_leaf_type {
    (
        $type_def_vis:vis $type_def:ident => $parent:ident($parent_content:ident :: $parent_variant:ident) $(=> $ancestor:ty)*,
        content: $leaf_type:ty,
        kind: $kind_vis:vis $kind:ident,
        type_name: $source_type_name:literal,
        articled_display_name: $articled_display_name:literal,
    ) => {
        $type_def_vis struct $type_def;

        impl IsType for $type_def {
            type Variant = HierarchicalTypeVariant;

            const SOURCE_TYPE_NAME: &'static str = $source_type_name;
            const ARTICLED_DISPLAY_NAME: &'static str = $articled_display_name;

            fn type_kind() -> TypeKind {
                TypeKind::Leaf(ValueKind::from($kind))
            }
        }

        impl IsHierarchicalType for $type_def {
            type Content<'a, F: IsHierarchicalForm> = F::Leaf<'a, $leaf_type>;
            type LeafKind = $kind;

            fn map_with<'a, F: IsHierarchicalForm, M: LeafMapper<F>>(
                content: Self::Content<'a, F>,
            ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>> {
                M::map_leaf::<$leaf_type>(content)
            }
        }

        impl MethodResolver for $type_def {
            fn resolve_method(&self, _method_name: &str) -> Option<MethodInterface> {
                unimplemented!()
            }

            fn resolve_unary_operation(
                &self,
                _operation: &UnaryOperation,
            ) -> Option<UnaryOperationInterface> {
                unimplemented!()
            }

            fn resolve_binary_operation(
                &self,
                _operation: &BinaryOperation,
            ) -> Option<BinaryOperationInterface> {
                unimplemented!()
            }

            fn resolve_type_property(&self, _property_name: &str) -> Option<Value> {
                unimplemented!()
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        $kind_vis struct $kind;

        impl From<$kind> for ValueKind {
            fn from(kind: $kind) -> Self {
                let as_parent_kind = <$parent as IsHierarchicalType>::LeafKind::$parent_variant(kind);
                ValueKind::from(as_parent_kind)
            }
        }

        impl IsSpecificLeafKind for $kind {
            fn articled_display_name(&self) -> &'static str {
                $articled_display_name
            }

            fn method_resolver(&self) -> &'static dyn MethodResolver {
                &$type_def
            }
        }

        impl<'a> IsValueContent<'a> for $leaf_type {
            type Type = $type_def;
            type Form = BeOwned;
        }

        impl<'a> IntoValueContent<'a> for $leaf_type {
            fn into_content(self) -> Self {
                self
            }
        }

        impl<'a> FromValueContent<'a> for $leaf_type {
            fn from_content(content: Self) -> Self {
                content
            }
        }

        impl IsValueLeaf for $leaf_type {}
        impl CastDyn<dyn IsIterable> for $leaf_type {}

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
        }

        impl IsDynType for $type_def {
            type DynContent = $dyn_type;
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
