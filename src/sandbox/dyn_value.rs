#![allow(unused)]

use std::any::Any;
trait IsValue: Any {
    type TypeData: 'static + TypeData<Value = Self>;
}
trait DynValue: Any {
    fn type_data(&self) -> &dyn DynTypeData;
}
impl<T: IsValue> DynValue for T {
    fn type_data(&self) -> &'static dyn DynTypeData {
        <T as IsValue>::TypeData::static_ref()
    }
}

trait IsIterable {
    fn do_something(self) -> String;
}

trait DynIterable {
    fn do_something(self: Box<Self>) -> String;
}

impl<T: IsIterable> DynIterable for T {
    fn do_something(self: Box<Self>) -> String {
        <T as IsIterable>::do_something(*self)
    }
}

enum IntegerValue {
    U32(u32),
    // Other integer types could be added here
}

impl IsValue for u32 {
    type TypeData = U32TypeData;
}
struct AnyRef<'a, T: ?Sized + 'static> {
    value: &'a T,
}

trait TypeData {
    type Value: 'static + IsValue<TypeData = Self>;

    fn static_ref() -> &'static dyn DynTypeData;

    fn as_value(value: Box<dyn DynValue>) -> Option<Self::Value> {
        let value_any: Box<dyn Any> = value;
        value_any.downcast::<Self::Value>().ok().map(|v| *v)
    }

    fn as_value_ref(value: AnyRef<dyn DynValue>) -> Option<AnyRef<Self::Value>> {
        let value_any: &dyn Any = value.value;
        value_any
            .downcast_ref::<Self::Value>()
            .map(|v| AnyRef { value: v })
    }

    fn as_integer(value: Self::Value) -> Option<IntegerValue> {
        let _ = value;
        None
    }

    fn as_iterable(value: Self::Value) -> Option<Box<dyn DynIterable>> {
        let _ = value;
        None
    }
}

trait DynTypeData {
    fn as_integer(&self, value: Box<dyn DynValue>) -> Option<IntegerValue>;
    fn as_iterable(&self, value: Box<dyn DynValue>) -> Option<Box<dyn DynIterable>>;
}

impl<T: TypeData> DynTypeData for T {
    fn as_integer(&self, value: Box<dyn DynValue>) -> Option<IntegerValue> {
        T::as_integer(Self::as_value(value)?)
    }

    fn as_iterable(&self, value: Box<dyn DynValue>) -> Option<Box<dyn DynIterable>> {
        T::as_iterable(Self::as_value(value)?)
    }
}

struct U32TypeData;

impl TypeData for U32TypeData {
    type Value = u32;

    fn static_ref() -> &'static dyn DynTypeData {
        &Self
    }

    fn as_integer(value: Self::Value) -> Option<IntegerValue> {
        Some(IntegerValue::U32(value))
    }
}

struct ArrayValue;
impl IsValue for ArrayValue {
    type TypeData = ArrayTypeData;
}
impl IsIterable for ArrayValue {
    fn do_something(self) -> String {
        "ArrayValue iterable".to_string()
    }
}
struct ArrayTypeData;

impl TypeData for ArrayTypeData {
    type Value = ArrayValue;

    fn static_ref() -> &'static dyn DynTypeData {
        &Self
    }

    fn as_iterable(value: Self::Value) -> Option<Box<dyn DynIterable>> {
        Some(Box::new(value))
    }
}
