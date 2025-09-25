#![allow(unused)]
use std::mem;

// TODO[unused-clearup]
use super::*;

#[macro_use]
mod macros {
    macro_rules! handle_arg_mapping {
        // No more args
        ([$(,)?] [$($bindings:tt)*]) => {
            $($bindings)*
        };
        // By captured shared reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : Shared<$ty:ty>, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let tmp = handle_arg_name!($($arg_part)+).expect_shared();
                let handle_arg_name!($($arg_part)+): Shared<$ty> = tmp.try_map(|value, _| <$ty as ResolvableArgument>::resolve_from_ref(value))?;
            ])
        };
        // SharedValue is an alias for Shared<ExpressionValue>
        ([$($arg_part:ident)+ : SharedValue, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let handle_arg_name!($($arg_part)+) = handle_arg_name!($($arg_part)+).expect_shared();
            ])
        };
        // By captured mutable reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : Mutable<$ty:ty>, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let mut tmp = handle_arg_name!($($arg_part)+).expect_mutable();
                let handle_arg_name!($($arg_part)+): Mutable<$ty> = tmp.try_map(|value, _| <$ty as ResolvableArgument>::resolve_from_mut(value))?;
            ])
        };
        // MutableValue is an alias for Mutable<ExpressionValue>
        ([$($arg_part:ident)+ : MutableValue, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let handle_arg_name!($($arg_part)+) = handle_arg_name!($($arg_part)+).expect_mutable();
            ])
        };
        // By copy-on-write
        ([$($arg_part:ident)+ : CopyOnWriteValue, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let handle_arg_name!($($arg_part)+) = handle_arg_name!($($arg_part)+).expect_copy_on_write();
            ])
        };
        // By value
        ([$($arg_part:ident)+ : Owned<$ty:ty>, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let tmp = handle_arg_name!($($arg_part)+).expect_owned();
                let handle_arg_name!($($arg_part)+): Owned<$ty> = tmp.try_map(|value, _| <$ty as ResolvableArgument>::resolve_from_owned(value))?;
            ])
        };
        // By value
        ([$($arg_part:ident)+ : OwnedValue, $($rest:tt)*] [$($bindings:tt)*]) => {
            handle_arg_mapping!([$($rest)*] [
                $($bindings)*
                let handle_arg_name!($($arg_part)+) = handle_arg_name!($($arg_part)+).expect_owned();
            ])
        };
    }

    macro_rules! handle_arg_ownerships {
        // No more args
        ([$(,)?] [$($outputs:tt)*]) => {
            vec![$($outputs)*]
        };
        // By captured shared reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : Shared<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::Shared,])
        };
        // SharedValue is an alias for Shared<ExpressionValue>
        ([$($arg_part:ident)+ : SharedValue, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::Shared,])
        };
        // By captured mutable reference (i.e. can return a sub-reference from it)
        ([$($arg_part:ident)+ : Mutable<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::Mutable,])
        };
        // MutableValue is an alias for Mutable<ExpressionValue>
        ([$($arg_part:ident)+ : MutableValue, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::Mutable,])
        };
        // By copy-on-write
        ([$($arg_part:ident)+ : CopyOnWrite<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::CopyOnWrite,])
        };
        // By value
        ([$($arg_part:ident)+ : CopyOnWriteValue, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::CopyOnWrite,])
        };
        // By value
        ([$($arg_part:ident)+ : Owned<$ty:ty>, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::Owned,])
        };
        // By value
        ([$($arg_part:ident)+ : OwnedValue, $($rest:tt)*] [$($outputs:tt)*]) => {
            handle_arg_ownerships!([$($rest)*] [$($outputs)* ResolvedValueOwnership::Owned,])
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
    /// semantics, but false for types which are expensive to clone or
    /// are expected to have reference semantics.
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
            // Strings are value-like, so it makes sense to transparently clone them
            ValueKind::String => true,
            ValueKind::Char => true,
            ValueKind::UnsupportedLiteral => false,
            ValueKind::Array => false,
            ValueKind::Object => false,
            // It's super common to want to embed a stream in another stream
            // Having to embed it as #(type_name.clone()) instead of
            // #type_name would be awkward
            ValueKind::Stream => true,
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
                wrap_method! {(this: CopyOnWriteValue) -> ExecutionResult<OwnedValue> {
                    Ok(this.into_owned_infallible())
                }}
            }
            (_, "take", 0) => {
                wrap_method! {(mut this: MutableValue) -> ExecutionResult<ExpressionValue> {
                    let span_range = this.span_range();
                    Ok(mem::replace(this.deref_mut(), ExpressionValue::None(span_range)))
                }}
            }
            (_, "as_mut", 0) => {
                wrap_method! {(this: OwnedValue) -> ExecutionResult<MutableValue> {
                    Ok(MutableValue::new_from_owned(this))
                }}
            }
            (_, "as_ref", 0) => {
                wrap_method! {(this: SharedValue) -> ExecutionResult<SharedValue> {
                    Ok(this)
                }}
            }
            (_, "debug_string", 0) => {
                wrap_method! {(this: CopyOnWriteValue) -> ExecutionResult<String> {
                    this.into_owned_infallible().into_inner().into_debug_string()
                }}
            }
            (_, "debug", 0) => wrap_method! {(this: CopyOnWriteValue) -> ExecutionResult<String> {
                let value = this.into_owned_infallible();
                let span_range = value.span_range();
                let message = value.into_inner().into_debug_string()?;
                span_range.execution_err(message)
            }},
            // Mostly just a test of mutable values
            (_, "swap", 1) => {
                wrap_method! {(mut a: MutableValue, mut b: MutableValue) -> ExecutionResult<()> {
                    mem::swap(a.deref_mut(), b.deref_mut());
                    Ok(())
                }}
            }
            (ValueKind::Array, "len", 0) => {
                wrap_method! {(this: Shared<ExpressionArray>) -> ExecutionResult<usize> {
                    Ok(this.items.len())
                }}
            }
            (ValueKind::Array, "push", 1) => {
                wrap_method! {(mut this: Mutable<ExpressionArray>, item: OwnedValue) -> ExecutionResult<()> {
                    this.items.push(item.into());
                    Ok(())
                }}
            }
            (ValueKind::Stream, "len", 0) => {
                wrap_method! {(this: Shared<ExpressionStream>) -> ExecutionResult<usize> {
                    Ok(this.value.len())
                }}
            }
            (ValueKind::Stream, "flatten", 0) => {
                wrap_method! {(this: Owned<ExpressionStream>) -> ExecutionResult<TokenStream> {
                    Ok(this.into_inner().value.into_token_stream_removing_any_transparent_groups())
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
    argument_ownerships: Vec<ResolvedValueOwnership>,
}

impl MethodInterface {
    pub fn execute(
        &self,
        arguments: Vec<ResolvedValue>,
        span_range: SpanRange,
    ) -> ExecutionResult<ResolvedValue> {
        (self.method)(arguments, span_range)
    }

    pub(super) fn ownerships(&self) -> &[ResolvedValueOwnership] {
        &self.argument_ownerships
    }
}

use outputs::*;

mod outputs {
    use super::*;

    #[diagnostic::on_unimplemented(
        message = "`ResolvableOutput` is not implemented for `{Self}`",
        note = "`ResolvableOutput` is not implemented for `Shared<X>` or `Mutable<X>` unless `X` is `ExpressionValue`. If we wish to change this, we'd need to have some way to represent some kind of `ExpressionReference`, i.e. a `Typed<Shared<..>>` rather than a `Shared<Typed<..>>`"
    )]
    pub(crate) trait ResolvableOutput {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue>;
    }

    impl ResolvableOutput for Shared<ExpressionValue> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Shared(
                self.update_span_range(|_| output_span_range),
            ))
        }
    }

    impl ResolvableOutput for Mutable<ExpressionValue> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Mutable(
                self.update_span_range(|_| output_span_range),
            ))
        }
    }

    impl ResolvableOutput for Owned<ExpressionValue> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Owned(
                self.update_span_range(|_| output_span_range),
            ))
        }
    }

    impl<T: ToExpressionValue> ResolvableOutput for T {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Owned(
                self.to_value(output_span_range).into(),
            ))
        }
    }
}

pub(crate) use arguments::*;

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

    impl<'a> ResolveAs<&'a str> for &'a ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<&'a str> {
            match self {
                ExpressionValue::String(s) => Ok(&s.value),
                _ => self.execution_err("Expected string"),
            }
        }
    }

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
