#![allow(unused)] // TODO: Remove when type resolution is implemented
use super::*;

pub(super) trait ResolvedTypeDetails {
    /// This should be true for types which users expect to have value
    /// semantics, but false for mutable types / types with reference
    /// semantics.
    /// 
    /// This indicates if an &x can be converted to an x via cloning
    /// when doing method resolution.
    fn supports_transparent_cloning(&self) -> bool;

    /// Resolves a method for this resolved type with the given arguments.
    fn resolve_method(&self, method: &MethodAccess, num_arguments: usize) -> ExecutionResult<ResolvedMethod>;

    // TODO: Eventually we can migrate operations under this umbrella too
    // fn resolve_unary_operation(&self, operation: UnaryOperation) -> ExecutionResult<ResolvedMethod>;
    // fn resolve_binary_operation(&self, operation: BinaryOperation) -> ExecutionResult<ResolvedMethod>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    None,
    Integer,
    Float,
    Boolean,
    String,
    Char,
    UnsupportedLiteral,
    Array,
    Object,
    Stream,
    Range,
    Iterator,
}

impl ResolvedTypeDetails for ValueKind {
    fn supports_transparent_cloning(&self) -> bool {
        match self {
            ValueKind::None => true,
            ValueKind::Integer => true,
            ValueKind::Float => true,
            ValueKind::Boolean => true,
            ValueKind::String => true,
            ValueKind::Char => true,
            ValueKind::UnsupportedLiteral => false,
            ValueKind::Array => false,
            ValueKind::Object => false,
            ValueKind::Stream => false,
            ValueKind::Range => true,
            ValueKind::Iterator => false,
        }
    }

    fn resolve_method(&self, method: &MethodAccess, num_arguments: usize) -> ExecutionResult<ResolvedMethod> {
        let method_name = method.method.to_string();
        match (self, method_name.as_str(), num_arguments) {
            // (ValueKind::Array, "len", 0) => Ok(ResolvedMethod::new(array_len)),
            // (ValueKind::Stream, "len", 0) => Ok(ResolvedMethod::new(stream_len)),
            _ => method.execution_err(format!("{self:?} has no method `{method_name}` with {num_arguments} arguments")),
        }
    }
}


pub(super) struct ResolvedMethod {
    method: WrappedMethod,
    argument_ownerships: Vec<RequestedValueOwnership>,
}

impl ResolvedMethod {
    // fn new<SelfType, Arguments, Output>(
    //     method: fn(SelfType, Arguments) -> ExecutionResult<Output>,
    // ) -> Self
    // where
    //     SelfType: ResolvableArgument + 'static,
    //     Arguments: ResolvableArguments + 'static,
    //     Output: ResolvableOutput + 'static,
    // {
    //     // TODO - Find some way of avoiding creating a new box for each method call (e.g. using a cache)
    //     // Could also consider using a GAT by upgrading to Rust 1.65
    //     Self {
    //         method: Box::new(move |
    //             self_value: ResolvedValue,
    //             arguments: Vec<ResolvedValue>,
    //             output_span_range: SpanRange,
    //         | -> ExecutionResult<ResolvedValue> {
    //             SelfType::run_resolved(self_value, |self_value| {
    //                 Arguments::run_with_arguments(arguments, |arguments| {
    //                     method(self_value, arguments)?.to_value(output_span_range)
    //                 }, output_span_range)
    //             })
    //         }),
    //         argument_ownerships: Arguments::ownerships(),
    //     }
    // }

    pub fn execute(
        &self,
        object: ResolvedValue,
        parameters: Vec<ResolvedValue>,
        span_range: SpanRange
    ) -> ExecutionResult<ResolvedValue> {
        (self.method)(object, parameters, span_range)
    }

    pub fn ownerships(&self) -> &[RequestedValueOwnership] {
        &self.argument_ownerships
    }
}

fn array_len(self_value: &ExpressionArray, arguments: ()) -> ExecutionResult<usize> {
    Ok(self_value.items.len())
}

