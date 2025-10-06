#![allow(unused)]
// TODO[unused-clearup]
use super::*;

pub(crate) trait FromResolved: Sized {
    type ValueType: HierarchicalTypeData;
    const OWNERSHIP: ResolvedValueOwnership;
    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self>;
}

impl<T: ResolvableArgumentOwned + ResolvableArgumentTarget> FromResolved for Owned<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        value
            .expect_owned()
            .try_map(|v, _| T::resolve_from_owned(v))
    }
}

impl<T: ResolvableArgumentShared + ResolvableArgumentTarget + ?Sized> FromResolved for Shared<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Shared;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        value.expect_shared().try_map(|v, _| T::resolve_from_ref(v))
    }
}

impl<T: 'static + ?Sized> FromResolved for Ref<'static, T>
where
    Shared<T>: FromResolved,
{
    type ValueType = <Shared<T> as FromResolved>::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = <Shared<T> as FromResolved>::OWNERSHIP;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(Shared::<T>::from_resolved(value)?.into())
    }
}

impl<T: ResolvableArgumentMutable + ResolvableArgumentTarget + ?Sized> FromResolved for Mutable<T> {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Mutable;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        value
            .expect_mutable()
            .try_map(|v, _| T::resolve_from_mut(v))
    }
}

impl<T: 'static + ?Sized> FromResolved for RefMut<'static, T>
where
    Mutable<T>: FromResolved,
{
    type ValueType = <Mutable<T> as FromResolved>::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = <Mutable<T> as FromResolved>::OWNERSHIP;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        Ok(Mutable::<T>::from_resolved(value)?.into())
    }
}

impl<T: ResolvableArgumentOwned + ResolvableArgumentTarget> FromResolved for T {
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        T::resolve_from_owned(value.expect_owned().into_inner())
    }
}

impl<T: ResolvableArgumentShared + ResolvableArgumentTarget + ToOwned> FromResolved
    for CopyOnWrite<T>
where
    T::Owned: ResolvableArgumentOwned,
{
    type ValueType = T::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::CopyOnWrite;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        value.expect_copy_on_write().map_any(
            T::resolve_shared,
            <T::Owned as ResolvableArgumentOwned>::resolve_owned,
        )
    }
}

impl<T: FromResolved> FromResolved for Spanned<T> {
    type ValueType = <T as FromResolved>::ValueType;
    const OWNERSHIP: ResolvedValueOwnership = <T as FromResolved>::OWNERSHIP;

    fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
        let span_range = value.span_range();
        Ok(Spanned {
            value: T::from_resolved(value)?,
            span_range,
        })
    }
}

pub(crate) trait ResolveAs<T> {
    fn resolve_as(self) -> ExecutionResult<T>;
}

impl<T: ResolvableArgumentOwned> ResolveAs<T> for ExpressionValue {
    fn resolve_as(self) -> ExecutionResult<T> {
        T::resolve_from_owned(self)
    }
}

impl<'a, T: ResolvableArgumentShared + ?Sized> ResolveAs<&'a T> for &'a ExpressionValue {
    fn resolve_as(self) -> ExecutionResult<&'a T> {
        T::resolve_from_ref(self)
    }
}

impl<'a, T: ResolvableArgumentMutable + ?Sized> ResolveAs<&'a mut T> for &'a mut ExpressionValue {
    fn resolve_as(self) -> ExecutionResult<&'a mut T> {
        T::resolve_from_mut(self)
    }
}

pub(crate) trait ResolvableArgumentTarget {
    type ValueType: HierarchicalTypeData;
}

pub(crate) trait ResolvableArgumentOwned: Sized {
    fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self>;
    fn resolve_owned(value: Owned<ExpressionValue>) -> ExecutionResult<Owned<Self>> {
        value.try_map(|v, _| Self::resolve_from_owned(v))
    }
}

pub(crate) trait ResolvableArgumentShared {
    fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self>;
    fn resolve_shared(value: Shared<ExpressionValue>) -> ExecutionResult<Shared<Self>> {
        value.try_map(|v, _| Self::resolve_from_ref(v))
    }
}

pub(crate) trait ResolvableArgumentMutable {
    fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self>;
    fn resolve_mutable(value: Mutable<ExpressionValue>) -> ExecutionResult<Mutable<Self>> {
        value.try_map(|v, _| Self::resolve_from_mut(v))
    }
}

impl ResolvableArgumentTarget for ExpressionValue {
    type ValueType = ValueTypeData;
}
impl ResolvableArgumentOwned for ExpressionValue {
    fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
        Ok(value)
    }
}
impl ResolvableArgumentShared for ExpressionValue {
    fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self> {
        Ok(value)
    }
}
impl ResolvableArgumentMutable for ExpressionValue {
    fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
        Ok(value)
    }
}

macro_rules! impl_resolvable_argument_for {
    ($value_type:ty, ($value:ident) -> $type:ty $body:block) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_owned($value: ExpressionValue) -> ExecutionResult<Self> {
                $body
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref($value: &ExpressionValue) -> ExecutionResult<&Self> {
                $body
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut($value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
                $body
            }
        }
    };
}

