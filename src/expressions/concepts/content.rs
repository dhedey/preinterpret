use std::mem::transmute;

use super::*;

/// Shorthand for representing the form F of a type T with a particular lifetime 'a.
pub(crate) type Actual<'a, T, F> = <F as IsFormOf<T>>::Content<'a>;

/// For types which have an associated value (type and form)
pub(crate) trait IsValueContent<'a> {
    type Type: IsType;
    type Form: IsFormOf<Self::Type>;
}

pub(crate) trait FromValueContent<'a>: IsValueContent<'a> {
    fn from_content(content: <Self::Form as IsFormOf<Self::Type>>::Content<'a>) -> Self;
}

pub(crate) trait IntoValueContent<'a>: IsValueContent<'a> {
    fn into_content(self) -> <Self::Form as IsFormOf<Self::Type>>::Content<'a>;

    #[inline]
    fn upcast<S: IsType>(self) -> Actual<'a, S, Self::Form>
    where
        Self: Sized,
        Self::Form: IsFormOf<S>,
        Self::Type: UpcastTo<S, Self::Form>,
    {
        <Self::Type as UpcastTo<S, Self::Form>>::upcast_to(self.into_content())
    }

    #[inline]
    fn downcast<U: DowncastFrom<Self::Type, Self::Form>>(self) -> Option<Actual<'a, U, Self::Form>>
    where
        Self: Sized,
        Self::Form: IsFormOf<U>,
        for<'l> Self::Type: IsHierarchicalType<
            Content<'l, Self::Form> = <Self::Form as form::IsFormOf<Self::Type>>::Content<'l>,
        >,
        Self::Form: IsHierarchicalForm,
    {
        U::downcast_from(self.into_content())
    }

    #[inline]
    fn into_any(self) -> Actual<'a, AnyType, Self::Form>
    where
        Self: Sized,
        Self::Type: UpcastTo<AnyType, Self::Form>,
        Self::Form: IsFormOf<AnyType>,
    {
        self.upcast()
    }

    fn map_with<M: LeafMapper<Self::Form>>(
        self,
        mapper: M,
    ) -> Result<Actual<'a, Self::Type, M::OutputForm>, M::ShortCircuit<'a>>
    where
        Self: Sized,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,
    {
        <Self::Type>::map_with::<Self::Form, _>(mapper, self.into_content())
    }

    fn into_referenceable(self) -> Actual<'a, Self::Type, BeReferenceable>
    where
        Self: Sized,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,
        for<'l> OwnedToReferenceableMapper:
            LeafMapper<Self::Form, OutputForm = BeReferenceable, ShortCircuit<'l> = Infallible>,
    {
        match self.map_with(OwnedToReferenceableMapper) {
            Ok(output) => output,
            Err(infallible) => match infallible {}, // Need to include because of MSRV
        }
    }
}

impl<'a, C> Spanned<C>
where
    C: IntoValueContent<'a>,
    for<'l> C::Type:
        IsHierarchicalType<Content<'l, C::Form> = <C::Form as IsFormOf<C::Type>>::Content<'l>>,
    C::Form: IsHierarchicalForm,
{
    pub(crate) fn downcast_resolve<X: FromValueContent<'a, Form = C::Form>>(
        self,
        description: &str,
    ) -> ExecutionResult<X>
    where
        C::Form: IsFormOf<<X as IsValueContent<'a>>::Type>,
        <X as IsValueContent<'a>>::Type: DowncastFrom<C::Type, C::Form>,
    {
        let Spanned(value, span_range) = self;
        let content = value.into_content();
        let resolved =
            <<X as IsValueContent<'a>>::Type>::resolve(content, span_range, description)?;
        Ok(X::from_content(resolved))
    }

    pub(crate) fn downcast_resolve_spanned<X: FromValueContent<'a, Form = C::Form>>(
        self,
        description: &str,
    ) -> ExecutionResult<Spanned<X>>
    where
        C::Form: IsFormOf<<X as IsValueContent<'a>>::Type>,
        <X as IsValueContent<'a>>::Type: DowncastFrom<C::Type, C::Form>,
    {
        let span_range = self.1;
        Ok(Spanned(
            self.downcast_resolve::<X>(description)?,
            span_range,
        ))
    }
}

// TODO[concepts]: Remove eventually, along with IntoValue impl
impl<X: IntoValueContent<'static, Form = BeOwned>> IntoValue for X
where
    X::Type: UpcastTo<AnyType, BeOwned>,
    BeOwned: IsFormOf<X::Type>,
{
    fn into_value(self) -> AnyValue {
        self.into_any()
    }
}

