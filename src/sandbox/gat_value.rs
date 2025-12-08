#![allow(unused)]

// SUMMARY:
// - Strip SpanRange out of Owned/Shared etc.
// - Instead, have Spanned<T> wrapper type, which can be destructured with Spanned(value, span)
// - Implement something akin to the below:
//   - `type Owned<T> = Actual<'static, T, BeOwned>`
//   - `type CopyOnWrite<T> = Actual<'static, T, BeCopyOnWrite>`
//   - `type Shared<T> = Actual<'static, T, BeShared>`
//
// - For mapping outputs to methods/functions, we can do:
//   `<Output as IsValueContent<'static>>::into_actual().map_type::<ValueType>().into_returned_value()`
// - For mapping arguments to methods/functions, we can use the `IsArgument` implementation below

use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::internal_prelude::*;

trait IsOwnership: Sized {
    type Leaf<'a, T: IsLeaf>;
    type DynLeaf<'a, T: 'static + ?Sized>;
    const ARGUMENT_OWNERSHIP: ArgumentOwnership;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>>;
}

trait IsLeaf: 'static {
    fn into_iterable(self: Box<Self>) -> Option<Box<dyn IsIterable>> {
        None
    }
    fn as_iterable(&self) -> Option<&dyn IsIterable> {
        None
    }
}

struct BeOwned;
impl IsOwnership for BeOwned {
    type Leaf<'a, T: IsLeaf> = T;
    type DynLeaf<'a, T: 'static + ?Sized> = Box<T>;
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        // value.expect_owned()
        todo!()
    }
}

// Roughly equivalent to an owned, but wrapped so that it can be turned into a Shared/Mutable easily.
struct BeReferencable;
impl IsOwnership for BeReferencable {
    type Leaf<'a, T: IsLeaf> = Rc<RefCell<T>>;
    type DynLeaf<'a, T: 'static + ?Sized> = Rc<RefCell<T>>;

    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Owned;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        // Rc::new(RefCell::new(value.expect_owned()))
        todo!()
    }
}

struct BeAnyRef;
impl IsOwnership for BeAnyRef {
    type Leaf<'a, T: IsLeaf> = AnyRef<'a, T>;
    type DynLeaf<'a, T: 'static + ?Sized> = AnyRef<'a, T>;
    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Shared;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        todo!()
    }
}

struct BeAnyRefMut;
impl IsOwnership for BeAnyRefMut {
    type Leaf<'a, T: IsLeaf> = AnyRefMut<'a, T>;
    type DynLeaf<'a, T: 'static + ?Sized> = AnyRefMut<'a, T>;

    const ARGUMENT_OWNERSHIP: ArgumentOwnership = ArgumentOwnership::Mutable;

    fn from_argument_value(
        value: ArgumentValue,
    ) -> ExecutionResult<Actual<'static, ValueType, Self>> {
        todo!()
    }
}

struct OwnedToReferencableMapper;

trait OwnershipMapper {
    type HFrom: IsOwnership;
    type HTo: IsOwnership;

    fn map_leaf<'a, T: IsLeaf>(
        leaf: <Self::HFrom as IsOwnership>::Leaf<'a, T>,
    ) -> <Self::HTo as IsOwnership>::Leaf<'a, T>;
}

impl OwnershipMapper for OwnedToReferencableMapper {
    type HFrom = BeOwned;
    type HTo = BeReferencable;

    fn map_leaf<'a, T: IsLeaf>(
        leaf: <Self::HFrom as IsOwnership>::Leaf<'a, T>,
    ) -> <Self::HTo as IsOwnership>::Leaf<'a, T> {
        Rc::new(RefCell::new(leaf))
    }
}

trait IsType: Sized {
    type Content<'a, H: IsOwnership>;

    fn map_ownership<'a, M: OwnershipMapper>(
        content: Self::Content<'a, M::HFrom>,
    ) -> Self::Content<'a, M::HTo>;

    fn map_content<'a, H: IsOwnership, M: ContentMapper<H>>(
        content: Self::Content<'a, H>,
    ) -> M::TOut<'a>;

    fn type_name() -> &'static str {
        todo!()
    }
}

