use super::*;

/// Shorthand for representing the form F of a type T with a particular lifetime 'a.
pub(crate) type Content<'a, T, F> = <T as IsHierarchicalType>::Content<'a, F>;
pub(crate) type DynContent<'a, D, F> =
    <F as IsDynCompatibleForm>::DynLeaf<'a, <D as IsDynType>::DynContent>;

/// For types which have an associated value (type and form)
pub(crate) trait IsValueContent<'a> {
    type Type: IsHierarchicalType;
    type Form: IsHierarchicalForm;
}

pub(crate) trait FromValueContent<'a>: IsValueContent<'a> {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self;
}

pub(crate) trait FromSpannedValueContent<'a>: IsValueContent<'a> {
    fn from_spanned_content(content: Spanned<Content<'a, Self::Type, Self::Form>>) -> Self;
}

impl<'a, C: FromValueContent<'a>> FromSpannedValueContent<'a> for C {
    fn from_spanned_content(content: Spanned<Content<'a, Self::Type, Self::Form>>) -> Self {
        let Spanned(inner, _span_range) = content;
        C::from_content(inner)
    }
}

impl<'a, C: IsValueContent<'a>> IsValueContent<'a> for Spanned<C> {
    type Type = C::Type;
    type Form = C::Form;
}

impl<'a, C: FromValueContent<'a>> FromSpannedValueContent<'a> for Spanned<C> {
    fn from_spanned_content(
        Spanned(content, span_range): Spanned<Content<'a, Self::Type, Self::Form>>,
    ) -> Self {
        Spanned(C::from_content(content), span_range)
    }
}

pub(crate) trait IntoValueContent<'a>: IsValueContent<'a>
where
    Self: Sized,
{
    fn into_content(self) -> Content<'a, Self::Type, Self::Form>;

    #[inline]
    fn upcast<S: IsHierarchicalType>(self) -> Content<'a, S, Self::Form>
    where
        Self::Type: UpcastTo<S, Self::Form>,
    {
        <Self::Type>::upcast_to(self.into_content())
    }

    #[inline]
    fn downcast<U>(self) -> Option<Content<'a, U, Self::Form>>
    where
        U: DowncastFrom<Self::Type, Self::Form>,
    {
        U::downcast_from(self.into_content()).ok()
    }

    #[inline]
    fn into_any(self) -> Content<'a, AnyType, Self::Form>
    where
        Self::Type: UpcastTo<AnyType, Self::Form>,
    {
        self.upcast()
    }

    fn map_with<M: LeafMapper<Self::Form>>(self, mapper: M) -> M::Output<'a, Self::Type> {
        <Self::Type>::map_with::<Self::Form, M>(mapper, self.into_content())
    }

    fn into_referenceable(self) -> Content<'a, Self::Type, BeReferenceable>
    where
        for<'l> OwnedToReferenceableMapper: LeafMapper<
            Self::Form,
            Output<'l, Self::Type> = Content<'l, Self::Type, BeReferenceable>,
        >,
    {
        self.map_with(OwnedToReferenceableMapper)
    }
}

impl<'a, C> Spanned<C>
where
    C: IntoValueContent<'a>,
    C::Type: IsHierarchicalType,
    C::Form: IsHierarchicalForm,
{
    pub(crate) fn downcast_resolve<X: FromSpannedValueContent<'a, Form = C::Form>>(
        self,
        description: &str,
    ) -> ExecutionResult<X>
    where
        <X as IsValueContent<'a>>::Type: DowncastFrom<C::Type, C::Form>,
    {
        let Spanned(value, span_range) = self;
        let content = value.into_content();
        let resolved =
            <<X as IsValueContent<'a>>::Type>::resolve(content, span_range, description)?;
        Ok(X::from_spanned_content(Spanned(resolved, span_range)))
    }
}

// TODO[concepts]: Remove eventually, along with IntoValue impl
impl<X: IntoValueContent<'static, Form = BeOwned>> IntoAnyValue for X
where
    X::Type: UpcastTo<AnyType, BeOwned>,
{
    fn into_any_value(self) -> AnyValue {
        self.into_any()
    }
}

/// Implementations on the value contents *themselves*.
/// Typically this is reserved for things operating on references - otherwise we can
/// implement on IntoValueContent / FromValueContent.
pub(crate) trait IsSelfValueContent<'a>: IsValueContent<'a>
where
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
    Self::Form: IsHierarchicalForm,
{
    fn map_mut_with<'r, M: MutLeafMapper<Self::Form>>(
        &'r mut self,
        mapper: M,
    ) -> M::Output<'r, 'a, Self::Type>
    where
        'a: 'r,
    {
        <Self::Type>::map_mut_with::<Self::Form, M>(mapper, self)
    }

    fn map_ref_with<'r, M: RefLeafMapper<Self::Form>>(
        &'r self,
        mapper: M,
    ) -> M::Output<'r, 'a, Self::Type>
    where
        'a: 'r,
    {
        <Self::Type>::map_ref_with::<Self::Form, M>(mapper, self)
    }

    fn as_mut_value<'r>(&'r mut self) -> Content<'r, Self::Type, BeMut>
    where
        'a: 'r,
        Self::Form: LeafAsMutForm,
    {
        self.map_mut_with(ToMutMapper)
    }

    fn as_ref_value<'r>(&'r self) -> Content<'r, Self::Type, BeRef>
    where
        'a: 'r,
        Self::Form: LeafAsRefForm,
    {
        self.map_ref_with(ToRefMapper)
    }

    /// This method should only be used when you are certain that the value should be cloned.
    /// In most situations, you may wish to use [IsSelfValueContent::clone_to_owned_transparently]
    /// instead.
    fn clone_to_owned_infallible<'r>(&'r self) -> Content<'static, Self::Type, BeOwned>
    where
        'a: 'r,
        Self::Form: LeafAsRefForm,
        Self: Sized,
    {
        self.map_ref_with(ToOwnedInfallibleMapper)
    }

    /// A transparent clone is allowed for some types when doing method resolution.
    /// * For these types, a &a can be transparently cloned into an owned a.
    /// * For other types, an error is raised suggesting to use .clone() explicitly.
    ///
    /// See [TypeKind::supports_transparent_cloning] for more details.
    fn clone_to_owned_transparently<'r>(
        &'r self,
        span_range: SpanRange,
    ) -> ExecutionResult<Content<'static, Self::Type, BeOwned>>
    where
        'a: 'r,
        Self::Form: LeafAsRefForm,
        Self: Sized,
    {
        self.map_ref_with(ToOwnedTransparentlyMapper { span_range })
    }
}

impl<'a, X> IsSelfValueContent<'a> for X
where
    X: IsValueContent<'a>,
    X::Type: IsHierarchicalType<Content<'a, X::Form> = X>,
    X::Form: IsHierarchicalForm,
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
