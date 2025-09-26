#![allow(unused)]
use std::mem;

// TODO[unused-clearup]
use super::*;

pub(crate) trait MethodResolver {
    /// Resolves a unary operation as a method interface for this type.
    fn resolve_method(&self, method_name: &str) -> Option<MethodInterface>;

    /// Resolves a unary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_unary_operation(&self, operation: &UnaryOperation) -> Option<MethodInterface>;

    /// Resolves a binary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_binary_operation(&self, operation: &BinaryOperation) -> Option<MethodInterface>;
}

impl<T: MethodResolutionTarget> MethodResolver for T {
    fn resolve_method(&self, method_name: &str) -> Option<MethodInterface> {
        match Self::resolve_own_method(method_name) {
            Some(method) => Some(method),
            None => Self::PARENT.and_then(|p| p.resolve_method(method_name)),
        }
    }

    fn resolve_unary_operation(&self, operation: &UnaryOperation) -> Option<MethodInterface> {
        match Self::resolve_own_unary_operation(operation) {
            Some(method) => Some(method),
            None => Self::PARENT.and_then(|p| p.resolve_unary_operation(operation)),
        }
    }

    fn resolve_binary_operation(&self, operation: &BinaryOperation) -> Option<MethodInterface> {
        match Self::resolve_own_binary_operation(operation) {
            Some(method) => Some(method),
            None => Self::PARENT.and_then(|p| p.resolve_binary_operation(operation)),
        }
    }
}

pub(crate) trait MethodResolutionTarget {
    type Parent: MethodResolutionTarget;
    const PARENT: Option<Self::Parent>;

    fn assert_first_argument<T: FromResolved<ValueType = Self>>() {}

    fn resolve_own_method(method_name: &str) -> Option<MethodInterface> {
        None
    }

    /// Resolves a unary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<MethodInterface> {
        None
    }

    /// Resolves a binary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_own_binary_operation(operation: &BinaryOperation) -> Option<MethodInterface> {
        None
    }
}

pub(super) use macros::*;

mod macros {
    use super::*;

    macro_rules! ignore_all {
        ($($_:tt)*) => {};
    }

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
                let tmp = handle_arg_name!($($arg_part)+).expect_mutable();
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

    macro_rules! handle_first_arg_type {
        // By value
        ($($arg_part:ident)+ : $type:ty, $($rest:tt)*) => {
            $type
        };
    }

    macro_rules! count {
        () => { 0 };
        ($head:tt $($tail:tt)*) => { 1 + count!($($tail)*) };
    }

