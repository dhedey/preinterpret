use super::*;

impl ToExpressionValue for () {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::None
    }
}

impl ResolvableArgumentTarget for () {
    type ValueType = NoneTypeData;
}

impl ResolvableArgumentOwned for () {
    fn resolve_from_value(
        value: ExpressionValue,
        context: ResolutionContext,
    ) -> ExecutionResult<Self> {
        match value {
            ExpressionValue::None => Ok(()),
            other => context.err("None", other),
        }
    }
}

define_optional_object! {
    pub(crate) struct SettingsInputs {
        iteration_limit: usize => (DEFAULT_ITERATION_LIMIT_STR, "The new iteration limit"),
    }
}

define_interface! {
    struct NoneTypeData,
    parent: ValueTypeData,
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
