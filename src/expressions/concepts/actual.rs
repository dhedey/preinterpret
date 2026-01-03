use super::*;

pub(crate) struct Actual<'a, T: IsType, F: IsFormOf<T>>(pub(crate) F::Content<'a>);

impl<'a, T: IsType, F: IsFormOf<T>> Actual<'a, T, F> {
    #[inline]
    pub(crate) fn of(content: impl IntoValueContent<'a, Type = T, Form = F>) -> Self {
        Actual(content.into_content())
    }

    #[inline]
    pub(crate) fn upcast<S: IsType>(self) -> Actual<'a, S, F>
    where
        F: IsFormOf<S>,
        T: UpcastTo<S, F>,
    {
        Actual(T::upcast_to(self.0))
    }

    #[inline]
    pub(crate) fn downcast<U: DowncastFrom<T, F>>(self) -> Option<Actual<'a, U, F>>
    where
        F: IsFormOf<U>,
    {
        Some(Actual(U::downcast_from(self.0)?))
    }

    #[inline]
    pub(crate) fn map_with<M: LeafMapper<F>>(
        self,
    ) -> Result<Actual<'a, T, M::OutputForm>, M::ShortCircuit<'a>>
    where
        for<'l> T: IsHierarchicalType<Content<'l, F> = <F as form::IsFormOf<T>>::Content<'l>>,
        F: IsHierarchicalForm,
    {
        T::map_with::<'a, F, M>(self.0).map(|c| Actual(c))
    }
}

impl<'a, T: IsType, F: IsFormOf<T>> Spanned<Actual<'a, T, F>> {
    #[inline]
    pub(crate) fn resolve_as<X: FromValueContent<'a, Form = F>>(
        self,
        description: &str,
    ) -> ExecutionResult<X>
    where
        F: IsFormOf<<X as IsValueContent<'a>>::Type>,
        <X as IsValueContent<'a>>::Type: DowncastFrom<T, F>,
    {
        let Spanned(value, span_range) = self;
        let resolved = <<X as IsValueContent<'a>>::Type>::resolve(value, span_range, description)?;
        Ok(X::from_actual(resolved))
    }
}

impl<'a, T: IsType + UpcastTo<ValueType, F>, F: IsFormOf<T> + IsFormOf<ValueType>>
    Actual<'a, T, F>
{
    pub(crate) fn into_value(self) -> Actual<'a, ValueType, F> {
        self.upcast()
    }
}

impl<'a, T: IsType, F: IsFormOf<T>> Deref for Actual<'a, T, F> {
    type Target = F::Content<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: IsType, F: IsFormOf<T>> DerefMut for Actual<'a, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: IsHierarchicalType, F: IsFormOf<T>> HasLeafKind for Actual<'a, T, F>
where
    F: IsHierarchicalForm,
    for<'l> T: IsHierarchicalType<Content<'l, F> = <F as form::IsFormOf<T>>::Content<'l>>,
{
    type LeafKind = <T as IsHierarchicalType>::LeafKind;

    fn kind(&self) -> Self::LeafKind {
        <T as IsHierarchicalType>::content_to_leaf_kind::<F>(&self.0)
    }
}

pub(crate) trait IsValueContent<'a> {
    type Type: IsType;
    type Form: IsFormOf<Self::Type>;
}

pub(crate) trait IntoValueContent<'a>: IsValueContent<'a> {
    fn into_content(self) -> <Self::Form as IsFormOf<Self::Type>>::Content<'a>;

    #[inline]
    fn into_actual(self) -> Actual<'a, Self::Type, Self::Form>
    where
        Self: Sized,
    {
        Actual::of(self)
    }
}

pub(crate) trait FromValueContent<'a>: IsValueContent<'a> {
    fn from_content(content: <Self::Form as IsFormOf<Self::Type>>::Content<'a>) -> Self;

    #[inline]
    fn from_actual(actual: Actual<'a, Self::Type, Self::Form>) -> Self
    where
        Self: Sized,
    {
        Self::from_content(actual.0)
    }
}

impl<'a, T: IsType, F: IsFormOf<T>> IsValueContent<'a> for Actual<'a, T, F> {
    type Type = T;
    type Form = F;
}

impl<'a, T: IsType, F: IsFormOf<T>> FromValueContent<'a> for Actual<'a, T, F> {
    fn from_content(content: <F as IsFormOf<T>>::Content<'a>) -> Self {
        Actual(content)
    }
}

impl<'a, T: IsType, F: IsFormOf<T>> IntoValueContent<'a> for Actual<'a, T, F> {
    fn into_content(self) -> <F as IsFormOf<T>>::Content<'a> {
        self.0
    }
}