trait MapToType<T: IsType>: IsType {
    fn map_to_type<'a, H: IsOwnership>(content: Self::Content<'a, H>) -> T::Content<'a, H>;
}

impl<T: IsChildType<ParentType = U>, U: MapToType<A>, A: IsType> MapToType<A> for T {
    fn map_to_type<'a, H: IsOwnership>(content: Self::Content<'a, H>) -> A::Content<'a, H> {
        U::map_to_type(T::into_parent(content))
    }
}

trait MaybeMapFromType<T: IsType, H: IsOwnership>: IsType {
    fn maybe_map_from_type<'a>(content: T::Content<'a, H>) -> Option<Self::Content<'a, H>>;

    fn resolve<'a>(
        actual: Actual<'a, T, H>,
        span_range: SpanRange,
        context: &str,
    ) -> ExecutionResult<Actual<'a, Self, H>> {
        let content = match Self::maybe_map_from_type(actual.0) {
            Some(c) => c,
            None => {
                return span_range.value_err(format!(
                    "{} cannot be mapped to {}",
                    context,
                    T::type_name(),
                ))
            }
        };
        Ok(Actual(content))
    }
}

// *sigh*
// To avoid implementation conflict errors, we have to implement for each H separately
impl<T: IsChildType<ParentType = U>, U: MaybeMapFromType<A, BeOwned>, A: IsType>
    MaybeMapFromType<A, BeOwned> for T
{
    fn maybe_map_from_type<'a>(
        content: A::Content<'a, BeOwned>,
    ) -> Option<Self::Content<'a, BeOwned>> {
        T::from_parent(U::maybe_map_from_type(content)?)
    }
}

impl<T: IsChildType<ParentType = U>, U: MaybeMapFromType<A, BeAnyRef>, A: IsType>
    MaybeMapFromType<A, BeAnyRef> for T
{
    fn maybe_map_from_type<'a>(
        content: A::Content<'a, BeAnyRef>,
    ) -> Option<Self::Content<'a, BeAnyRef>> {
        T::from_parent(U::maybe_map_from_type(content)?)
    }
}

// TODO: Add for BeReferencable, BeAnyRef, BeAnyRefMut etc.

impl MapToType<ValueType> for ValueType {
    fn map_to_type<'a, H: IsOwnership>(content: Self::Content<'a, H>) -> Self::Content<'a, H> {
        content
    }
}

impl<H: IsOwnership> MaybeMapFromType<ValueType, H> for ValueType {
    fn maybe_map_from_type<'a>(content: Self::Content<'a, H>) -> Option<Self::Content<'a, H>> {
        Some(content)
    }
}

trait IsChildType: IsType {
    type ParentType: IsType;
    fn into_parent<'a, H: IsOwnership>(
        content: Self::Content<'a, H>,
    ) -> <Self::ParentType as IsType>::Content<'a, H>;
    fn from_parent<'a, H: IsOwnership>(
        content: <Self::ParentType as IsType>::Content<'a, H>,
    ) -> Option<Self::Content<'a, H>>;
}

struct ValueType;

impl IsType for ValueType {
    type Content<'a, H: IsOwnership> = ValueContent<'a, H>;

    fn map_ownership<'a, M: OwnershipMapper>(
        content: Self::Content<'a, M::HFrom>,
    ) -> Self::Content<'a, M::HTo> {
        match content {
            ValueContent::Integer(x) => ValueContent::Integer(x.map_ownership::<M>()),
            ValueContent::Object(x) => ValueContent::Object(x.map_ownership::<M>()),
        }
    }

    fn map_content<'a, H: IsOwnership, M: ContentMapper<H>>(
        content: Self::Content<'a, H>,
    ) -> M::TOut<'a> {
        match content {
            ValueContent::Integer(x) => x.map_content::<M>(),
            ValueContent::Object(x) => x.map_content::<M>(),
        }
    }
}

