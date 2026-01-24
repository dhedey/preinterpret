use super::*;

define_leaf_type! {
    pub(crate) NoneType => AnyType(AnyValueContent::None),
    content: (),
    kind: pub(crate) NoneKind,
    type_name: "none",
    // Instead of saying "expected a none value", we can say "expected None"
    articled_display_name: "None",
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
