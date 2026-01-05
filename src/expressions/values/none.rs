use super::*;

define_leaf_type! {
    pub(crate) NoneType => ValueType(ValueContent::None),
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

impl ResolvableOwned<Value> for () {
    fn resolve_from_value(value: Value, context: ResolutionContext) -> ExecutionResult<Self> {
        match value {
            Value::None(_) => Ok(()),
            other => context.err("None", other),
        }
    }
}

define_optional_object! {
    pub(crate) struct SettingsInputs {
        iteration_limit: usize => (DEFAULT_ITERATION_LIMIT_STR, "The new iteration limit"),
    }
}

define_type_features! {
    impl NoneType,
    pub(crate) mod none_interface {
        pub(crate) mod methods {
            [context] fn configure_preinterpret(_none: (), inputs: SettingsInputs) {
                if let Some(limit) = inputs.iteration_limit {
                    context.interpreter.set_iteration_limit(Some(limit));
                }
            }
        }
        pub(crate) mod unary_operations {}
        pub(crate) mod binary_operations {}
        interface_items {}
    }
}