macro_rules! impl_delegated_resolvable_argument_for {
    ($value_type:ty, ($value:ident: $delegate:ty) -> $type:ty { $expr:expr }) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_owned(input_value: ExpressionValue) -> ExecutionResult<Self> {
                let $value: $delegate = input_value.resolve_as()?;
                Ok($expr)
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref(input_value: &ExpressionValue) -> ExecutionResult<&Self> {
                let $value: &$delegate = input_value.resolve_as()?;
                Ok(&$expr)
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut(input_value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
                let $value: &mut $delegate = input_value.resolve_as()?;
                Ok(&mut $expr)
            }
        }
    };
}

pub(crate) use impl_resolvable_argument_for;

impl_resolvable_argument_for! {
    BooleanTypeData,
    (value) -> ExpressionBoolean {
        match value {
            ExpressionValue::Boolean(value) => Ok(value),
            _ => value.execution_err("Expected boolean"),
        }
    }
}

impl_delegated_resolvable_argument_for! {
    BooleanTypeData,
    (value: ExpressionBoolean) -> bool { value.value }
}

// Integer types
impl_resolvable_argument_for! {
    IntegerTypeData,
    (value) -> ExpressionInteger {
        match value {
            ExpressionValue::Integer(value) => Ok(value),
            _ => value.execution_err("Expected integer"),
        }
    }
}

pub(crate) struct UntypedIntegerFallback(pub FallbackInteger);

impl ResolvableArgumentTarget for UntypedIntegerFallback {
    type ValueType = UntypedIntegerTypeData;
}

impl ResolvableArgumentOwned for UntypedIntegerFallback {
    fn resolve_from_owned(input_value: ExpressionValue) -> ExecutionResult<Self> {
        let value: UntypedInteger = input_value.resolve_as()?;
        Ok(UntypedIntegerFallback(value.parse_fallback()?))
    }
}

impl_resolvable_argument_for! {
    UntypedIntegerTypeData,
    (value) -> UntypedInteger {
        match value {
            ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::Untyped(x), ..}) => Ok(x),
            _ => value.execution_err("Expected untyped integer"),
        }
    }
}

macro_rules! impl_resolvable_integer_subtype {
    ($value_type:ty, $type:ty, $variant:ident, $expected_msg:expr) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
                match value {
                    ExpressionValue::Integer(ExpressionInteger {
                        value: ExpressionIntegerValue::Untyped(x),
                        ..
                    }) => x.parse_as(),
                    ExpressionValue::Integer(ExpressionInteger {
                        value: ExpressionIntegerValue::$variant(x),
                        ..
                    }) => Ok(x),
                    _ => value.execution_err($expected_msg),
                }
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self> {
                match value {
                    ExpressionValue::Integer(ExpressionInteger {
                        value: ExpressionIntegerValue::$variant(x),
                        ..
                    }) => Ok(x),
                    _ => value.execution_err($expected_msg),
                }
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
                match value {
                    ExpressionValue::Integer(ExpressionInteger {
                        value: ExpressionIntegerValue::$variant(x),
                        ..
                    }) => Ok(x),
                    _ => value.execution_err($expected_msg),
                }
            }
        }
    };
}

impl_resolvable_integer_subtype!(I8TypeData, i8, I8, "Expected i8");
impl_resolvable_integer_subtype!(I16TypeData, i16, I16, "Expected i16");
impl_resolvable_integer_subtype!(I32TypeData, i32, I32, "Expected i32");
impl_resolvable_integer_subtype!(I64TypeData, i64, I64, "Expected i64");
impl_resolvable_integer_subtype!(I128TypeData, i128, I128, "Expected i128");
impl_resolvable_integer_subtype!(IsizeTypeData, isize, Isize, "Expected isize");
impl_resolvable_integer_subtype!(U8TypeData, u8, U8, "Expected u8");
impl_resolvable_integer_subtype!(U16TypeData, u16, U16, "Expected u16");
impl_resolvable_integer_subtype!(U32TypeData, u32, U32, "Expected u32");
impl_resolvable_integer_subtype!(U64TypeData, u64, U64, "Expected u64");
impl_resolvable_integer_subtype!(U128TypeData, u128, U128, "Expected u128");
impl_resolvable_integer_subtype!(UsizeTypeData, usize, Usize, "Expected usize");

// Float types
impl_resolvable_argument_for! {
    FloatTypeData,
    (value) -> ExpressionFloat {
        match value {
            ExpressionValue::Float(value) => Ok(value),
            _ => value.execution_err("Expected float"),
        }
    }
}

pub(crate) struct UntypedFloatFallback(pub FallbackFloat);

impl ResolvableArgumentTarget for UntypedFloatFallback {
    type ValueType = UntypedFloatTypeData;
}

