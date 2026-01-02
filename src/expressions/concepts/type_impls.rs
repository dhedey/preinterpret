use super::*;

// NOTE: These should be moved out to the value definitions soon

pub(crate) type QqqValue = Actual<'static, ValueType, BeOwned>;
pub(crate) type QqqValueReferencable = Actual<'static, ValueType, BeReferenceable>;
pub(crate) type QqqValueRef<'a> = Actual<'a, ValueType, BeAnyRef>;
pub(crate) type QqqValueMut<'a> = Actual<'a, ValueType, BeAnyMut>;

define_parent_type! {
    pub(crate) ValueType,
    pub(crate) enum ValueContent {
        Integer => IntegerType,
        Object => ObjectType,
    },
    "any value",
}

define_parent_type! {
    pub(crate) IntegerType => ValueType(ValueContent::Integer),
    pub(crate) enum IntegerContent {
        U32 => U32Type,
        U64 => U64Type,
    },
    "an integer",
}

define_leaf_type! {
    pub(crate) U32Type => IntegerType(IntegerContent::U32, IntegerKind::U32) => ValueType,
    u32,
    "a u32",
}

define_leaf_type! {
    pub(crate) U64Type => IntegerType(IntegerContent::U64, IntegerKind::U64) => ValueType,
    u64,
    "a u64",
}

define_leaf_type! {
    pub(crate) ObjectType => ValueType(ValueContent::Object, ValueKind::Object),
    ObjectValue,
    "an object",
}
