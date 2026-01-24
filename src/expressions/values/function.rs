use super::*;

define_leaf_type! {
    pub(crate) FunctionType => AnyType(AnyValueContent::Function),
    content: FunctionValue,
    kind: pub(crate) FunctionValueKind,
    type_name: "fn",
    articled_display_name: "a function",
    dyn_impls: {},
}

#[derive(Clone)]
pub(crate) struct FunctionValue {
    pub(crate) definition: FunctionDefinition,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum FunctionDefinition {
    Native(FunctionInterface),
    // TODO[closures]: Add closed_variables: Vec<ClosedVariable> to Closure
    Closure(ClosureExpression),
}

impl ValuesEqual for FunctionValue {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.definition == other.definition {
            return ctx.values_equal();
        } else {
            return ctx.leaf_values_not_equal("left_anon_function", "right_anon_function");
        }
    }
}

impl IsValueContent for FunctionInterface {
    type Type = FunctionType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for FunctionInterface {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        FunctionValue {
            definition: FunctionDefinition::Native(self),
        }
    }
}

impl IsValueContent for ClosureExpression {
    type Type = FunctionType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for ClosureExpression {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        FunctionValue {
            definition: FunctionDefinition::Closure(self),
        }
    }
}

define_type_features! {
    impl FunctionType,
    pub(crate) mod function_value_interface {}
}