    // Creating an inner method vastly improves IDE support when writing the method body
    macro_rules! handle_define_inner_method {
        // The $arg_part+ allows for mut x in the argument list
        ($method_name:ident [$($args:tt)*] $body:block $output_ty:ty) => {
            fn $method_name($($args)*) -> $output_ty {
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

    macro_rules! handle_correct_arity {
        ($method_name:ident ($(,)?) -> $output_ty:ty) => {
            MethodInterface::Arity0 {
                method: |a, output_span_range| apply_fn0($method_name, a, output_span_range),
                argument_ownership: [],
            }
        };
        ($method_name:ident ($($arg_part:ident)+ : $ty:ty $(,)?) -> $output_ty:ty) => {
            MethodInterface::Arity1 {
                method: |a, output_span_range| apply_fn1($method_name, a, output_span_range),
                argument_ownership: [<$ty as FromResolved>::OWNERSHIP],
            }
        };
        ($method_name:ident (
            $($arg_part1:ident)+ : $ty1:ty,
            $($arg_part2:ident)+ : $ty2:ty $(,)?
        ) -> $output_ty:ty) => {
            MethodInterface::Arity2 {
                method: |a, b, output_span_range| apply_fn2($method_name, a, b, output_span_range),
                argument_ownership: [
                    <$ty1 as FromResolved>::OWNERSHIP,
                    <$ty2 as FromResolved>::OWNERSHIP,
                ],
            }
        };
        ($method_name:ident (
            $($arg_part1:ident)+ : $ty1:ty,
            $($arg_part2:ident)+ : $ty2:ty,
            $($arg_part3:ident)+ : $ty3:ty $(,)?
        ) -> $output_ty:ty) => {
            MethodInterface::Arity3 {
                method: |a, b, c, output_span_range| {
                    apply_fn3($method_name, a, b, c, output_span_range)
                },
                argument_ownership: [
                    <$ty1 as FromResolved>::OWNERSHIP,
                    <$ty2 as FromResolved>::OWNERSHIP,
                    <$ty3 as FromResolved>::OWNERSHIP,
                ],
            }
        }; // TODO: Add back in if needed (e.g. if wanting to support variadic functions)
           // ($method_name:ident ($($args:tt)*) -> $output_ty:ty) => {
           //     MethodInterface::ArityAny {
           //         method: |
           //             all_arguments: Vec<ResolvedValue>,
           //             output_span_range: SpanRange,
           //         | -> ExecutionResult<ResolvedValue> {
           //             handle_arg_separation!([$($args)*], all_arguments, output_span_range);
           //             handle_arg_mapping!([$($args)*,] []);
           //             let output = handle_call_inner_method!(inner_method [$($args)*]);
           //             <$output_ty as ResolvableOutput>::to_resolved_value(output, output_span_range)
           //         },
           //         argument_ownership: handle_arg_ownerships!([$($args)*,] []),
           //     }
           // };
    }

    // NOTE: We use function pointers here rather than generics to avoid monomorphization bloat.
    // This means that we only need to compile the mapping glue combination once for each (A, B) -> C combination

    pub(crate) fn apply_fn0<R>(
        f: fn() -> R,
        output_span_range: SpanRange,
    ) -> ExecutionResult<ResolvedValue>
    where
        R: ResolvableOutput,
    {
        f().to_resolved_value(output_span_range)
    }

    pub(crate) fn apply_fn1<A, R>(
        f: fn(A) -> R,
        a: ResolvedValue,
        output_span_range: SpanRange,
    ) -> ExecutionResult<ResolvedValue>
    where
        A: FromResolved,
        R: ResolvableOutput,
    {
        f(A::from_resolved(a)?).to_resolved_value(output_span_range)
    }

    pub(crate) fn apply_fn2<A, B, C>(
        f: fn(A, B) -> C,
        a: ResolvedValue,
        b: ResolvedValue,
        output_span_range: SpanRange,
    ) -> ExecutionResult<ResolvedValue>
    where
        A: FromResolved,
        B: FromResolved,
        C: ResolvableOutput,
    {
        f(A::from_resolved(a)?, B::from_resolved(b)?).to_resolved_value(output_span_range)
    }

    pub(crate) fn apply_fn3<A, B, C, R>(
        f: fn(A, B, C) -> R,
        a: ResolvedValue,
        b: ResolvedValue,
        c: ResolvedValue,
        output_span_range: SpanRange,
    ) -> ExecutionResult<ResolvedValue>
    where
        A: FromResolved,
        B: FromResolved,
        C: FromResolved,
        R: ResolvableOutput,
    {
        f(
            A::from_resolved(a)?,
            B::from_resolved(b)?,
            C::from_resolved(c)?,
        )
        .to_resolved_value(output_span_range)
    }

    macro_rules! wrap_method {
        (($($args:tt)*) -> $output_ty:ty $body:block) => {{
            fn inner_method($($args)*) -> $output_ty {
                $body
            }
            handle_correct_arity!(inner_method($($args)*) -> $output_ty)
        }};
    }

    macro_rules! define_method_matcher {
        (
            (match $var_method_name:ident on $self:ident)
            $(
                fn $method_name:ident($($args:tt)*) -> $output_ty:ty $body:block
            )*
        ) => {
            $(
                $self::assert_first_argument::<handle_first_arg_type!($($args)*,)>();
            )*
            Some(match $var_method_name {
                $(
                    stringify!($method_name) => wrap_method!(($($args)*) -> $output_ty $body),
                )*
                _ => return None,
            })
        }
    }

    pub(crate) use {
        count, define_method_matcher, handle_arg_mapping, handle_arg_name, handle_arg_ownerships,
        handle_arg_separation, handle_call_inner_method, handle_correct_arity,
        handle_define_inner_method, handle_first_arg_type, ignore_all, wrap_method,
    };
}

pub(crate) enum MethodInterface {
    Arity0 {
        method: fn(SpanRange) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 0],
    },
    Arity1 {
        method: fn(ResolvedValue, SpanRange) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 1],
    },
    Arity2 {
        method: fn(ResolvedValue, ResolvedValue, SpanRange) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 2],
    },
    Arity3 {
        method: fn(
            ResolvedValue,
            ResolvedValue,
            ResolvedValue,
            SpanRange,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 3],
    },
    ArityAny {
        method: fn(Vec<ResolvedValue>, SpanRange) -> ExecutionResult<ResolvedValue>,
        argument_ownership: Vec<ResolvedValueOwnership>,
    },
}

