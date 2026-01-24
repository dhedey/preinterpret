use super::*;

define_leaf_type! {
    pub(crate) NoneType => AnyType(AnyValueContent::None),
    content: (),
    kind: pub(crate) NoneKind,
    type_name: "none",
    articled_value_name: "a none",
    dyn_impls: {},
}

impl ResolvableArgumentTarget for () {
    type ValueType = NoneType;
}

impl ResolvableOwned<AnyValue> for () {
    fn resolve_from_value(value: AnyValue, context: ResolutionContext) -> ExecutionResult<Self> {
        match value {
            AnyValue::None(_) => Ok(()),
            other => context.err("None", other),
        }
    }
}

define_type_features! {
    impl NoneType,
    pub(crate) mod none_interface {}
}