impl ResolvableArgumentOwned for UntypedFloatFallback {
    fn resolve_from_owned(input_value: ExpressionValue) -> ExecutionResult<Self> {
        let value: UntypedFloat = input_value.resolve_as()?;
        Ok(UntypedFloatFallback(value.parse_fallback()?))
    }
}

impl_resolvable_argument_for! {
    UntypedFloatTypeData,
    (value) -> UntypedFloat {
        match value {
            ExpressionValue::Float(ExpressionFloat { value: ExpressionFloatValue::Untyped(x), ..}) => Ok(x),
            _ => value.execution_err("Expected untyped float"),
        }
    }
}

macro_rules! impl_resolvable_float_subtype {
    ($value_type:ty, $type:ty, $variant:ident, $expected_msg:expr) => {
        impl ResolvableArgumentTarget for $type {
            type ValueType = $value_type;
        }

        impl ResolvableArgumentOwned for $type {
            fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
                match value {
                    ExpressionValue::Float(ExpressionFloat {
                        value: ExpressionFloatValue::Untyped(x),
                        ..
                    }) => x.parse_as(),
                    ExpressionValue::Float(ExpressionFloat {
                        value: ExpressionFloatValue::$variant(x),
                        ..
                    }) => Ok(x),
                    _ => value.execution_err($expected_msg),
                }
            }
        }

        impl ResolvableArgumentShared for $type {
            fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self> {
                match value {
                    ExpressionValue::Float(ExpressionFloat {
                        value: ExpressionFloatValue::$variant(x),
                        ..
                    }) => Ok(x),
                    _ => value.execution_err($expected_msg),
                }
            }
        }

        impl ResolvableArgumentMutable for $type {
            fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
                match value {
                    ExpressionValue::Float(ExpressionFloat {
                        value: ExpressionFloatValue::$variant(x),
                        ..
                    }) => Ok(x),
                    _ => value.execution_err($expected_msg),
                }
            }
        }
    };
}

impl_resolvable_float_subtype!(F32TypeData, f32, F32, "Expected f32");
impl_resolvable_float_subtype!(F64TypeData, f64, F64, "Expected f64");

impl_resolvable_argument_for! {
    StringTypeData,
    (value) -> ExpressionString {
        match value {
            ExpressionValue::String(value) => Ok(value),
            _ => value.execution_err("Expected string"),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    StringTypeData,
    (value: ExpressionString) -> String { value.value }
);

impl ResolvableArgumentTarget for str {
    type ValueType = StringTypeData;
}

impl ResolvableArgumentShared for str {
    fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self> {
        match value {
            ExpressionValue::String(s) => Ok(s.value.as_str()),
            _ => value.execution_err("Expected string"),
        }
    }
}

impl_resolvable_argument_for! {
    CharTypeData,
    (value) -> ExpressionChar {
        match value {
            ExpressionValue::Char(value) => Ok(value),
            _ => value.execution_err("Expected char"),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    CharTypeData,
    (value: ExpressionChar) -> char { value.value }
);

impl_resolvable_argument_for! {
    ArrayTypeData,
    (value) -> ExpressionArray {
        match value {
            ExpressionValue::Array(value) => Ok(value),
            _ => value.execution_err("Expected array"),
        }
    }
}

impl_resolvable_argument_for! {
    ObjectTypeData,
    (value) -> ExpressionObject {
        match value {
            ExpressionValue::Object(value) => Ok(value),
            _ => value.execution_err("Expected object"),
        }
    }
}

impl_resolvable_argument_for! {
    StreamTypeData,
    (value) -> ExpressionStream {
        match value {
            ExpressionValue::Stream(value) => Ok(value),
            _ => value.execution_err("Expected stream"),
        }
    }
}

impl_delegated_resolvable_argument_for!(
    StreamTypeData,
    (value: ExpressionStream) -> OutputStream { value.value }
);

impl_resolvable_argument_for! {
    RangeTypeData,
    (value) -> ExpressionRange {
        match value {
            ExpressionValue::Range(value) => Ok(value),
            _ => value.execution_err("Expected range"),
        }
    }
}

impl ResolvableArgumentTarget for IterableValue {
    type ValueType = IterableTypeData;
}

impl ResolvableArgumentOwned for IterableValue {
    fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
        Ok(match value {
            ExpressionValue::Array(x) => Self::Array(x),
            ExpressionValue::Object(x) => Self::Object(x),
            ExpressionValue::Stream(x) => Self::Stream(x),
            ExpressionValue::Range(x) => Self::Range(x),
            ExpressionValue::Iterator(x) => Self::Iterator(x),
            ExpressionValue::String(x) => Self::String(x),
            _ => {
                return value.execution_err(
                    "Expected iterable (iterator, array, object, stream, range or string)",
                )
            }
        })
    }
}

impl_resolvable_argument_for! {
    IteratorTypeData,
    (value) -> ExpressionIterator {
        match value {
            ExpressionValue::Iterator(value) => Ok(value),
            _ => value.execution_err("Expected iterator"),
        }
    }
}