impl MethodInterface {
    pub(crate) fn execute(
        &self,
        arguments: Vec<ResolvedValue>,
        span_range: SpanRange,
    ) -> ExecutionResult<ResolvedValue> {
        match self {
            MethodInterface::Arity0 { method, .. } => {
                if !arguments.is_empty() {
                    return span_range.execution_err("Expected 0 arguments");
                }
                method(span_range)
            }
            MethodInterface::Arity1 { method, .. } => {
                match <[ResolvedValue; 1]>::try_from(arguments) {
                    Ok([a]) => method(a, span_range),
                    Err(_) => span_range.execution_err("Expected 1 argument"),
                }
            }
            MethodInterface::Arity2 { method, .. } => {
                match <[ResolvedValue; 2]>::try_from(arguments) {
                    Ok([a, b]) => method(a, b, span_range),
                    Err(_) => span_range.execution_err("Expected 2 arguments"),
                }
            }
            MethodInterface::Arity3 { method, .. } => {
                match <[ResolvedValue; 3]>::try_from(arguments) {
                    Ok([a, b, c]) => method(a, b, c, span_range),
                    Err(_) => span_range.execution_err("Expected 3 arguments"),
                }
            }
            MethodInterface::ArityAny { method, .. } => method(arguments, span_range),
        }
    }

    pub(crate) fn argument_ownerships(&self) -> &[ResolvedValueOwnership] {
        match self {
            MethodInterface::Arity0 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::Arity1 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::Arity2 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::Arity3 {
                argument_ownership, ..
            } => argument_ownership,
            MethodInterface::ArityAny {
                argument_ownership, ..
            } => argument_ownership,
        }
    }
}

pub(crate) use outputs::*;

mod outputs {
    use super::*;

    // TODO: Find some way to selectively enable only on MSRV (e.g. following the build.rs feature flag pattern)
    // #[diagnostic::on_unimplemented(
    //     message = "`ResolvableOutput` is not implemented for `{Self}`",
    //     note = "`ResolvableOutput` is not implemented for `Shared<X>` or `Mutable<X>` unless `X` is `ExpressionValue`. If we wish to change this, we'd need to have some way to represent some kind of `ExpressionReference`, i.e. a `Typed<Shared<..>>` rather than a `Shared<Typed<..>>`"
    // )]
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

    impl<T: ResolvableOutput> ResolvableOutput for ExecutionResult<T> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            self?.to_resolved_value(output_span_range)
        }
    }
}

pub(crate) use arguments::*;

mod arguments {
    use super::*;

    pub(crate) trait FromResolved: Sized {
        type ValueType: MethodResolutionTarget;
        const OWNERSHIP: ResolvedValueOwnership;
        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self>;
    }

