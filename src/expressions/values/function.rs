use super::*;

define_leaf_type! {
    pub(crate) FunctionType => AnyType(AnyValueContent::Function),
    content: FunctionValue,
    kind: pub(crate) FunctionValueKind,
    type_name: "fn",
    articled_value_name: "a function",
    dyn_impls: {},
}

define_type_features! {
    impl FunctionType,
    pub(crate) mod function_value_interface {}
}

#[derive(Clone)]
pub(crate) struct FunctionValue {
    pub(crate) disabled_bound_arguments: Vec<Spanned<DisabledArgumentValue>>,
    pub(crate) invokable: InvokableFunction,
}

#[derive(Clone)]
pub(crate) enum InvokableFunction {
    Native(&'static FunctionInterface),
    // TODO[closures]: Add closed_variables: Vec<ClosedVariable> to Closure
    Closure(ClosureValue),
}

impl Eq for InvokableFunction {}

impl PartialEq for InvokableFunction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InvokableFunction::Native(a), InvokableFunction::Native(b)) => std::ptr::eq(*a, *b),
            (InvokableFunction::Closure(a), InvokableFunction::Closure(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl InvokableFunction {
    /// Returns (argument_ownerships, required_argument_count)
    pub(crate) fn argument_ownerships(&self) -> (&[ArgumentOwnership], usize) {
        match self {
            InvokableFunction::Native(interface) => interface.argument_ownerships(),
            InvokableFunction::Closure(closure) => closure.argument_ownerships(),
        }
    }

    pub(crate) fn invoke(
        self,
        arguments: Vec<Spanned<ArgumentValue>>,
        context: &mut FunctionCallContext,
    ) -> FunctionResult<Spanned<ReturnedValue>> {
        match self {
            InvokableFunction::Native(interface) => interface.invoke(arguments, context),
            InvokableFunction::Closure(closure) => closure.invoke(arguments, context),
        }
    }
}

impl ValuesEqual for FunctionValue {
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.invokable == other.invokable {
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
            invokable: InvokableFunction::Native(self),
            disabled_bound_arguments: vec![],
        }
    }
}

impl IsValueContent for ClosureValue {
    type Type = FunctionType;
    type Form = BeOwned;
}

impl IntoValueContent<'static> for ClosureValue {
    fn into_content(self) -> Content<'static, Self::Type, Self::Form> {
        FunctionValue {
            invokable: InvokableFunction::Closure(self),
            disabled_bound_arguments: vec![],
        }
    }
}