/// Implementations on the value contents *themselves*.
/// Typically this is reserved for things operating on references - otherwise we can
/// implement on IntoValueContent / FromValueContent.
pub(crate) trait IsSelfValueContent<'a>: IsValueContent<'a>
where
    Self::Form: IsFormOf<Self::Type, Content<'a> = Self>,
{
    fn map_mut_with<'r, M: MutLeafMapper<Self::Form>>(
        &'r mut self,
        mapper: M,
    ) -> Result<Actual<'r, Self::Type, M::OutputForm>, M::ShortCircuit<'a>>
    where
        'a: 'r,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,
    {
        <Self::Type>::map_mut_with::<Self::Form, _>(mapper, self)
    }

    fn map_ref_with<'r, M: RefLeafMapper<Self::Form>>(
        &'r self,
        mapper: M,
    ) -> Result<Actual<'r, Self::Type, M::OutputForm>, M::ShortCircuit<'a>>
    where
        'a: 'r,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,
    {
        <Self::Type>::map_ref_with::<Self::Form, _>(mapper, self)
    }

    fn as_mut_value<'r>(&'r mut self) -> Actual<'r, Self::Type, BeMut>
    where
        // Bounds for map_mut_with to work
        'a: 'r,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,

        // Bounds for ToMutMapper to work
        Self::Form: LeafAsMutForm,
        BeMut: IsFormOf<Self::Type>,
    {
        match self.map_mut_with(ToMutMapper) {
            Ok(x) => x,
            Err(infallible) => match infallible {}, // Need to include because of MSRV
        }
    }

    fn as_ref_value<'r>(&'r self) -> Actual<'r, Self::Type, BeRef>
    where
        // Bounds for map_ref_with to work
        'a: 'r,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,

        // Bounds for ToRefMapper to work
        Self::Form: LeafAsRefForm,
    {
        match self.map_ref_with(ToRefMapper) {
            Ok(x) => x,
            Err(infallible) => match infallible {}, // Need to include because of MSRV
        }
    }

    /// This method should only be used when you are certain that the value should be cloned.
    /// In most situations, you may wish to use [IsSelfValueContent::clone_to_owned_transparently]
    /// instead.
    fn clone_to_owned_infallible<'r>(&'r self) -> Actual<'static, Self::Type, BeOwned>
    where
        // Bounds for map_mut_with to work
        'a: 'r,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,

        // Bounds for LeafAsRefForm to work
        Self::Form: LeafAsRefForm,

        // Bounds for cloning to work
        Self: Sized,
    {
        let mapped = match self.map_ref_with(ToOwnedInfallibleMapper) {
            Ok(x) => x,
            Err(infallible) => match infallible {}, // Need to include because of MSRV
        };
        // SAFETY: All owned values don't make use of the lifetime parameter,
        // so we can safely transmute to 'static here.
        // I'd have liked to make this a where bound, but type resolution gets stuck in
        // an infinite loop in that case.
        unsafe {
            transmute::<Actual<'r, Self::Type, BeOwned>, Actual<'static, Self::Type, BeOwned>>(
                mapped,
            )
        }
    }

    /// A transparent clone is allowed for some types when doing method resolution.
    /// * For these types, a &a can be transparently cloned into an owned a.
    /// * For other types, an error is raised suggesting to use .clone() explicitly.
    ///
    /// See [TypeKind::supports_transparent_cloning] for more details.
    fn clone_to_owned_transparently<'r>(
        &'r self,
        span_range: SpanRange,
    ) -> ExecutionResult<Actual<'static, Self::Type, BeOwned>>
    where
        // Bounds for map_mut_with to work
        'a: 'r,
        Self::Type: IsHierarchicalType<
            Content<'a, Self::Form> = <Self::Form as IsFormOf<Self::Type>>::Content<'a>,
        >,
        Self::Form: IsHierarchicalForm,

        // Bounds for LeafAsRefForm to work
        Self::Form: LeafAsRefForm,

        // Bounds for cloning to work
        Self: Sized,
        BeOwned: for<'l> IsFormOf<Self::Type, Content<'l> = Self>,
    {
        let mapped = self.map_ref_with(ToOwnedTransparentlyMapper { span_range });
        // SAFETY: All owned values don't make use of the lifetime parameter,
        // so we can safely transmute to 'static here.
        // I'd have liked to make this a where bound, but type resolution gets stuck in
        // an infinite loop in that case.
        unsafe {
            #[allow(clippy::useless_transmute)]
            // Clippy thinks these types are identical but is wrong here
            transmute::<
                ExecutionResult<Actual<'r, Self::Type, BeOwned>>,
                ExecutionResult<Actual<'static, Self::Type, BeOwned>>,
            >(mapped)
        }
    }
}

impl<'a, X> IsSelfValueContent<'a> for X
where
    X: IsValueContent<'a>,
    X::Form: IsFormOf<X::Type, Content<'a> = X>,
{
}

// Clashes with other blanket impl it will replace!
//
// impl<
//     X: FromValueContent<'static, Type = T, Form = F>,
//     F: IsForm + MapFromArgument,
//     T: TypeData + DowncastFrom<ValueType, F>,
// > IsArgument for X {
//     type ValueType = T;
//     const OWNERSHIP: ArgumentOwnership = F::ARGUMENT_OWNERSHIP;
//     fn from_argument(Spanned(value, span_range): Spanned<ArgumentValue>) -> ExecutionResult<Self> {
//         let ownership_mapped = F::from_argument_value(value)?;
//         let type_mapped = T::resolve(ownership_mapped, span_range, "This argument")?;
//         Ok(X::from_actual(type_mapped))
//     }
// }

// Clashes with other blanket impl it will replace!
//
// impl<
//     X: IntoValueContent<'static, Type = T, Form = F>,
//     F: IsForm + MapIntoReturned,
//     T: UpcastTo<ValueType, F>,
// > IsReturnable for X {
//     fn to_returned_value(self) -> ExecutionResult<ReturnedValue> {
//         let type_mapped = self.into_actual()
//             .upcast::<ValueType>();
//         F::into_returned_value(type_mapped)
//     }
// }
