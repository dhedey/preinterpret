use super::*;

macro_rules! handle_arg_mapping {
    // No more args
    ([$(,)?] [$($bindings:tt)*]) => {
        $($bindings)*
    };
    // By shared reference
    ([$arg:ident : &$ty:ty, $($rest:tt)*] [$($bindings:tt)*]) => {
        handle_arg_mapping!([$($rest)*] [
            $($bindings)*
            let $arg: &$ty = <$ty as ResolvableArgument>::resolve_ref($arg.as_ref())?;
        ])
    };
    // By captured shared reference (i.e. can return a sub-reference from it)
    ([$arg:ident : CapturedRef<$ty:ty>, $($rest:tt)*] [$($bindings:tt)*]) => {
        handle_arg_mapping!([$($rest)*] [
            $($bindings)*
            let tmp = $arg.into_shared_reference();
            let $arg: CapturedRef<$ty> = tmp.try_map(|value, _| <$ty as ResolvableArgument>::resolve_ref(value))?;
        ])
    };
    // SharedValue is an alias for CapturedRef<ExpressionValue>
    ([$arg:ident : SharedValue, $($rest:tt)*] [$($bindings:tt)*]) => {
        handle_arg_mapping!([$($rest)*] [
            $($bindings)*
            let $arg = $arg.into_shared_reference();
        ])
    };
    // By mutable reference
    ([$arg:ident : &mut $ty:ty, $($rest:tt)*] [$($bindings:tt)*]) => {
        handle_arg_mapping!([$($rest)*] [
            $($bindings)*
            let mut tmp = $arg.into_mutable_reference("This argument")?;
            let $arg: &mut $ty = <$ty as ResolvableArgument>::resolve_mut(tmp.as_mut());
        ])
    };
    // By captured mutable reference (i.e. can return a sub-reference from it)
    ([$arg:ident : CapturedMut<$ty:ty>, $($rest:tt)*] [$($bindings:tt)*]) => {
        handle_arg_mapping!([$($rest)*] [
            $($bindings)*
            let mut tmp = $arg.into_mutable_reference("This argument")?;
            let $arg: CapturedMut<$ty> = tmp.try_map(|value, _| <$ty as ResolvableArgument>::resolve_mut(value))?;
        ])
    };
    // MutableValue is an alias for CapturedMut<ExpressionValue>
    ([$arg:ident : MutableValue, $($rest:tt)*] [$($bindings:tt)*]) => {
        handle_arg_mapping!([$($rest)*] [
            $($bindings)*
            let $arg = $arg.into_mutable_reference("This argument")?;
        ])
    };
    // By value
    ([$arg:ident : $ty:ty, $($rest:tt)*] [$($bindings:tt)*]) => {
        handle_arg_mapping!([$($rest)*] [
            $($bindings)*
            let tmp = $arg.into_owned_value("This argument")?;
            let $arg: $ty = <$ty as ResolvableArgument>::resolve_owned(tmp)?;
        ])
    };
}

macro_rules! handle_arg_ownerships {
    // No more args
    ([$(,)?] [$($outputs:tt)*]) => {
        vec![$($outputs)*]
    };
    // By shared reference
    ([$arg:ident : &$ty:ty, $($rest:tt)*] [$($outputs:tt)*]) => {
        handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::SharedReference,])
    };
    // By captured shared reference (i.e. can return a sub-reference from it)
    ([$arg:ident : CapturedRef<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
        handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::SharedReference,])
    };
    // SharedValue is an alias for CapturedRef<ExpressionValue>
    ([$arg:ident : SharedValue, $($rest:tt)*] [$($outputs:tt)*]) => {
        handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::SharedReference,])
    };
    // By mutable reference
    ([$arg:ident : &mut $ty:ty, $($rest:tt)*] [$($outputs:tt)*]) => {
        handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::MutableReference,])
    };
    // By captured mutable reference (i.e. can return a sub-reference from it)
    ([$arg:ident : CapturedMut<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
        handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::MutableReference,])
    };
    // MutableValue is an alias for CapturedMut<ExpressionValue>
    ([$arg:ident : MutableValue, $($rest:tt)*] [$($outputs:tt)*]) => {
        handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::MutableReference,])
    };
    // By value
    ([$arg:ident : $ty:ty, $($rest:tt)*] [$($outputs:tt)*]) => {
        handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::Owned,])
    };
}

macro_rules! count {
    () => { 0 };
    ($head:tt $($tail:tt)*) => { 1 + count!($($tail)*) };
}

trait ResolvableArgument: Sized {
    fn resolve_owned(value: ExpressionValue) -> ExecutionResult<Self>;
    fn resolve_ref<'a>(value: &'a ExpressionValue) -> ExecutionResult<&'a Self>;
    fn resolve_mut<'a>(value: &'a mut ExpressionValue) -> ExecutionResult<&'a mut Self>;
}

// Creating an inner method vastly improves IDE support when writing the method body
macro_rules! handle_define_inner_method {
    ($method_name:ident [$($arg:ident : $ty:ty),* $(,)?] $body:block $output_ty:ty) => {
        fn $method_name($($arg: $ty),*) -> ExecutionResult<$output_ty> {
            $body
        }
    };
}

macro_rules! handle_arg_separation {
    ([$($arg:ident : $ty:ty),* $(,)?], $all_arguments:ident, $output_span_range:ident) => {
        const LEN: usize = count!($($arg)*);
        let Ok([
            $($arg,)*
        ]) = <[ResolvedValue; LEN]>::try_from($all_arguments) else {
            return $output_span_range.execution_err(format!("Expected {LEN} argument/s"));
        };
    };
}