    impl<T: ResolvableArgument> FromResolved for Owned<T> {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            value
                .expect_owned()
                .try_map(|v, _| T::resolve_from_owned(v))
        }
    }

    impl<T: ResolvableArgument> FromResolved for Shared<T> {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Shared;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            value.expect_shared().try_map(|v, _| T::resolve_from_ref(v))
        }
    }

    impl<T: ResolvableArgument> FromResolved for Mutable<T> {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Mutable;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            value
                .expect_mutable()
                .try_map(|v, _| T::resolve_from_mut(v))
        }
    }

    impl<T: ResolvableArgument> FromResolved for T {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            T::resolve_from_owned(value.expect_owned().into_inner())
        }
    }

    impl<T: ResolvableArgument> FromResolved for CopyOnWrite<T> {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::CopyOnWrite;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            value
                .expect_copy_on_write()
                .map_any(T::resolve_shared, T::resolve_owned)
        }
    }

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
        type ValueType: MethodResolutionTarget;

        fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self>;
        fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self>;
        fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self>;

        fn resolve_owned(value: Owned<ExpressionValue>) -> ExecutionResult<Owned<Self>> {
            value.try_map(|v, _| Self::resolve_from_owned(v))
        }
        fn resolve_shared(value: Shared<ExpressionValue>) -> ExecutionResult<Shared<Self>> {
            value.try_map(|v, _| Self::resolve_from_ref(v))
        }
        fn resolve_mutable(value: Mutable<ExpressionValue>) -> ExecutionResult<Mutable<Self>> {
            value.try_map(|v, _| Self::resolve_from_mut(v))
        }
    }

    impl ResolvableArgument for ExpressionValue {
        type ValueType = ValueTypeData;

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
        ($value_type:ty, ($value:ident) -> $type:ty $body:block) => {
            impl ResolvableArgument for $type {
                type ValueType = $value_type;

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

    macro_rules! impl_delegated_resolvable_argument_for {
        ($value_type:ty, ($value:ident: $delegate:ty) -> $type:ty { $expr:expr }) => {
            impl ResolvableArgument for $type {
                type ValueType = $value_type;

                fn resolve_from_owned(input_value: ExpressionValue) -> ExecutionResult<Self> {
                    let $value: $delegate = input_value.resolve_as()?;
                    Ok($expr)
                }

                fn resolve_from_ref(input_value: &ExpressionValue) -> ExecutionResult<&Self> {
                    let $value: &$delegate = input_value.resolve_as()?;
                    Ok(&$expr)
                }

                fn resolve_from_mut(
                    input_value: &mut ExpressionValue,
                ) -> ExecutionResult<&mut Self> {
                    let $value: &mut $delegate = input_value.resolve_as()?;
                    Ok(&mut $expr)
                }
            }
        };
    }

    pub(crate) use impl_resolvable_argument_for;

    impl_resolvable_argument_for! {
        BooleanTypeData,
        (value) -> ExpressionBoolean {
            match value {
                ExpressionValue::Boolean(value) => Ok(value),
                _ => value.execution_err("Expected boolean"),
            }
        }
    }

    impl_delegated_resolvable_argument_for! {
        BooleanTypeData,
        (value: ExpressionBoolean) -> bool { value.value }
    }

    // Integer types
    impl_resolvable_argument_for! {
        IntegerTypeData,
        (value) -> ExpressionInteger {
            match value {
                ExpressionValue::Integer(value) => Ok(value),
                _ => value.execution_err("Expected integer"),
            }
        }
    }

    impl_resolvable_argument_for! {
        UntypedIntegerTypeData,
        (value) -> UntypedInteger {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::Untyped(x), ..}) => Ok(x),
                _ => value.execution_err("Expected untyped integer"),
            }
        }
    }

    impl_resolvable_argument_for! {
        I8TypeData,
        (value) -> i8 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::I8(x), ..}) => Ok(x),
                _ => value.execution_err("Expected i8"),
            }
        }
    }

    impl_resolvable_argument_for! {
        I16TypeData,
        (value) -> i16 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::I16(x), ..}) => Ok(x),
                _ => value.execution_err("Expected i16"),
            }
        }
    }

    impl_resolvable_argument_for! {
        I32TypeData,
        (value) -> i32 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::I32(x), ..}) => Ok(x),
                _ => value.execution_err("Expected i32"),
            }
        }
    }

    impl_resolvable_argument_for! {
        I64TypeData,
        (value) -> i64 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::I64(x), ..}) => Ok(x),
                _ => value.execution_err("Expected i64"),
            }
        }
    }

    impl_resolvable_argument_for! {
        I128TypeData,
        (value) -> i128 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::I128(x), ..}) => Ok(x),
                _ => value.execution_err("Expected i128"),
            }
        }
    }

    impl_resolvable_argument_for! {
        IsizeTypeData,
        (value) -> isize {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::Isize(x), ..}) => Ok(x),
                _ => value.execution_err("Expected isize"),
            }
        }
    }

    impl_resolvable_argument_for! {
        U8TypeData,
        (value) -> u8 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U8(x), ..}) => Ok(x),
                _ => value.execution_err("Expected u8"),
            }
        }
    }

    impl_resolvable_argument_for! {
        U16TypeData,
        (value) -> u16 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U16(x), ..}) => Ok(x),
                _ => value.execution_err("Expected u16"),
            }
        }
    }

    impl_resolvable_argument_for! {
        U32TypeData,
        (value) -> u32 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U32(x), ..}) => Ok(x),
                _ => value.execution_err("Expected u32"),
            }
        }
    }

    impl_resolvable_argument_for! {
        U64TypeData,
        (value) -> u64 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U64(x), ..}) => Ok(x),
                _ => value.execution_err("Expected u64"),
            }
        }
    }

    impl_resolvable_argument_for! {
        U128TypeData,
        (value) -> u128 {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::U128(x), ..}) => Ok(x),
                _ => value.execution_err("Expected u128"),
            }
        }
    }

    impl_resolvable_argument_for! {
        UsizeTypeData,
        (value) -> usize {
            match value {
                ExpressionValue::Integer(ExpressionInteger { value: ExpressionIntegerValue::Usize(x), ..}) => Ok(x),
                _ => value.execution_err("Expected usize"),
            }
        }
    }

    // Float types
    impl_resolvable_argument_for! {
        FloatTypeData,
        (value) -> ExpressionFloat {
            match value {
                ExpressionValue::Float(value) => Ok(value),
                _ => value.execution_err("Expected float"),
            }
        }
    }

    impl_resolvable_argument_for! {
        UntypedFloatTypeData,
        (value) -> UntypedFloat {
            match value {
                ExpressionValue::Float(ExpressionFloat { value: ExpressionFloatValue::Untyped(x), ..}) => Ok(x),
                _ => value.execution_err("Expected untyped float"),
            }
        }
    }

    impl_resolvable_argument_for! {
        F32TypeData,
        (value) -> f32 {
            match value {
                ExpressionValue::Float(ExpressionFloat { value: ExpressionFloatValue::F32(x), ..}) => Ok(x),
                _ => value.execution_err("Expected f32"),
            }
        }
    }

    impl_resolvable_argument_for! {
        F64TypeData,
        (value) -> f64 {
            match value {
                ExpressionValue::Float(ExpressionFloat { value: ExpressionFloatValue::F64(x), ..}) => Ok(x),
                _ => value.execution_err("Expected f64"),
            }
        }
    }

    impl_resolvable_argument_for! {
        StringTypeData,
        (value) -> ExpressionString {
            match value {
                ExpressionValue::String(value) => Ok(value),
                _ => value.execution_err("Expected string"),
            }
        }
    }

    impl<'a> ResolveAs<&'a str> for &'a ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<&'a str> {
            match self {
                ExpressionValue::String(s) => Ok(&s.value),
                _ => self.execution_err("Expected string"),
            }
        }
    }

    impl_resolvable_argument_for! {
        CharTypeData,
        (value) -> ExpressionChar {
            match value {
                ExpressionValue::Char(value) => Ok(value),
                _ => value.execution_err("Expected char"),
            }
        }
    }

    impl_resolvable_argument_for! {
        ArrayTypeData,
        (value) -> ExpressionArray {
            match value {
                ExpressionValue::Array(value) => Ok(value),
                _ => value.execution_err("Expected array"),
            }
        }
    }

    impl_resolvable_argument_for! {
        ObjectTypeData,
        (value) -> ExpressionObject {
            match value {
                ExpressionValue::Object(value) => Ok(value),
                _ => value.execution_err("Expected object"),
            }
        }
    }

    impl_resolvable_argument_for! {
        StreamTypeData,
        (value) -> ExpressionStream {
            match value {
                ExpressionValue::Stream(value) => Ok(value),
                _ => value.execution_err("Expected stream"),
            }
        }
    }

    impl_resolvable_argument_for! {
        RangeTypeData,
        (value) -> ExpressionRange {
            match value {
                ExpressionValue::Range(value) => Ok(value),
                _ => value.execution_err("Expected range"),
            }
        }
    }

    impl_resolvable_argument_for! {
        IteratorTypeData,
        (value) -> ExpressionIterator {
            match value {
                ExpressionValue::Iterator(value) => Ok(value),
                _ => value.execution_err("Expected iterator"),
            }
        }
    }
}