fn stream_len(self_value: &ExpressionStream, arguments: ()) -> ExecutionResult<usize> {
    Ok(self_value.value.len())
}

type WrappedMethod = Box<(dyn Fn(ResolvedValue, Vec<ResolvedValue>, SpanRange) -> ExecutionResult<ResolvedValue>)>;
/*
trait ResolvableOutput {
    fn to_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue>;
}

impl<T: ToExpressionValue> ResolvableOutput for T {
    fn to_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Owned(self.to_value(output_span_range)))
    }
}

use arguments::*;

mod arguments {
    use super::*;

    // FRAMEWORK TO DO DISJOINT TRAIT IMPLEMENTATIONS

    pub(super) trait ResolvableArgument: Sized {
        fn run_resolved<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O>;
        fn resolve<O>(value: ResolvedValue) -> ExecutionResult<O>;
        fn ownership() -> RequestedValueOwnership;
    }

    pub(super) trait ResolvableArguments: Sized {
        fn run_with_arguments<O>(value: Vec<ResolvedValue>, inner: impl FnOnce(Self) -> ExecutionResult<O>, arguments_span: SpanRange) -> ExecutionResult<O>;
        fn ownerships() -> Vec<RequestedValueOwnership>;
    }

    impl ResolvableArguments for () {
        fn run_with_arguments<O>(value: Vec<ResolvedValue>, inner: impl FnOnce(Self) -> ExecutionResult<O>, arguments_span: SpanRange) -> ExecutionResult<O> {
            let Ok([
                // No arguments
            ]) = <[ResolvedValue; 0]>::try_from(value) else {
                return arguments_span.execution_err("Expected 0 arguments");
            };
            inner((
                // No arguments
            ))
        }
        
        fn ownerships() -> Vec<RequestedValueOwnership> {
            vec![]
        }
    }

    impl <T: ResolvableArgument> ResolvableArguments for (T,) {
        fn run_with_arguments<O>(value: Vec<ResolvedValue>, inner: impl FnOnce(Self) -> ExecutionResult<O>, arguments_span: SpanRange) -> ExecutionResult<O> {
            let Ok([
                value0,
            ]) = <[ResolvedValue; 1]>::try_from(value) else {
                return arguments_span.execution_err("Expected 1 argument");
            };

            T::run_resolved(value0, |value0| {
                // Further nesting...
                inner((value0,))
            })
        }
        
        fn ownerships() -> Vec<RequestedValueOwnership> {
            vec![T::ownership()]
        }
    }

    // TODO: Add more tuples, maybe using a macro to generate them

    trait ArgumentKind {}
    struct ArgumentKindOwned;
    impl ArgumentKind for ArgumentKindOwned {}
    struct ArgumentKindDisposedRef;
    impl ArgumentKind for ArgumentKindDisposedRef {}
    struct ArgumentKindCapturedRef;
    impl ArgumentKind for ArgumentKindCapturedRef {}
    struct ArgumentKindDisposedRefMut;
    impl ArgumentKind for ArgumentKindDisposedRefMut {}
    struct ArgumentKindCapturedRefMut;
    impl ArgumentKind for ArgumentKindCapturedRefMut {}

    trait InferredArgumentType: Sized {
        type ArgumentKind: ArgumentKind;
    }

    trait ResolvableArgumentAs<K: ArgumentKind>: Sized {
        fn run_with_argument<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O>;
        fn ownership() -> RequestedValueOwnership;
    }

    impl<T: InferredArgumentType + ResolvableArgumentAs<<T as InferredArgumentType>::ArgumentKind>> ResolvableArgument for T {        
        fn run_resolved<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O> {
            <T as ResolvableArgumentAs<<T as InferredArgumentType>::ArgumentKind>>::run_with_argument(value, inner)
        }
        
        fn ownership() -> RequestedValueOwnership {
            <T as ResolvableArgumentAs<<T as InferredArgumentType>::ArgumentKind>>::ownership()
        }
    }

    // TODO:
    // The automatic conversion of &XXX in a method into this might just not be feasible; because stacked borrows go from
    // out to in; and here we want to go from in (the arguments of an inner method) to out.
    // Instead, we might need to use a macro-based approach rather than just leveraging the type system.

    // trait ResolvableRef
    //     where for<'a> &'a Self: InferredArgumentType<ArgumentKind = ArgumentKindDisposedRef>
    // {
    //     fn resolve_from_value<'a>(value: &'a ExpressionValue) -> ExecutionResult<&'a Self>;
    // }

    // impl<'o, T: ResolvableRef> ResolvableArgumentAs<ArgumentKindDisposedRef> for &'o T
    //     where for<'a> &'a T: InferredArgumentType<ArgumentKind = ArgumentKindDisposedRef>
    // {
    //     fn run_with_argument<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O> {
    //         let output = {
    //             let value = value.as_value_ref();

    //             let output = inner(
    //                 // This needs to be 'o to match Self and work with the callback to match up with the defined function...
    //                 // But then this implies that 'o is smaller than this function call, which doesn't work.
    //                 <T as ResolvableRef>::resolve_from_value(value)?
    //             )?;
    //             output
    //         };
    //         Ok(output)
    //         // let mut value = value.as_value_ref();
    //         // inner(
    //         //     <T as ResolvableRef<'_>>::resolve_from_value(value)?
    //         // )
    //     }

    //     fn ownership() -> RequestedValueOwnership {
    //         RequestedValueOwnership::SharedReference
    //     }
    // }

    trait ResolvableRef<'a>: InferredArgumentType<ArgumentKind = ArgumentKindDisposedRef>
    {
        fn resolve_from_value(value: &'a ExpressionValue) -> ExecutionResult<Self>;
    }

    impl<'a, T: ResolvableRef<'a>> ResolvableArgumentAs<ArgumentKindDisposedRef> for T {
        fn run_with_argument<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O> {
            let mut value = value.as_value_ref();
            inner(
                <T as ResolvableRef<'_>>::resolve_from_value(value)?
            )
        }

        fn ownership() -> RequestedValueOwnership {
            RequestedValueOwnership::SharedReference
        }
    }

    trait ResolvableCapturedRef: InferredArgumentType<ArgumentKind = ArgumentKindCapturedRef> {
        fn resolve_from_value(value: SharedValue) -> ExecutionResult<Self>;
    }

    impl<'a, T: ResolvableCapturedRef> ResolvableArgumentAs<ArgumentKindCapturedRef> for T {
        fn run_with_argument<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O> {
            let mut value = value.into_shared_reference();
            inner(
                <T as ResolvableCapturedRef>::resolve_from_value(value)?
            )
        }
        
        fn ownership() -> RequestedValueOwnership {
            RequestedValueOwnership::SharedReference
        }
    }

    trait ResolvableMutRef<'a>: InferredArgumentType<ArgumentKind = ArgumentKindDisposedRefMut> {
        fn resolve_from_value(value: &'a mut ExpressionValue) -> ExecutionResult<Self>;
    }

    impl<T: for<'a> ResolvableMutRef<'a>> ResolvableArgumentAs<ArgumentKindDisposedRefMut> for T {
        fn run_with_argument<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O> {
            let mut value = value.into_mutable_reference("This argument")?;
            inner(
                <T as ResolvableMutRef<'_>>::resolve_from_value(value.as_mut())?
            )
        }
        
        fn ownership() -> RequestedValueOwnership {
            RequestedValueOwnership::MutableReference
        }
    }

    trait ResolvableCapturedMutRef: InferredArgumentType<ArgumentKind = ArgumentKindCapturedRefMut> {
        fn resolve_from_value(value: MutableValue) -> ExecutionResult<Self>;
    }

    impl<T: ResolvableCapturedMutRef> ResolvableArgumentAs<ArgumentKindCapturedRefMut> for T {
        fn run_with_argument<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O> {
            let mut value = value.into_mutable_reference("This argument")?;
            inner(
                <T as ResolvableCapturedMutRef>::resolve_from_value(value)?
            )
        }
        
        fn ownership() -> RequestedValueOwnership {
            RequestedValueOwnership::MutableReference
        }
    }

    trait ResolvableOwned: InferredArgumentType<ArgumentKind = ArgumentKindOwned> {
        fn resolve_from_value(value: ExpressionValue) -> ExecutionResult<Self>;
    }

    impl<T: ResolvableOwned> ResolvableArgumentAs<ArgumentKindOwned> for T {
        fn run_with_argument<O>(value: ResolvedValue, inner: impl FnOnce(Self) -> ExecutionResult<O>) -> ExecutionResult<O> {
            let mut value = value.into_owned_value("This argument")?;
            inner(
                <T as ResolvableOwned>::resolve_from_value(value)?
            )
        }
        
        fn ownership() -> RequestedValueOwnership {
            RequestedValueOwnership::Owned
        }
    }

    // IMPLEMENTATIONS

    impl InferredArgumentType for &'_ ExpressionValue {
        type ArgumentKind = ArgumentKindDisposedRef;
    }

    impl<'a> ResolvableRef<'a> for &'a ExpressionValue {
        fn resolve_from_value(value: &'a ExpressionValue) -> ExecutionResult<Self> {
            Ok(value)
        }
    }

    impl<T> InferredArgumentType for SharedSubPlace<T> {
        type ArgumentKind = ArgumentKindCapturedRef;
    }

    impl<T> ResolvableCapturedRef for SharedSubPlace<T>
        where 
            for<'a> &'a T: ResolvableRef<'a>,
    {
        fn resolve_from_value(value: SharedValue) -> ExecutionResult<Self> {
            value.resolve_internally_mapped(|value| <&'_ T as ResolvableRef<'_>>::resolve_from_value(value))
        }
    }

    impl InferredArgumentType for &'_ mut ExpressionValue {
        type ArgumentKind = ArgumentKindDisposedRefMut;
    }

    impl<'a> ResolvableMutRef<'a> for &'a mut ExpressionValue {
        fn resolve_from_value(value: &'a mut ExpressionValue) -> ExecutionResult<Self> {
            Ok(value)
        }
    }

    impl<T> InferredArgumentType for MutableSubPlace<T> {
        type ArgumentKind = ArgumentKindCapturedRefMut;
    }

    impl<T> ResolvableCapturedMutRef for MutableSubPlace<T>
        where 
            for<'a> &'a mut T: ResolvableMutRef<'a>,
    {
        fn resolve_from_value(value: MutableValue) -> ExecutionResult<Self> {
            value.resolve_internally_mapped(|value, _| <&'_ mut T as ResolvableMutRef<'_>>::resolve_from_value(value))
        }
    }

    impl InferredArgumentType for ExpressionValue {
        type ArgumentKind = ArgumentKindOwned;
    }

    impl ResolvableOwned for ExpressionValue {
        fn resolve_from_value(value: ExpressionValue) -> ExecutionResult<Self> {
            Ok(value)
        }
    }

    impl InferredArgumentType for &'_ ExpressionArray {
        type ArgumentKind = ArgumentKindDisposedRef;
    }

    impl<'a> ResolvableRef<'a> for &'a ExpressionArray {
        fn resolve_from_value(value: &'a ExpressionValue) -> ExecutionResult<Self> {
            match value {
                ExpressionValue::Array(arr) => Ok(arr),
                _ => value.execution_err("Expected array"),
            }
        }
    }

    impl InferredArgumentType for &'_ ExpressionStream {
        type ArgumentKind = ArgumentKindDisposedRef;
    }

    impl<'a> ResolvableRef<'a> for &'a ExpressionStream {
        fn resolve_from_value(value: &'a ExpressionValue) -> ExecutionResult<Self> {
            match value {
                ExpressionValue::Stream(stream) => Ok(stream),
                _ => value.execution_err("Expected stream"),
            }
        }
    }
}
 */