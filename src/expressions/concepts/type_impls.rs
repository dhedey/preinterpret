use super::*;

// NOTE: These should be moved out to the value definitions soon

pub(crate) struct ValueType;

// type Value = Actual<'static, ValueType, BeOwned>;
// type ValueReferencable = Actual<'static, ValueType, BeReferencable>;
// type ValueRef<'a> = Actual<'a, ValueType, BeAnyRef>;
// type ValueMut<'a> = Actual<'a, ValueType, BeAnyRefMut>;

impl<F: IsForm> MapToType<ValueType, F> for ValueType {
    fn map_to_type<'a>(content: Self::Content<'a, F>) -> Self::Content<'a, F> {
        content
    }
}

impl<F: IsForm> MaybeMapFromType<ValueType, F> for ValueType {
    fn maybe_map_from_type<'a>(content: Self::Content<'a, F>) -> Option<Self::Content<'a, F>> {
        Some(content)
    }
}

impl IsType for ValueType {
    type Content<'a, F: IsForm> = ValueContent<'a, F>;

    fn articled_type_name() -> &'static str {
        "any value"
    }
}

impl IsHierarchyType for ValueType {
    fn map_with<'a, F: IsForm, M: GeneralMapper<F>>(
        content: Self::Content<'a, F>,
    ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>> {
        match content {
            ValueContent::Integer(x) => Ok(ValueContent::Integer(x.map_with::<M>()?)),
            ValueContent::Object(x) => Ok(ValueContent::Object(x.map_with::<M>()?)),
        }
    }
}

pub(crate) enum ValueContent<'a, F: IsForm> {
    Integer(Actual<'a, IntegerType, F>),
    Object(Actual<'a, ObjectType, F>),
    // ...
}

pub(crate) enum IntegerContent<'a, F: IsForm> {
    U32(Actual<'a, U32Type, F>),
    U64(Actual<'a, U64Type, F>),
    // ...
}

pub(crate) struct IntegerType;

impl IsType for IntegerType {
    type Content<'a, F: IsForm> = IntegerContent<'a, F>;

    fn articled_type_name() -> &'static str {
        "an integer"
    }
}

impl IsHierarchyType for IntegerType {
    fn map_with<'a, F: IsForm, M: GeneralMapper<F>>(
        content: Self::Content<'a, F>,
    ) -> Result<Self::Content<'a, M::OutputForm>, M::ShortCircuit<'a>> {
        match content {
            IntegerContent::U32(x) => Ok(IntegerContent::U32(x.map_with::<M>()?)),
            IntegerContent::U64(x) => Ok(IntegerContent::U64(x.map_with::<M>()?)),
        }
    }
}

impl_ancestor_chain_conversions!(
    IntegerType => ValueType => []
);

impl IsChildType for IntegerType {
    type ParentType = ValueType;

    fn into_parent<'a, F: IsForm>(
        content: Self::Content<'a, F>,
    ) -> <Self::ParentType as IsType>::Content<'a, F> {
        ValueContent::Integer(Actual(content))
    }

    fn from_parent<'a, F: IsForm>(
        content: <Self::ParentType as IsType>::Content<'a, F>,
    ) -> Option<Self::Content<'a, F>> {
        match content {
            ValueContent::Integer(i) => Some(i.0),
            _ => None,
        }
    }
}

define_leaf_type!(u32: pub(crate) U32Type "a u32");

impl_ancestor_chain_conversions!(
    U32Type => IntegerType => [ValueType]
);

impl IsChildType for U32Type {
    type ParentType = IntegerType;

    fn into_parent<'a, F: IsForm>(
        content: Self::Content<'a, F>,
    ) -> <Self::ParentType as IsType>::Content<'a, F> {
        IntegerContent::U32(Actual(content))
    }

    fn from_parent<'a, F: IsForm>(
        content: <Self::ParentType as IsType>::Content<'a, F>,
    ) -> Option<Self::Content<'a, F>> {
        match content {
            IntegerContent::U32(i) => Some(i.0),
            _ => None,
        }
    }
}

define_leaf_type!(u64: pub(crate) U64Type "a u64");

impl_ancestor_chain_conversions!(
    U64Type => IntegerType => [ValueType]
);

impl IsChildType for U64Type {
    type ParentType = IntegerType;

    fn into_parent<'a, F: IsForm>(
        content: Self::Content<'a, F>,
    ) -> <Self::ParentType as IsType>::Content<'a, F> {
        IntegerContent::U64(Actual(content))
    }

    fn from_parent<'a, F: IsForm>(
        content: <Self::ParentType as IsType>::Content<'a, F>,
    ) -> Option<Self::Content<'a, F>> {
        match content {
            IntegerContent::U64(i) => Some(i.0),
            _ => None,
        }
    }
}

define_leaf_type!(ObjectValue: pub(crate) ObjectType "an object");

impl_ancestor_chain_conversions!(
    ObjectType => ValueType => []
);

impl IsChildType for ObjectType {
    type ParentType = ValueType;

    fn into_parent<'a, F: IsForm>(
        content: Self::Content<'a, F>,
    ) -> <Self::ParentType as IsType>::Content<'a, F> {
        ValueContent::Object(Actual(content))
    }

    fn from_parent<'a, F: IsForm>(
        content: <Self::ParentType as IsType>::Content<'a, F>,
    ) -> Option<Self::Content<'a, F>> {
        match content {
            ValueContent::Object(o) => Some(o.0),
            _ => None,
        }
    }
}
