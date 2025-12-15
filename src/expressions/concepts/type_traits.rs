use super::*;

pub(crate) trait IsType: Sized {
    type Content<'a, F: IsForm>;

    fn articled_type_name() -> &'static str;
}

pub(crate) trait IsHierarchyType: IsType {
    fn map_with<'a, F: IsForm, M: GeneralMapper<F>>(
        content: Self::Content<'a, F>,
    ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>>;
}

pub(crate) trait IsDynType: IsType
where
    DynMapper<Self::DynContent>: GeneralMapper<BeOwned>,
{
    type DynContent: ?Sized;
}

pub(crate) trait GeneralMapper<F: IsForm> {
    type OutputForm: IsForm;
    type ShortCircuit<'a>;

    fn map_leaf<'a, L: IsValueLeaf, T>(
        leaf: F::Leaf<'a, L>,
    ) -> Result<<Self::OutputForm as IsForm>::Leaf<'a, L>, Self::ShortCircuit<'a>>
    where
        T: for<'l> IsType<Content<'l, F> = F::Leaf<'l, L>>,
        T: for<'l> IsType<
            Content<'l, Self::OutputForm> = <Self::OutputForm as IsForm>::Leaf<'l, L>,
        >;
}

pub(crate) trait MapToType<T: IsType, F: IsForm>: IsType {
    fn map_to_type<'a>(content: Self::Content<'a, F>) -> T::Content<'a, F>;
}

pub(crate) trait MaybeMapFromType<T: IsType, F: IsForm>: IsType {
    fn maybe_map_from_type<'a>(content: T::Content<'a, F>) -> Option<Self::Content<'a, F>>;

    fn resolve<'a>(
        actual: Actual<'a, T, F>,
        span_range: SpanRange,
        resolution_target: &str,
    ) -> ExecutionResult<Actual<'a, Self, F>> {
        let content = match Self::maybe_map_from_type(actual.0) {
            Some(c) => c,
            None => {
                return span_range.value_err(format!(
                    "{} is expected to be {}, but it is {}",
                    resolution_target,
                    Self::articled_type_name(),
                    T::articled_type_name(),
                ))
            }
        };
        Ok(Actual(content))
    }
}

pub(crate) trait IsChildType: IsType {
    type ParentType: IsType;
    fn into_parent<'a, F: IsForm>(
        content: Self::Content<'a, F>,
    ) -> <Self::ParentType as IsType>::Content<'a, F>;
    fn from_parent<'a, F: IsForm>(
        content: <Self::ParentType as IsType>::Content<'a, F>,
    ) -> Option<Self::Content<'a, F>>;
}

macro_rules! impl_ancestor_chain_conversions {
    ($child:ty => $parent:ty => [$($ancestor:ty),* $(,)?]) => {
        impl<F: IsForm> MaybeMapFromType<$child, F> for $child
        {
            fn maybe_map_from_type<'a>(
                content: <$child as IsType>::Content<'a, F>,
            ) -> Option<<$child as IsType>::Content<'a, F>> {
                Some(content)
            }
        }

        impl<F: IsForm> MapToType<$child, F> for $child
        {
            fn map_to_type<'a>(
                content: <$child as IsType>::Content<'a, F>,
            ) -> <$child as IsType>::Content<'a, F> {
                content
            }
        }

        impl<F: IsForm> MaybeMapFromType<$parent, F> for $child
        {
            fn maybe_map_from_type<'a>(
                content: <$parent as IsType>::Content<'a, F>,
            ) -> Option<<$child as IsType>::Content<'a, F>> {
                <$child as IsChildType>::from_parent(content)
            }
        }

        impl<F: IsForm> MapToType<$parent, F> for $child
        {
            fn map_to_type<'a>(
                content: <$child as IsType>::Content<'a, F>,
            ) -> <$parent as IsType>::Content<'a, F> {
                <$child as IsChildType>::into_parent(content)
            }
        }

        $(
            impl<F: IsForm> MaybeMapFromType<$ancestor, F> for $child {
                fn maybe_map_from_type<'a>(
                    content: <$ancestor as IsType>::Content<'a, F>,
                ) -> Option<<$child as IsType>::Content<'a, F>> {
                    <$child as MaybeMapFromType<$parent, F>>::maybe_map_from_type(<$parent as MaybeMapFromType<$ancestor, F>>::maybe_map_from_type(content)?)
                }
            }

            impl<F: IsForm> MapToType<$ancestor, F> for $child {
                fn map_to_type<'a>(
                    content: <$child as IsType>::Content<'a, F>,
                ) -> <$ancestor as IsType>::Content<'a, F> {
                    <$parent as MapToType<$ancestor, F>>::map_to_type(<$child as MapToType<$parent, F>>::map_to_type(content))
                }
            }
        )*
    };
}

pub(crate) use impl_ancestor_chain_conversions;

pub(crate) trait IsValueLeaf:
    'static + IntoValueContent<'static, Form = BeOwned> + CastDyn<dyn IsIterable>
{
}

pub(crate) trait IsDynLeaf: 'static + IsValueContent<'static>
where
    DynMapper<Self>: GeneralMapper<BeOwned>,
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

macro_rules! define_leaf_type {
    ($leaf_type:ty : $type_data_vis:vis $leaf_type_def:ident $articled_type_name:literal) => {
        $type_data_vis struct $leaf_type_def;

        impl IsType for $leaf_type_def {
            type Content<'a, F: IsForm> = F::Leaf<'a, $leaf_type>;

            fn articled_type_name() -> &'static str {
                $articled_type_name
            }
        }

        impl IsHierarchyType for $leaf_type_def {
            fn map_with<'a, F: IsForm, M: GeneralMapper<F>>(
                content: Self::Content<'a, F>,
            ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>> {
                M::map_leaf::<$leaf_type, Self>(content)
            }
        }

        impl<'a> IsValueContent<'a> for $leaf_type {
            type Type = $leaf_type_def;
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
    };

}

pub(crate) use define_leaf_type;
