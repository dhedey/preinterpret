use super::*;

// NOTE: These should be moved out to the value definitions soon

pub(crate) type QqqValue = Actual<'static, ValueType, BeOwned>;
pub(crate) type QqqValueReferencable = Actual<'static, ValueType, BeReferenceable>;
pub(crate) type QqqValueRef<'a> = Actual<'a, ValueType, BeAnyRef>;
pub(crate) type QqqValueMut<'a> = Actual<'a, ValueType, BeAnyMut>;

impl From<XxxValueKind> for ValueKind {
    fn from(_kind: XxxValueKind) -> Self {
        unimplemented!()
    }
}

define_parent_type! {
    pub(crate) ValueType,
    content: pub(crate) ValueContent,
    kind: pub(crate) XxxValueKind,
    parent_kind: ParentTypeKind::Value,
    variants: {
        Integer => IntegerType,
        Object => ObjectType,
    },
    type_name: "value",
    articled_display_name: "any value",
}

define_parent_type! {
    pub(crate) IntegerType => ValueType(ValueContent::Integer),
    content: pub(crate) IntegerContent,
    kind: pub(crate) XxxIntegerKind,
    parent_kind: ParentTypeKind::Integer,
    variants: {
        U32 => U32Type,
        U64 => U64Type,
    },
    type_name: "integer",
    articled_display_name: "an integer",
}

define_leaf_type! {
    pub(crate) U32Type => IntegerType(IntegerContent::U32) => ValueType,
    content: u32,
    kind: pub(crate) U32Kind,
    type_name: "u32",
    articled_display_name: "a u32",
}

define_leaf_type! {
    pub(crate) U64Type => IntegerType(IntegerContent::U64) => ValueType,
    content: u64,
    kind: pub(crate) U64Kind,
    type_name: "u64",
    articled_display_name: "a u64",
}

define_leaf_type! {
    pub(crate) ObjectType => ValueType(ValueContent::Object),
    content: ObjectValue,
    kind: pub(crate) ObjectKind,
    type_name: "object",
    articled_display_name: "an object",
}