macro_rules! handle_call_inner_method {
    ($method_name:ident [$($arg:ident : $ty:ty),* $(,)?]) => {
        $method_name($($arg),*)
    };
}

macro_rules! wrap_method {
    (($($args:tt)*) -> ExecutionResult<$output_ty:ty> $body:block) => {
        MethodInterface {
            method: Box::new(move |
                all_arguments: Vec<ResolvedValue>,
                output_span_range: SpanRange,
            | -> ExecutionResult<ResolvedValue> {
                handle_define_inner_method!(inner_method [$($args)*] $body $output_ty);
                handle_arg_separation!([$($args)*], all_arguments, output_span_range);
                handle_arg_mapping!([$($args)*,] []);
                let output: ExecutionResult<$output_ty> = handle_call_inner_method!(inner_method [$($args)*]);
                <$output_ty as ResolvableOutput>::to_resolved_value(output?, output_span_range)
            }),
            argument_ownerships: handle_arg_ownerships!([$($args)*,] []),
        }
    };
}

pub(super) struct MethodInterface {
    method: Box<(dyn Fn(Vec<ResolvedValue>, SpanRange) -> ExecutionResult<ResolvedValue>)>,
    argument_ownerships: Vec<RequestedValueOwnership>,
}

impl MethodInterface {
    pub fn execute(
        &self,
        parameters: Vec<ResolvedValue>,
        span_range: SpanRange
    ) -> ExecutionResult<ResolvedValue> {
        (self.method)(parameters, span_range)
    }
}

impl ResolvableArgument for ExpressionValue {
    fn resolve_owned(value: ExpressionValue) -> ExecutionResult<Self> {
        Ok(value)
    }

    fn resolve_ref<'a>(value: &'a ExpressionValue) -> ExecutionResult<&'a Self> {
        Ok(value)
    }

    fn resolve_mut<'a>(value: &'a mut ExpressionValue) -> ExecutionResult<&'a mut Self> {
        Ok(value)
    }
}

impl ResolvableArgument for ExpressionArray {
    fn resolve_owned(value: ExpressionValue) -> ExecutionResult<Self> {
        match value {
            ExpressionValue::Array(value) => Ok(value),
            _ => value.execution_err("Expected array"),
        }
    }
    
    fn resolve_ref<'a>(value: &'a ExpressionValue) -> ExecutionResult<&'a Self> {
        match value {
            ExpressionValue::Array(value) => Ok(value),
            _ => value.execution_err("Expected array"),
        }
    }
    
    fn resolve_mut<'a>(value: &'a mut ExpressionValue) -> ExecutionResult<&'a mut Self> {
        match value {
            ExpressionValue::Array(value) => Ok(value),
            _ => value.execution_err("Expected array"),
        }
    }
}

impl ResolvableArgument for u8 {
    fn resolve_owned(value: ExpressionValue) -> ExecutionResult<Self> {
        match value {
            ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U8(x), ..}) => Ok(x),
            _ => value.execution_err("Expected u8"),
        }
    }

    fn resolve_ref<'a>(value: &'a ExpressionValue) -> ExecutionResult<&'a Self> {
        match value {
            ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U8(x), ..}) => Ok(x),
            _ => value.execution_err("Expected u8"),
        }
    }
    
    fn resolve_mut<'a>(value: &'a mut ExpressionValue) -> ExecutionResult<&'a mut Self> {
        match value {
            ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U8(x), ..}) => Ok(x),
            _ => value.execution_err("Expected u8"),
        }
    }
}

#[diagnostic::on_unimplemented(
    message = "`ResolvableOutput` is not implemented for `{Self}`",
    note = "`ResolvableOutput` is not implemented for `CapturedRef<X>` or `CapturedMut<X>` unless `X` is `ExpressionValue`. If we wish to change this, we'd need to have some way to represent some kind of `ExpressionReference`, i.e. a `Typed<CapturedRef<..>>` rather than a `CapturedRef<Typed<..>>`",
)]
trait ResolvableOutput {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue>;
}

impl ResolvableOutput for CapturedRef<ExpressionValue> {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Shared {
            shared_ref: self.update_span_range(|_| output_span_range),
            reason_not_mutable: Some(syn::Error::new(output_span_range.join_into_span_else_start(), "It was output from a method a captured shared reference")),
        })
    }
}

impl ResolvableOutput for CapturedMut<ExpressionValue> {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Mutable(self.update_span_range(|_| output_span_range)))
    }
}

impl<T: ToExpressionValue> ResolvableOutput for T {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Owned(self.to_value(output_span_range)))
    }
}

#[test]
fn test2() {
    // Example usage:
    let span_range = SpanRange::new_single(Span::call_site());
    let x = ResolvableOutput::to_resolved_value(vec![10u8.to_value(span_range)], span_range).unwrap();
    let y = ResolvableOutput::to_resolved_value(0usize, span_range).unwrap();

    let method1 = wrap_method!((a: CapturedRef<ExpressionArray>, index: &ExpressionValue) -> ExecutionResult<SharedValue> {
        let access = IndexAccess { brackets: Brackets { delim_span: Group::new(Delimiter::Brace, TokenStream::new()).delim_span() }};
        a.try_map(|a, _| a.index_ref(access, index))
    });
    let output = method1.execute(vec![x, y], span_range).unwrap();

    assert_eq!(*u8::resolve_ref(output.as_value_ref()).unwrap(), 10u8);
}