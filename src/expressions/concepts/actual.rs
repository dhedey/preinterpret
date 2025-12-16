use super::*;

pub(crate) struct Actual<'a, T: IsType, F: IsForm>(pub(crate) T::Content<'a, F>);

impl<'a, T: IsType, F: IsForm> Actual<'a, T, F> {
    #[inline]
    pub(crate) fn of(content: impl IntoValueContent<'a, Type = T, Form = F>) -> Self {
        Actual(content.into_content())
    }

    #[inline]
    pub(crate) fn map_type<S: IsType>(self) -> Actual<'a, S, F>
    where
        T: UpcastTo<S, F>,
    {
        Actual(T::upcast_to(self.0))
    }

    #[inline]
    pub(crate) fn map_type_maybe<U: DowncastFrom<T, F>>(self) -> Option<Actual<'a, U, F>> {
        Some(Actual(U::downcast_from(self.0)?))
    }

    #[inline]
    pub(crate) fn map_with<M: GeneralMapper<F>>(
        self,
    ) -> Result<Actual<'a, T, M::OutputForm>, M::ShortCircuit<'a>>
    where
        T: IsHierarchyType,
    {
        T::map_with::<'a, F, M>(self.0).map(|c| Actual(c))
    }
}

impl<'a, T: IsType, F: IsForm> Spanned<Actual<'a, T, F>> {
    #[inline]
    pub(crate) fn resolve_as<X: FromValueContent<'a, Form = F>>(
        self,
        description: &str,
    ) -> ExecutionResult<X>
    where
        <X as IsValueContent<'a>>::Type: DowncastFrom<T, F>,
    {
        let Spanned(value, span_range) = self;
        let resolved = <<X as IsValueContent<'a>>::Type>::resolve(value, span_range, description)?;
        Ok(X::from_actual(resolved))
    }
}

impl<'a, T: IsType + UpcastTo<ValueType, F>, F: IsForm> Actual<'a, T, F> {
    pub(crate) fn into_value(self) -> Actual<'a, ValueType, F> {
        self.map_type()
    }
}

impl<'a, T: IsType, F: IsForm> Deref for Actual<'a, T, F> {
    type Target = T::Content<'a, F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: IsType, F: IsForm> DerefMut for Actual<'a, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) trait ContentMapper<F: IsForm> {
    type TOut<'a>;

    fn map_leaf<'a, L: IsValueLeaf>(leaf: F::Leaf<'a, L>) -> Self::TOut<'a>;
}

pub(crate) trait IsValueContent<'a> {
    type Type: IsType;
    type Form: IsForm;
}

pub(crate) trait IntoValueContent<'a>: IsValueContent<'a> {
    fn into_content(self) -> <Self::Type as IsType>::Content<'a, Self::Form>;

    #[inline]
    fn into_actual(self) -> Actual<'a, Self::Type, Self::Form>
    where
        Self: Sized,
    {
        Actual::of(self)
    }
}

pub(crate) trait FromValueContent<'a>: IsValueContent<'a> {
    fn from_content(content: <Self::Type as IsType>::Content<'a, Self::Form>) -> Self;

    #[inline]
    fn from_actual(actual: Actual<'a, Self::Type, Self::Form>) -> Self
    where
        Self: Sized,
    {
        Self::from_content(actual.0)
    }
}

impl<'a, T: IsType, F: IsForm> IsValueContent<'a> for Actual<'a, T, F> {
    type Type = T;
    type Form = F;
}

impl<'a, T: IsType, F: IsForm> FromValueContent<'a> for Actual<'a, T, F> {
    fn from_content(content: <T as IsType>::Content<'a, F>) -> Self {
        Actual(content)
    }
}

impl<'a, T: IsType, F: IsForm> IntoValueContent<'a> for Actual<'a, T, F> {
    fn into_content(self) -> <T as IsType>::Content<'a, F> {
        self.0
    }
}
