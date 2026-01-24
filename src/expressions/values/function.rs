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

#[derive(Clone)]
pub(crate) enum FunctionDefinition {
    Native(&'static FunctionInterface),
    // TODO[closures]: Add closed_variables: Vec<ClosedVariable> to Closure
    Closure(ClosureExpression),
}

impl Eq for FunctionDefinition {}

impl PartialEq for FunctionDefinition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FunctionDefinition::Native(a), FunctionDefinition::Native(b)) => std::ptr::eq(*a, *b),
            (FunctionDefinition::Closure(a), FunctionDefinition::Closure(b)) => a == b,
            _ => false,
        }
    }
}

impl ValuesEqual for FunctionValue {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.definition == other.definition {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal("left_anon_function", "right_anon_function")
        }
    }
}

impl IsValueContent for &'static FunctionInterface {
    type Type = FunctionType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for &'static FunctionInterface {
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