enum ValueContent<'a, H: IsOwnership> {
    Integer(Actual<'a, IntegerType, H>),
    Object(Actual<'a, ObjectType, H>),
    // ...
}

struct Actual<'a, K: IsType, H: IsOwnership>(K::Content<'a, H>);

impl<'a, K: IsType, H: IsOwnership> Actual<'a, K, H> {
    #[inline]
    fn of(content: impl IntoValueContent<'a, TypeData = K, Ownership = H>) -> Self {
        Actual(content.into_content())
    }

    #[inline]
    fn map_ownership<M: OwnershipMapper<HFrom = H>>(self) -> Actual<'a, K, M::HTo> {
        Actual(K::map_ownership::<M>(self.0))
    }

    #[inline]
    fn map_content<M: ContentMapper<H>>(self) -> M::TOut<'a> {
        K::map_content::<'a, H, M>(self.0)
    }

    fn map_type<U: IsType>(self) -> Actual<'a, U, H>
    where
        K: MapToType<U>,
    {
        Actual(K::map_to_type(self.0))
    }

    fn map_type_maybe<U: MaybeMapFromType<K, H>>(self) -> Option<Actual<'a, U, H>> {
        Some(Actual(U::maybe_map_from_type(self.0)?))
    }
}

impl<'a, K: IsType> Actual<'a, K, BeOwned> {
    fn into_referencable(self) -> Actual<'a, K, BeReferencable> {
        self.map_ownership::<OwnedToReferencableMapper>()
    }
}

impl<'a, K: IsType + MapToType<ValueType>, H: IsOwnership> Actual<'a, K, H> {
    fn into_value(self) -> Actual<'a, ValueType, H> {
        self.map_type()
    }
}

type Value = Actual<'static, ValueType, BeOwned>;
type ValueReferencable = Actual<'static, ValueType, BeReferencable>;
type ValueRef<'a> = Actual<'a, ValueType, BeAnyRef>;
type ValueMut<'a> = Actual<'a, ValueType, BeAnyRefMut>;

struct ObjectValue;

impl IsLeaf for ObjectValue {}

struct ObjectType;

impl IsType for ObjectType {
    type Content<'a, H: IsOwnership> = H::Leaf<'a, ObjectValue>;

    fn map_ownership<'a, M: OwnershipMapper>(
        content: Self::Content<'a, M::HFrom>,
    ) -> Self::Content<'a, M::HTo> {
        M::map_leaf(content)
    }

    fn map_content<'a, H: IsOwnership, M: ContentMapper<H>>(
        content: Self::Content<'a, H>,
    ) -> M::TOut<'a> {
        M::map_leaf(content)
    }
}

enum IntegerContent<'a, H: IsOwnership> {
    U32(Actual<'a, U32Type, H>),
    // ...
}

struct IntegerType;

impl IsType for IntegerType {
    type Content<'a, H: IsOwnership> = IntegerContent<'a, H>;

    fn map_ownership<'a, M: OwnershipMapper>(
        content: Self::Content<'a, M::HFrom>,
    ) -> Self::Content<'a, M::HTo> {
        match content {
            IntegerContent::U32(i) => IntegerContent::U32(i.map_ownership::<M>()),
        }
    }

    fn map_content<'a, H: IsOwnership, M: ContentMapper<H>>(
        content: Self::Content<'a, H>,
    ) -> M::TOut<'a> {
        match content {
            IntegerContent::U32(x) => x.map_content::<M>(),
        }
    }
}

impl IsChildType for IntegerType {
    type ParentType = ValueType;

    fn into_parent<'a, H: IsOwnership>(
        content: Self::Content<'a, H>,
    ) -> <Self::ParentType as IsType>::Content<'a, H> {
        ValueContent::Integer(Actual(content))
    }

    fn from_parent<'a, H: IsOwnership>(
        content: <Self::ParentType as IsType>::Content<'a, H>,
    ) -> Option<Self::Content<'a, H>> {
        match content {
            ValueContent::Integer(i) => Some(i.0),
            _ => None,
        }
    }
}

impl IsLeaf for u32 {
    fn into_iterable(self: Box<Self>) -> Option<Box<dyn IsIterable>> {
        Some(self)
    }
    fn as_iterable(&self) -> Option<&dyn IsIterable> {
        Some(self)
    }
}

