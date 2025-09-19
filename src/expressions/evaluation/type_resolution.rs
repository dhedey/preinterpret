#![allow(unused)] use std::mem;

// TODO[unused-clearup]
use super::*;

#[macro_use]
mod macros {
    macro_rules! handle_arg_mapping {
        // No more args
        ([$(,)?] [$($bindings:tt)*]) => {
            $($bindings)*
        };
        // By shared reference
        ([$($arg_part:ident)+ : &$ty:ty, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let handle_arg_name!($($arg_part)+): &$ty = <$ty as ResolvableArgument>::resolve_from_ref(handle_arg_name!($($arg_part)+).as_ref())?;
            ])
        };
        // By captured shared reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : CapturedRef<$ty:ty>, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let tmp = handle_arg_name!($($arg_part)+).into_shared_reference();
                let handle_arg_name!($($arg_part)+): CapturedRef<$ty> = tmp.try_map(|value, _| <$ty as ResolvableArgument>::resolve_from_ref(value))?;
            ])
        };
        // SharedValue is an alias for CapturedRef<ExpressionValue>
        ([$($arg_part:ident)+ : SharedValue, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let handle_arg_name!($($arg_part)+) = handle_arg_name!($($arg_part)+).into_shared_reference();
            ])
        };
        // By mutable reference
        ([$($arg_part:ident)+ : &mut $ty:ty, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let mut tmp = handle_arg_name!($($arg_part)+).into_mutable_reference()?;
                let handle_arg_name!($($arg_part)+): &mut $ty = <$ty as ResolvableArgument>::resolve_from_mut(tmp.as_mut())?;
            ])
        };
        // By captured mutable reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : CapturedMut<$ty:ty>, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let mut tmp = handle_arg_name!($($arg_part)+).into_mutable_reference()?;
                let handle_arg_name!($($arg_part)+): CapturedMut<$ty> = tmp.try_map(|value, _| <$ty as ResolvableArgument>::resolve_from_mut(value))?;
            ])
        };
        // MutableValue is an alias for CapturedMut<ExpressionValue>
        ([$($arg_part:ident)+ : MutableValue, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let handle_arg_name!($($arg_part)+) = handle_arg_name!($($arg_part)+).into_mutable_reference()?;
            ])
        };
        // By value
        ([$($arg_part:ident)+ : $ty:ty, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let tmp = handle_arg_name!($($arg_part)+).into_owned_value()?;
                let handle_arg_name!($($arg_part)+): $ty = <$ty as ResolvableArgument>::resolve_from_owned(tmp)?;
            ])
        };
    }

    macro_rules! handle_arg_ownerships {
        // No more args
        ([$(,)?] [$($outputs:tt)*]) => {
            vec![$($outputs)*]
        };
        // By shared reference
        ([$($arg_part:ident)+ : &$ty:ty, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::SharedReference,])
        };
        // By captured shared reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : CapturedRef<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::SharedReference,])
        };
        // SharedValue is an alias for CapturedRef<ExpressionValue>
        ([$($arg_part:ident)+ : SharedValue, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::SharedReference,])
        };
        // By mutable reference
        ([$($arg_part:ident)+ : &mut $ty:ty, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::MutableReference,])
        };
        // By captured mutable reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : CapturedMut<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::MutableReference,])
        };
        // MutableValue is an alias for CapturedMut<ExpressionValue>
        ([$($arg_part:ident)+ : MutableValue, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::MutableReference,])
        };
        // By value
        ([$($arg_part:ident)+ : $ty:ty, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* RequestedValueOwnership::Owned,])
        };
    }

    macro_rules! count {
        () => { 0 };
        ($head:tt $($tail:tt)*) => { 1 + count!($($tail)*) };
    }

    // Creating an inner method vastly improves IDE support when writing the method body
    macro_rules! handle_define_inner_method {
        // The $arg_part+ allows for mut x in the argument list
        ($method_name:ident [$($($arg_part:ident)+ : $ty:ty),* $(,)?] $body:block $output_ty:ty) => {
            fn $method_name($($($arg_part)+: $ty),*) -> ExecutionResult<$output_ty> {
                $body
            }
        };
    }

    macro_rules! handle_arg_name {
        (mut $name:ident) => {
            $name
        };
        ($name:ident) => {
            $name
        };
    }

    macro_rules! handle_arg_separation {
        ([$($($arg_part:ident)+ : $ty:ty),* $(,)?], $all_arguments:ident, $output_span_range:ident) => {
            const LEN: usize = count!($($ty)*);
            let Ok([
                $(handle_arg_name!($($arg_part)+),)*
            ]) = <[ResolvedValue; LEN]>::try_from($all_arguments) else {
                return $output_span_range.execution_err(format!("Expected {LEN} argument/s"));
            };
        };
    }

    macro_rules! handle_call_inner_method {
        ($method_name:ident [$($($arg_part:ident)+ : $ty:ty),* $(,)?]) => {
            $method_name($(handle_arg_name!($($arg_part)+)),*)
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
}

pub(crate) trait ResolvedTypeDetails {
    /// This should be true for types which users expect to have value
    /// semantics, but false for mutable types / types with reference
    /// semantics.
    ///
    /// This indicates if an &x can be converted to an x via cloning
    /// when doing method resolution.
    fn supports_transparent_cloning(&self) -> bool;

    /// Resolves a method for this resolved type with the given arguments.
    fn resolve_method(
        &self,
        method: &MethodAccess,
        num_arguments: usize,
    ) -> ExecutionResult<MethodInterface>;

    // TODO[operation-refactor]: Eventually we can migrate operations under this umbrella too
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

    fn resolve_method(
        &self,
        method: &MethodAccess,
        num_arguments: usize,
    ) -> ExecutionResult<MethodInterface> {
        let method_name = method.method.to_string();
        let method = match (self, method_name.as_str(), num_arguments) {
            (_, "clone", 0) => {
                wrap_method! {(this: &ExpressionValue) -> ExecutionResult<ExpressionValue> {
                    Ok(this.clone())
                }}
            }
            (_, "take", 0) => {
                wrap_method! {(mut this: MutableValue) -> ExecutionResult<ExpressionValue> {
                    let span_range = this.span_range();
                    Ok(mem::replace(this.deref_mut(), ExpressionValue::None(span_range)))
                }}
            }
            (_, "as_mut", 0) => {
                wrap_method! {(this: ExpressionValue) -> ExecutionResult<MutableValue> {
                    Ok(MutableValue::new_from_owned(this))
                }}
            }
            (_, "debug", 0) => wrap_method! {(this: &ExpressionValue) -> ExecutionResult<String> {
                this.clone().into_debug_string()
            }},
            (ValueKind::Array, "len", 0) => {
                wrap_method! {(this: &ExpressionArray) -> ExecutionResult<usize> {
                    Ok(this.items.len())
                }}
            }
            (ValueKind::Array, "push", 1) => {
                wrap_method! {(this: &mut ExpressionArray, item: ExpressionValue) -> ExecutionResult<()> {
                    this.items.push(item);
                    Ok(())
                }}
            }
            (ValueKind::Stream, "len", 0) => {
                wrap_method! {(this: &ExpressionStream) -> ExecutionResult<usize> {
                    Ok(this.value.len())
                }}
            }
            _ => {
                return method.execution_err(format!(
                    "{self:?} has no method `{method_name}` with {num_arguments} arguments"
                ))
            }
        };
        Ok(method)
    }
}

pub(crate) struct MethodInterface {
    method: Box<(dyn Fn(Vec<ResolvedValue>, SpanRange) -> ExecutionResult<ResolvedValue>)>,
    argument_ownerships: Vec<RequestedValueOwnership>,
}

impl MethodInterface {
    pub fn execute(
        &self,
        arguments: Vec<ResolvedValue>,
        span_range: SpanRange,
    ) -> ExecutionResult<ResolvedValue> {
        (self.method)(arguments, span_range)
    }

    pub(super) fn ownerships(&self) -> &[RequestedValueOwnership] {
        &self.argument_ownerships
    }
}

use outputs::*;

mod outputs {
    use super::*;

    #[diagnostic::on_unimplemented(
        message = "`ResolvableOutput` is not implemented for `{Self}`",
        note = "`ResolvableOutput` is not implemented for `CapturedRef<X>` or `CapturedMut<X>` unless `X` is `ExpressionValue`. If we wish to change this, we'd need to have some way to represent some kind of `ExpressionReference`, i.e. a `Typed<CapturedRef<..>>` rather than a `CapturedRef<Typed<..>>`"
    )]
    pub(crate) trait ResolvableOutput {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue>;
    }

    impl ResolvableOutput for CapturedRef<ExpressionValue> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Shared {
                shared_ref: self.update_span_range(|_| output_span_range),
                reason_not_mutable: Some(syn::Error::new(
                    output_span_range.join_into_span_else_start(),
                    "It was output from a method a captured shared reference",
                )),
            })
        }
    }

    impl ResolvableOutput for CapturedMut<ExpressionValue> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Mutable(
                self.update_span_range(|_| output_span_range),
            ))
        }
    }

    impl<T: ToExpressionValue> ResolvableOutput for T {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Owned(self.to_value(output_span_range)))
        }
    }
}

