use super::*;

/// Shorthand for representing the form F of a type T with a particular lifetime 'a.
pub(crate) type Content<'a, T, F> = <T as IsHierarchicalType>::Content<'a, F>;
pub(crate) type DynContent<'a, D, F> =
    <F as IsDynCompatibleForm>::DynLeaf<'a, <D as IsDynType>::DynContent>;

/// For types which have an associated value (type and form)
pub(crate) trait IsValueContent {
    type Type: IsHierarchicalType;
    type Form: IsHierarchicalForm;
}

pub(crate) trait FromValueContent<'a>: IsValueContent {
    fn from_content(content: Content<'a, Self::Type, Self::Form>) -> Self;
}

pub(crate) trait FromSpannedValueContent<'a>: IsValueContent {
    fn from_spanned_content(content: Spanned<Content<'a, Self::Type, Self::Form>>) -> Self;
}

impl<'a, C: FromValueContent<'a>> FromSpannedValueContent<'a> for C {
    fn from_spanned_content(content: Spanned<Content<'a, Self::Type, Self::Form>>) -> Self {
        let Spanned(inner, _span_range) = content;
        C::from_content(inner)
    }
}

impl<C: IsValueContent> IsValueContent for Spanned<C> {
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

pub(crate) trait IntoValueContent<'a>: IsValueContent
where
    Self: Sized,
{
    fn into_content(self) -> Content<'a, Self::Type, Self::Form>;

    #[inline]
    fn upcast<S: IsHierarchicalType>(self) -> Content<'a, S, Self::Form>
    where
        Self::Type: UpcastTo<S>,
    {
        <Self::Type>::upcast_to::<Self::Form>(self.into_content())
    }

    #[inline]
    #[allow(clippy::type_complexity)] // It's actually pretty readable
    fn downcast<U>(self) -> Result<Content<'a, U, Self::Form>, Content<'a, Self::Type, Self::Form>>
    where
        U: DowncastFrom<Self::Type>,
    {
        U::downcast_from::<Self::Form>(self.into_content())
    }

    #[inline]
    fn into_any(self) -> Content<'a, AnyType, Self::Form>
    where
        Self::Type: UpcastTo<AnyType>,
    {
        self.upcast()
    }

    fn into_referenceable(self) -> Content<'a, Self::Type, BeReferenceable>
    where
        Self: IsValueContent<Form = BeOwned>,
    {
        map_via_leaf! {
            input: (Content<'a, Self::Type, Self::Form>) = self.into_content(),
            fn map_leaf<F = BeOwned, T>(leaf) -> (Content<'a, T, BeReferenceable>) {
                Rc::new(RefCell::new(leaf))
            }
        }
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
        resolution_target: &str,
    ) -> ExecutionResult<X>
    where
        <X as IsValueContent>::Type: DowncastFrom<C::Type>,
    {
        let Spanned(value, span_range) = self;
        let content = value.into_content();
        let resolved = <<X as IsValueContent>::Type>::resolve::<C::Form>(
            content,
            span_range,
            resolution_target,
        )?;
        Ok(X::from_spanned_content(Spanned(resolved, span_range)))
    }

    // TODO[concepts]: Change to use a FromSpannedDynContent trait,
    // so that it can return a ExecutionResult<X> and avoid needing to specify D.
    pub(crate) fn dyn_resolve<D: IsDynLeaf + ?Sized>(
        self,
        resolution_target: &str,
    ) -> ExecutionResult<DynContent<'a, D::Type, C::Form>>
    where
        <D as IsDynLeaf>::Type: DynResolveFrom<C::Type>,
        C::Form: IsDynCompatibleForm,
    {
        let Spanned(value, span_range) = self;
        let content = value.into_content();
        let resolved =
            <<D as IsDynLeaf>::Type>::resolve::<C::Form>(content, span_range, resolution_target)?;
        Ok(resolved)
    }
}

// TODO[concepts]: Remove eventually, along with IntoValue impl
impl<X: IntoValueContent<'static, Form = BeOwned>> IntoAnyValue for X
where
    X::Type: UpcastTo<AnyType>,
{
    fn into_any_value(self) -> AnyValue {
        self.into_any()
    }
}

/// Implementations on the value contents *themselves*.
/// Typically this is reserved for things operating on references - otherwise we can
/// implement on IntoValueContent / FromValueContent.
pub(crate) trait IsSelfValueContent<'a>: IsValueContent
where
    Self::Type: IsHierarchicalType<Content<'a, Self::Form> = Self>,
    Self::Form: IsHierarchicalForm,
{
    fn as_mut_value<'r>(&'r mut self) -> Content<'r, Self::Type, BeMut>
    where
        'a: 'r,
        Self::Form: LeafAsMutForm,
    {
        map_via_leaf! {
            input: &'r mut (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F: LeafAsMutForm, T>(leaf) -> (Content<'r, T, BeMut>) {
                F::leaf_as_mut(leaf)
            }
        }
    }

    fn as_ref_value<'r>(&'r self) -> Content<'r, Self::Type, BeRef>
    where
        'a: 'r,
        Self::Form: LeafAsRefForm,
    {
        map_via_leaf! {
            input: &'r (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F: LeafAsRefForm, T>(leaf) -> (Content<'r, T, BeRef>) {
                F::leaf_as_ref(leaf)
            }
        }
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
        map_via_leaf! {
            input: &'r (Content<'a, Self::Type, Self::Form>) = self,
            fn map_leaf<F: LeafAsRefForm, T>(leaf) -> (Content<'static, T, BeOwned>) {
                F::leaf_clone_to_owned_infallible(leaf)
            }
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
    ) -> ExecutionResult<Content<'static, Self::Type, BeOwned>>
    where
        'a: 'r,
        Self::Form: LeafAsRefForm,
        Self: Sized,
    {
        map_via_leaf! {
            input: &'r (Content<'a, Self::Type, Self::Form>) = self,
            state: SpanRange | let span_range = span_range,
            fn map_leaf<F: LeafAsRefForm, T>(leaf) -> (ExecutionResult<Content<'static, T, BeOwned>>) {
                F::leaf_clone_to_owned_transparently(leaf, span_range)
            }
        }
    }
}

impl<'a, X> IsSelfValueContent<'a> for X
where
    X: IsValueContent,
    X::Type: IsHierarchicalType<Content<'a, X::Form> = X>,
    X::Form: IsHierarchicalForm,
{
}

impl<
        X: FromValueContent<'static, Type = T, Form = F>,
        F: MapFromArgument,
        T: DowncastFrom<AnyType>,
    > IsArgument for X
{
    type ValueType = T;
    const OWNERSHIP: ArgumentOwnership = F::ARGUMENT_OWNERSHIP;
    fn from_argument(Spanned(value, span_range): Spanned<ArgumentValue>) -> ExecutionResult<Self> {
        let ownership_mapped = F::from_argument_value(value)?;
        let type_mapped = T::resolve(ownership_mapped, span_range, "This argument")?;
        Ok(X::from_content(type_mapped))
    }
}

impl<
        X: IntoValueContent<'static, Type = T, Form = BeOwned>,
        // TODO[concepts]: Migrate to BeOwned => F when it doesn't break MSRV
        // due to clashes with `IntoValueContent` on Shared<AnyValue> / Mutable<AnyValue>
        // F: MapIntoReturned,
        T: UpcastTo<AnyType>,
    > IsReturnable for X
{
    fn to_returned_value(self) -> ExecutionResult<ReturnedValue> {
        let type_mapped = self.into_content().upcast::<AnyType>();
        BeOwned::into_returned_value(type_mapped)
    }
}