impl IsIterable for u32 {
    fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue> {
        Ok(IteratorValue::new_any(std::iter::once(
            (*self).into_value(),
        )))
    }

    fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize> {
        Ok(0)
    }
}

struct U32Type;

impl IsType for U32Type {
    type Content<'a, H: IsOwnership> = H::Leaf<'a, u32>;

    fn map_ownership<'a, M: OwnershipMapper>(
        content: Self::Content<'a, M::HFrom>,
    ) -> Self::Content<'a, M::HTo> {
        M::map_leaf(content)
    }

    fn map_content<'a, H: IsOwnership, M: ContentMapper<H>>(
        content: Self::Content<'a, H>,
    ) -> M::TOut<'a> {
        M::map_leaf(content)
    }
}

impl IsChildType for U32Type {
    type ParentType = IntegerType;

    fn into_parent<'a, H: IsOwnership>(
        content: Self::Content<'a, H>,
    ) -> <Self::ParentType as IsType>::Content<'a, H> {
        IntegerContent::U32(Actual(content))
    }

    fn from_parent<'a, H: IsOwnership>(
        content: <Self::ParentType as IsType>::Content<'a, H>,
    ) -> Option<Self::Content<'a, H>> {
        match content {
            IntegerContent::U32(i) => Some(i.0),
            _ => None,
        }
    }
}

impl<H: IsOwnership> MaybeMapFromType<U32Type, H> for U32Type {
    fn maybe_map_from_type<'a>(content: Self::Content<'a, H>) -> Option<Self::Content<'a, H>> {
        Some(content)
    }
}

#[test]
fn test() {
    let my_u32 = Actual::of(42u32);
    let as_value = my_u32.map_type::<ValueType>();
    let back_to_u32 = as_value.map_type_maybe::<U32Type>().unwrap();
    assert_eq!(back_to_u32.0, 42u32);
}

// Clashes with other blanket trait it will replace!
//
// impl<
//     X: FromValueContent<'static, TypeData = T, Ownership = H>,
//     T: MaybeMapFromType<ValueType> + HierarchicalTypeData,
//     H: IsOwnership,
// > IsArgument for X {
//     type ValueType = T;
//     const OWNERSHIP: ArgumentOwnership = H::ARGUMENT_OWNERSHIP;

//     fn from_argument(value: ArgumentValue) -> ExecutionResult<Self> {
//         let span_range = value.span_range();
//         let ownership_mapped = H::from_argument_value(value)?;
//         let type_mapped = T::resolve(ownership_mapped, span_range, "This argument")?;
//         Ok(X::from_actual(type_mapped))
//     }
// }

trait IsValueContent<'a> {
    type TypeData: IsType;
    type Ownership: IsOwnership;
}

trait IntoValueContent<'a>: IsValueContent<'a> {
    fn into_content(self) -> <Self::TypeData as IsType>::Content<'a, Self::Ownership>;

    #[inline]
    fn into_actual(self) -> Actual<'a, Self::TypeData, Self::Ownership>
    where
        Self: Sized,
    {
        Actual::of(self)
    }
}

trait FromValueContent<'a>: IsValueContent<'a> {
    fn from_content(content: <Self::TypeData as IsType>::Content<'a, Self::Ownership>) -> Self;

    #[inline]
    fn from_actual(actual: Actual<'a, Self::TypeData, Self::Ownership>) -> Self
    where
        Self: Sized,
    {
        Self::from_content(actual.0)
    }
}

impl<'a, T: IsType, H: IsOwnership> IsValueContent<'a> for Actual<'a, T, H> {
    type TypeData = T;
    type Ownership = H;
}

impl<'a, T: IsType, H: IsOwnership> IntoValueContent<'a> for Actual<'a, T, H> {
    fn into_content(self) -> <Self::TypeData as IsType>::Content<'a, Self::Ownership> {
        self.0
    }
}

