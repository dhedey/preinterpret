use super::*;

define_leaf_type! {
    pub(crate) PreinterpretApiType => AnyType(AnyValueContent::PreinterpretApi),
    content: PreinterpretApiValue,
    kind: pub(crate) PreinterpretApiKind,
    type_name: "preinterpret",
    articled_value_name: "a preinterpret api",
    dyn_impls: {},
}

#[derive(Clone)]
pub(crate) enum PreinterpretApiValue {}

define_type_features! {
    impl PreinterpretApiType,
    pub(crate) mod preinterpret_api_interface {
        functions {
            [context] fn set_iteration_limit(new_limit: OptionalSuffix<usize>) {
                context.interpreter.set_iteration_limit(new_limit.0);
            }

            [context] fn set_stack_depth_limit(new_limit: OptionalSuffix<usize>) {
                context.interpreter.set_stack_depth_limit(new_limit.0);
            }
        }
    }
}