use arguments::*;

mod arguments {
    use super::*;

    pub(crate) trait ResolveAs<T> {
        fn resolve_as(self) -> ExecutionResult<T>;
    }

    impl<T: ResolvableArgument> ResolveAs<T> for ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<T> {
            T::resolve_from_owned(self)
        }
    }

    impl<'a, T: ResolvableArgument> ResolveAs<&'a T> for &'a ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<&'a T> {
            T::resolve_from_ref(self)
        }
    }

    impl<'a, T: ResolvableArgument> ResolveAs<&'a mut T> for &'a mut ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<&'a mut T> {
            T::resolve_from_mut(self)
        }
    }

    pub(crate) trait ResolvableArgument: Sized {
        fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self>;
        fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self>;
        fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self>;
    }

    impl ResolvableArgument for ExpressionValue {
        fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
            Ok(value)
        }

        fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self> {
            Ok(value)
        }

        fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
            Ok(value)
        }
    }

    macro_rules! impl_resolvable_argument_for {
        (($value:ident) -> $type:ty $body:block) => {
            impl ResolvableArgument for $type {
                fn resolve_from_owned($value: ExpressionValue) -> ExecutionResult<Self> {
                    $body
                }

                fn resolve_from_ref($value: &ExpressionValue) -> ExecutionResult<&Self> {
                    $body
                }

                fn resolve_from_mut($value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
                    $body
                }
            }
        };
    }

    pub(crate) use impl_resolvable_argument_for;

    impl_resolvable_argument_for! {(value) -> ExpressionInteger {
        match value {
            ExpressionValue::Integer(value) => Ok(value),
            _ => value.execution_err("Expected integer"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> u8 {
        match value {
            ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U8(x), ..}) => Ok(x),
            _ => value.execution_err("Expected u8"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> usize {
        match value {
            ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::Usize(x), ..}) => Ok(x),
            _ => value.execution_err("Expected usize"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionFloat {
        match value {
            ExpressionValue::Float(value) => Ok(value),
            _ => value.execution_err("Expected float"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionBoolean {
        match value {
            ExpressionValue::Boolean(value) => Ok(value),
            _ => value.execution_err("Expected boolean"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionString {
        match value {
            ExpressionValue::String(value) => Ok(value),
            _ => value.execution_err("Expected string"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionChar {
        match value {
            ExpressionValue::Char(value) => Ok(value),
            _ => value.execution_err("Expected char"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionArray {
        match value {
            ExpressionValue::Array(value) => Ok(value),
            _ => value.execution_err("Expected array"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionObject {
        match value {
            ExpressionValue::Object(value) => Ok(value),
            _ => value.execution_err("Expected object"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionStream {
        match value {
            ExpressionValue::Stream(value) => Ok(value),
            _ => value.execution_err("Expected stream"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionRange {
        match value {
            ExpressionValue::Range(value) => Ok(value),
            _ => value.execution_err("Expected range"),
        }
    }}

    impl_resolvable_argument_for! {(value) -> ExpressionIterator {
        match value {
            ExpressionValue::Iterator(value) => Ok(value),
            _ => value.execution_err("Expected iterator"),
        }
    }}
}