impl<'a, T: IsType, H: IsOwnership> FromValueContent<'a> for Actual<'a, T, H> {
    fn from_content(content: <Self::TypeData as IsType>::Content<'a, Self::Ownership>) -> Self {
        Actual(content)
    }
}

impl<'a> IsValueContent<'a> for u32 {
    type TypeData = U32Type;
    type Ownership = BeOwned;
}

impl<'a> IntoValueContent<'a> for u32 {
    fn into_content(self) -> u32 {
        self
    }
}

impl<'a> FromValueContent<'a> for u32 {
    fn from_content(content: u32) -> Self {
        content
    }
}

impl<'a, H: IsOwnership> IsValueContent<'a> for ValueContent<'a, H> {
    type TypeData = ValueType;
    type Ownership = H;
}

impl<'a, H: IsOwnership> IntoValueContent<'a> for ValueContent<'a, H> {
    fn into_content(self) -> <Self::TypeData as IsType>::Content<'a, Self::Ownership> {
        self
    }
}

struct IterableType;

trait IsIterable: 'static {
    fn into_iterator(self: Box<Self>) -> ExecutionResult<IteratorValue>;
    fn len(&self, error_span_range: SpanRange) -> ExecutionResult<usize>;
}

impl IsType for IterableType {
    type Content<'a, H: IsOwnership> = H::DynLeaf<'a, dyn IsIterable>;

    fn map_ownership<'a, M: OwnershipMapper>(
        content: Self::Content<'a, M::HFrom>,
    ) -> Self::Content<'a, M::HTo> {
        // Might need to create separate IsMappableType trait and not impl it for dyn types
        // And/or have a separate dyn mapping facility if we need it
        todo!();
    }

    fn map_content<'a, H: IsOwnership, M: ContentMapper<H>>(
        content: Self::Content<'a, H>,
    ) -> M::TOut<'a> {
        // Might need to create separate IsMappableType trait and not impl it for dyn types
        // And/or have a separate dyn mapping facility if we need it
        todo!()
    }
}

impl<T: IsType> MaybeMapFromType<T, BeOwned> for IterableType {
    fn maybe_map_from_type<'a>(
        content: <T as IsType>::Content<'a, BeOwned>,
    ) -> Option<Self::Content<'a, BeOwned>> {
        T::map_content::<'a, BeOwned, IterableMapper>(content)
    }
}

impl<T: IsType> MaybeMapFromType<T, BeAnyRef> for IterableType {
    fn maybe_map_from_type<'a>(
        content: <T as IsType>::Content<'a, BeAnyRef>,
    ) -> Option<Self::Content<'a, BeAnyRef>> {
        T::map_content::<'a, BeAnyRef, IterableMapper>(content)
    }
}

trait ContentMapper<H: IsOwnership> {
    type TOut<'a>;

    fn map_leaf<'a, L: IsLeaf>(leaf: H::Leaf<'a, L>) -> Self::TOut<'a>;
}

struct IterableMapper;

impl ContentMapper<BeOwned> for IterableMapper {
    type TOut<'a> = Option<Box<dyn IsIterable>>;

    fn map_leaf<'a, L: IsLeaf>(leaf: L) -> Self::TOut<'a> {
        L::into_iterable(Box::new(leaf))
    }
}

impl ContentMapper<BeAnyRef> for IterableMapper {
    type TOut<'a> = Option<AnyRef<'a, dyn IsIterable>>;

    fn map_leaf<'a, L: IsLeaf>(leaf: <BeAnyRef as IsOwnership>::Leaf<'a, L>) -> Self::TOut<'a> {
        leaf.map_optional(|x| L::as_iterable(x))
    }
}

#[test]
fn test_iterable_mapping() {
    let my_value = Actual::of(42u32);
    // let my_value = my_value.map_type::<ValueType>();
    let as_iterable = my_value.map_type_maybe::<IterableType>().unwrap();
    let iterator = as_iterable.0.into_iterator().unwrap();
    let collected: Vec<u32> = iterator
        .map(|v| Owned::new(v).resolve_as("u32").unwrap())
        .collect();
    assert_eq!(collected, vec![42u32]);
}
