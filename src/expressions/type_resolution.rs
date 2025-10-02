#![allow(unused)]
use std::mem;

// TODO[unused-clearup]
use super::*;

pub(crate) struct UnaryOperationInterface {
    pub method: fn(UnaryOperationCallContext, ResolvedValue) -> ExecutionResult<ResolvedValue>,
    pub argument_ownership: ResolvedValueOwnership,
}

impl UnaryOperationInterface {
    pub(super) fn execute(
        &self,
        input: ResolvedValue,
        operation: &UnaryOperation,
    ) -> ExecutionResult<ResolvedValue> {
        let output_span_range = operation.output_span_range(input.span_range());
        (self.method)(
            UnaryOperationCallContext {
                operation,
                output_span_range,
            },
            input,
        )
    }

    pub(super) fn argument_ownership(&self) -> ResolvedValueOwnership {
        self.argument_ownership
    }
}

pub(super) trait MethodResolver {
    /// Resolves a unary operation as a method interface for this type.
    fn resolve_method(&self, method_name: &str) -> Option<MethodInterface>;

    /// Resolves a unary operation as a method interface for this type.
    /// Returns None if the operation should fallback to the legacy system.
    fn resolve_unary_operation(
        &self,
        operation: &UnaryOperation,
    ) -> Option<UnaryOperationInterface>;

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

    fn resolve_unary_operation(
        &self,
        operation: &UnaryOperation,
    ) -> Option<UnaryOperationInterface> {
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
    fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
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

    macro_rules! if_empty {
        ([] [$($output:tt)*]) => {
            $($output)*
        };
        ([$($input:tt)+] [$($output:tt)*]) => {
            $($input)*
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

    macro_rules! create_method_interface {
        ($method_name:path[$(,)?]) => {
            MethodInterface::Arity0 {
                method: |context| apply_fn0($method_name, context),
                argument_ownership: [],
            }
        };
        ($method_name:path[$($arg_part:ident)+ : $ty:ty $(,)?]) => {
            MethodInterface::Arity1 {
                method: |context, a| apply_fn1($method_name, a, context),
                argument_ownership: [<$ty as FromResolved>::OWNERSHIP],
            }
        };
        ($method_name:path[
            $($arg_part1:ident)+ : $ty1:ty,
            $($arg_part2:ident)+ : $ty2:ty $(,)?
        ]) => {
            MethodInterface::Arity2 {
                method: |context, a, b| apply_fn2($method_name, a, b, context),
                argument_ownership: [
                    <$ty1 as FromResolved>::OWNERSHIP,
                    <$ty2 as FromResolved>::OWNERSHIP,
                ],
            }
        };
        ($method_name:path[
            $($arg_part1:ident)+ : $ty1:ty,
            $($arg_part2:ident)+ : $ty2:ty,
            $($arg_part3:ident)+ : $ty3:ty $(,)?
        ]) => {
            MethodInterface::Arity3 {
                method: |context, a, b, c| apply_fn3($method_name, a, b, c, context),
                argument_ownership: [
                    <$ty1 as FromResolved>::OWNERSHIP,
                    <$ty2 as FromResolved>::OWNERSHIP,
                    <$ty3 as FromResolved>::OWNERSHIP,
                ],
            }
        };
    }

    // NOTE: We use function pointers here rather than generics to avoid monomorphization bloat.
    // This means that we only need to compile the mapping glue combination once for each (A, B) -> C combination

    pub(crate) fn apply_fn0<R>(
        f: fn(MethodCallContext) -> R,
        context: MethodCallContext,
    ) -> ExecutionResult<ResolvedValue>
    where
        R: ResolvableOutput,
    {
        let output_span_range = context.output_span_range;
        f(context).to_resolved_value(output_span_range)
    }

    pub(crate) fn apply_fn1<A, R>(
        f: fn(MethodCallContext, A) -> R,
        a: ResolvedValue,
        context: MethodCallContext,
    ) -> ExecutionResult<ResolvedValue>
    where
        A: FromResolved,
        R: ResolvableOutput,
    {
        let output_span_range = context.output_span_range;
        f(context, A::from_resolved(a)?).to_resolved_value(output_span_range)
    }

    pub(crate) fn apply_fn2<A, B, C>(
        f: fn(MethodCallContext, A, B) -> C,
        a: ResolvedValue,
        b: ResolvedValue,
        context: MethodCallContext,
    ) -> ExecutionResult<ResolvedValue>
    where
        A: FromResolved,
        B: FromResolved,
        C: ResolvableOutput,
    {
        let output_span_range = context.output_span_range;
        f(context, A::from_resolved(a)?, B::from_resolved(b)?).to_resolved_value(output_span_range)
    }

    pub(crate) fn apply_fn3<A, B, C, R>(
        f: fn(MethodCallContext, A, B, C) -> R,
        a: ResolvedValue,
        b: ResolvedValue,
        c: ResolvedValue,
        context: MethodCallContext,
    ) -> ExecutionResult<ResolvedValue>
    where
        A: FromResolved,
        B: FromResolved,
        C: FromResolved,
        R: ResolvableOutput,
    {
        let output_span_range = context.output_span_range;
        f(
            context,
            A::from_resolved(a)?,
            B::from_resolved(b)?,
            C::from_resolved(c)?,
        )
        .to_resolved_value(output_span_range)
    }

    macro_rules! create_unary_interface {
        ($method_name:path[$($arg_part:ident)+ : $ty:ty $(,)?]) => {
            UnaryOperationInterface {
                method: |context, a| apply_unary_fn($method_name, a, context),
                argument_ownership: <$ty as FromResolved>::OWNERSHIP,
            }
        };
    }

    pub(crate) fn apply_unary_fn<A, R>(
        f: fn(UnaryOperationCallContext, A) -> R,
        a: ResolvedValue,
        context: UnaryOperationCallContext,
    ) -> ExecutionResult<ResolvedValue>
    where
        A: FromResolved,
        R: ResolvableOutput,
    {
        let output_span_range = context.output_span_range;
        f(context, A::from_resolved(a)?).to_resolved_value(output_span_range)
    }

    macro_rules! wrap_method {
        ($([$context:ident])? ($($args:tt)*) $(-> $output_ty:ty)? $body:block) => {{
            fn inner_method(if_empty!([$($context)?][_context]): MethodCallContext, $($args)*) $(-> $output_ty)? {
                $body
            }
            create_method_interface!(inner_method[$($args)*])
        }};
    }

    pub(crate) struct MethodCallContext<'a> {
        pub interpreter: &'a mut Interpreter,
        pub output_span_range: SpanRange,
    }

    macro_rules! define_method_matcher {
        (
            (match $var_method_name:ident on $self:ident)
            $(
                $([$context:ident])? fn $method_name:ident($($args:tt)*) $(-> $output_ty:ty)? $body:block
            )*
        ) => {
            $(
                $self::assert_first_argument::<handle_first_arg_type!($($args)*,)>();
            )*
            Some(match $var_method_name {
                $(
                    stringify!($method_name) => wrap_method!($([$context])? ($($args)*) $(-> $output_ty)? $body),
                )*
                _ => return None,
            })
        }
    }

    macro_rules! wrap_unary {
        ($([$context:ident])?($($args:tt)*) $(-> $output_ty:ty)? $body:block) => {{
            fn inner_method(if_empty!([$($context)?][_context]): UnaryOperationCallContext, $($args)*) $(-> $output_ty)? {
                $body
            }
            create_unary_interface!(inner_method[$($args)*])
        }};
    }

    pub(crate) struct UnaryOperationCallContext<'a> {
        pub operation: &'a UnaryOperation,
        pub output_span_range: SpanRange,
    }

    macro_rules! define_interface {
        (
            struct $type_data:ident,
            parent: $parent_type_data:ident,
            $mod_vis:vis mod $mod_name:ident {
                $mod_methods_vis:vis mod methods {
                    $(
                        $([$method_context:ident])? fn $method_name:ident($($method_args:tt)*) $(-> $method_output_ty:ty)? $method_body:block
                    )*
                }
                $mod_unary_operations_vis:vis mod unary_operations {
                    $(
                        $([$unary_context:ident])? fn $unary_name:ident($($unary_args:tt)*) $(-> $unary_output_ty:ty)? $unary_body:block
                    )*
                }
                interface_items {
                    $($items:item)*
                }
            }
        ) => {
            #[derive(Clone, Copy)]
            pub(crate) struct $type_data;

            $mod_vis mod $mod_name {
                use super::*;

                $mod_vis const fn parent() -> Option<$parent_type_data> {
                    // Type ids aren't const, and strings aren't const-comparable, but I can use this work-around:
                    // https://internals.rust-lang.org/t/why-i-cannot-compare-two-static-str-s-in-a-const-context/17726/2
                    const OWN_TYPE_NAME: &'static [u8] = stringify!($type_data).as_bytes();
                    const PARENT_TYPE_NAME: &'static [u8] = stringify!($parent_type_data).as_bytes();
                    match PARENT_TYPE_NAME {
                        OWN_TYPE_NAME => None,
                        _ => Some($parent_type_data),
                    }
                }

                #[allow(unused)]
                fn asserts() {
                    $(
                        $type_data::assert_first_argument::<handle_first_arg_type!($($method_args)*,)>();
                    )*
                    $(
                        $type_data::assert_first_argument::<handle_first_arg_type!($($unary_args)*,)>();
                    )*
                }

                $mod_methods_vis mod methods {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $method_name(if_empty!([$($method_context)?][_context]): MethodCallContext, $($method_args)*) $(-> $method_output_ty)? {
                            $method_body
                        }
                    )*
                }

                $mod_methods_vis mod method_definitions {
                    #[allow(unused)]
                    use super::*;
                    $(
                        $mod_methods_vis fn $method_name() -> MethodInterface {
                            create_method_interface!(methods::$method_name[$($method_args)*])
                        }
                    )*
                }

                $mod_unary_operations_vis mod unary_operations {
                    #[allow(unused)]
                    use super::*;
                    $(
                        $mod_unary_operations_vis fn $unary_name(if_empty!([$($unary_context)?][_context]): UnaryOperationCallContext, $($unary_args)*) $(-> $unary_output_ty)? {
                            $unary_body
                        }
                    )*
                }

                $mod_unary_operations_vis mod unary_definitions {
                    #[allow(unused)]
                    use super::*;
                    $(
                        $mod_unary_operations_vis fn $unary_name() -> UnaryOperationInterface {
                            create_unary_interface!(unary_operations::$unary_name[$($unary_args)*])
                        }
                    )*
                }

                impl MethodResolutionTarget for $type_data {
                    type Parent = $parent_type_data;
                    const PARENT: Option<Self::Parent> = $mod_name::parent();

                    #[allow(unreachable_code)]
                    fn resolve_own_method(method_name: &str) -> Option<MethodInterface> {
                        Some(match method_name {
                            $(
                                stringify!($method_name) => method_definitions::$method_name(),
                            )*
                            _ => return None,
                        })
                    }

                    // Pass through resolve_own_unary_operation until there's a better way to define them
                    $($items)*
                }
            }
        }
    }

    pub(crate) use {
        count, create_method_interface, create_unary_interface, define_interface,
        define_method_matcher, handle_first_arg_type, if_empty, ignore_all, wrap_method,
        wrap_unary,
    };
}

pub(crate) enum MethodInterface {
    Arity0 {
        method: fn(MethodCallContext) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 0],
    },
    Arity1 {
        method: fn(MethodCallContext, ResolvedValue) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 1],
    },
    Arity2 {
        method:
            fn(MethodCallContext, ResolvedValue, ResolvedValue) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 2],
    },
    Arity3 {
        method: fn(
            MethodCallContext,
            ResolvedValue,
            ResolvedValue,
            ResolvedValue,
        ) -> ExecutionResult<ResolvedValue>,
        argument_ownership: [ResolvedValueOwnership; 3],
    },
    ArityAny {
        method: fn(MethodCallContext, Vec<ResolvedValue>) -> ExecutionResult<ResolvedValue>,
        argument_ownership: Vec<ResolvedValueOwnership>,
    },
}

impl MethodInterface {
    pub(crate) fn execute(
        &self,
        arguments: Vec<ResolvedValue>,
        context: MethodCallContext,
    ) -> ExecutionResult<ResolvedValue> {
        match self {
            MethodInterface::Arity0 { method, .. } => {
                if !arguments.is_empty() {
                    return context
                        .output_span_range
                        .execution_err("Expected 0 arguments");
                }
                method(context)
            }
            MethodInterface::Arity1 { method, .. } => {
                match <[ResolvedValue; 1]>::try_from(arguments) {
                    Ok([a]) => method(context, a),
                    Err(_) => context
                        .output_span_range
                        .execution_err("Expected 1 argument"),
                }
            }
            MethodInterface::Arity2 { method, .. } => {
                match <[ResolvedValue; 2]>::try_from(arguments) {
                    Ok([a, b]) => method(context, a, b),
                    Err(_) => context
                        .output_span_range
                        .execution_err("Expected 2 arguments"),
                }
            }
            MethodInterface::Arity3 { method, .. } => {
                match <[ResolvedValue; 3]>::try_from(arguments) {
                    Ok([a, b, c]) => method(context, a, b, c),
                    Err(_) => context
                        .output_span_range
                        .execution_err("Expected 3 arguments"),
                }
            }
            MethodInterface::ArityAny { method, .. } => method(context, arguments),
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

    impl ResolvableOutput for ResolvedValue {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(self.with_span_range(output_span_range))
        }
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

    impl<T: ToExpressionValue> ResolvableOutput for T {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Owned(
                self.to_value(output_span_range).into(),
            ))
        }
    }

    impl<T: ToExpressionValue> ResolvableOutput for Owned<T> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            Ok(ResolvedValue::Owned(
                self.map(|f, _| f.to_value(output_span_range))
                    .with_span_range(output_span_range),
            ))
        }
    }

    impl<T: ResolvableOutput> ResolvableOutput for ExecutionResult<T> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            self?.to_resolved_value(output_span_range)
        }
    }

    pub trait StreamAppender {
        fn append(self, output: &mut OutputStream) -> ExecutionResult<()>;
    }
    impl<F: FnOnce(&mut OutputStream) -> ExecutionResult<()>> StreamAppender for F {
        fn append(self, output: &mut OutputStream) -> ExecutionResult<()> {
            self(output)
        }
    }

    pub(crate) struct StreamOutput<T: StreamAppender>(T);
    impl<F: FnOnce(&mut OutputStream) -> ExecutionResult<()>> StreamOutput<F> {
        pub fn new(appender: F) -> Self {
            Self(appender)
        }
    }
    impl<T: StreamAppender> From<T> for StreamOutput<T> {
        fn from(value: T) -> Self {
            Self(value)
        }
    }
    impl<T: StreamAppender> ResolvableOutput for StreamOutput<T> {
        fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
            let mut output = OutputStream::new();
            self.0.append(&mut output)?;
            output.to_resolved_value(output_span_range)
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

    impl<T: ResolvableArgumentOwned + ResolvableArgumentTarget> FromResolved for Owned<T> {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            value
                .expect_owned()
                .try_map(|v, _| T::resolve_from_owned(v))
        }
    }

    impl<T: ResolvableArgumentShared + ResolvableArgumentTarget> FromResolved for Shared<T> {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Shared;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            value.expect_shared().try_map(|v, _| T::resolve_from_ref(v))
        }
    }

    impl<T: ResolvableArgumentMutable + ResolvableArgumentTarget> FromResolved for Mutable<T> {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Mutable;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            value
                .expect_mutable()
                .try_map(|v, _| T::resolve_from_mut(v))
        }
    }

    impl<T: ResolvableArgumentOwned + ResolvableArgumentTarget> FromResolved for T {
        type ValueType = T::ValueType;
        const OWNERSHIP: ResolvedValueOwnership = ResolvedValueOwnership::Owned;

        fn from_resolved(value: ResolvedValue) -> ExecutionResult<Self> {
            T::resolve_from_owned(value.expect_owned().into_inner())
        }
    }

    impl<T: ResolvableArgumentOwned + ResolvableArgumentShared + ResolvableArgumentTarget>
        FromResolved for CopyOnWrite<T>
    {
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

    impl<T: ResolvableArgumentOwned> ResolveAs<T> for ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<T> {
            T::resolve_from_owned(self)
        }
    }

    impl<'a, T: ResolvableArgumentShared> ResolveAs<&'a T> for &'a ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<&'a T> {
            T::resolve_from_ref(self)
        }
    }

    impl<'a, T: ResolvableArgumentMutable> ResolveAs<&'a mut T> for &'a mut ExpressionValue {
        fn resolve_as(self) -> ExecutionResult<&'a mut T> {
            T::resolve_from_mut(self)
        }
    }

    pub(crate) trait ResolvableArgumentTarget {
        type ValueType: MethodResolutionTarget;
    }

    pub(crate) trait ResolvableArgumentOwned: Sized {
        fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self>;
        fn resolve_owned(value: Owned<ExpressionValue>) -> ExecutionResult<Owned<Self>> {
            value.try_map(|v, _| Self::resolve_from_owned(v))
        }
    }

    pub(crate) trait ResolvableArgumentShared: Sized {
        fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self>;
        fn resolve_shared(value: Shared<ExpressionValue>) -> ExecutionResult<Shared<Self>> {
            value.try_map(|v, _| Self::resolve_from_ref(v))
        }
    }

    pub(crate) trait ResolvableArgumentMutable: Sized {
        fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self>;
        fn resolve_mutable(value: Mutable<ExpressionValue>) -> ExecutionResult<Mutable<Self>> {
            value.try_map(|v, _| Self::resolve_from_mut(v))
        }
    }

    impl ResolvableArgumentTarget for ExpressionValue {
        type ValueType = ValueTypeData;
    }
    impl ResolvableArgumentOwned for ExpressionValue {
        fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
            Ok(value)
        }
    }
    impl ResolvableArgumentShared for ExpressionValue {
        fn resolve_from_ref(value: &ExpressionValue) -> ExecutionResult<&Self> {
            Ok(value)
        }
    }
    impl ResolvableArgumentMutable for ExpressionValue {
        fn resolve_from_mut(value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
            Ok(value)
        }
    }

    macro_rules! impl_resolvable_argument_for {
        ($value_type:ty, ($value:ident) -> $type:ty $body:block) => {
            impl ResolvableArgumentTarget for $type {
                type ValueType = $value_type;
            }

            impl ResolvableArgumentOwned for $type {
                fn resolve_from_owned($value: ExpressionValue) -> ExecutionResult<Self> {
                    $body
                }
            }

            impl ResolvableArgumentShared for $type {
                fn resolve_from_ref($value: &ExpressionValue) -> ExecutionResult<&Self> {
                    $body
                }
            }

            impl ResolvableArgumentMutable for $type {
                fn resolve_from_mut($value: &mut ExpressionValue) -> ExecutionResult<&mut Self> {
                    $body
                }
            }
        };
    }

    macro_rules! impl_delegated_resolvable_argument_for {
        ($value_type:ty, ($value:ident: $delegate:ty) -> $type:ty { $expr:expr }) => {
            impl ResolvableArgumentTarget for $type {
                type ValueType = $value_type;
            }

            impl ResolvableArgumentOwned for $type {
                fn resolve_from_owned(input_value: ExpressionValue) -> ExecutionResult<Self> {
                    let $value: $delegate = input_value.resolve_as()?;
                    Ok($expr)
                }
            }

            impl ResolvableArgumentShared for $type {
                fn resolve_from_ref(input_value: &ExpressionValue) -> ExecutionResult<&Self> {
                    let $value: &$delegate = input_value.resolve_as()?;
                    Ok(&$expr)
                }
            }

            impl ResolvableArgumentMutable for $type {
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

    pub(crate) struct MaybeTypedInt<X>(X);

    impl<X: ResolvableArgumentOwned + FromStr> ResolvableArgumentOwned for MaybeTypedInt<X>
    where
        X::Err: core::fmt::Display,
    {
        fn resolve_from_owned(value: ExpressionValue) -> ExecutionResult<Self> {
            Ok(Self(match value {
                ExpressionValue::Integer(ExpressionInteger {
                    value: ExpressionIntegerValue::Untyped(x),
                    ..
                }) => x.parse_as()?,
                _ => value.resolve_as()?,
            }))
        }
    }

    pub(crate) struct UntypedIntegerFallback(pub FallbackInteger);

    impl ResolvableArgumentTarget for UntypedIntegerFallback {
        type ValueType = UntypedIntegerTypeData;
    }

    impl ResolvableArgumentOwned for UntypedIntegerFallback {
        fn resolve_from_owned(input_value: ExpressionValue) -> ExecutionResult<Self> {
            let value: UntypedInteger = input_value.resolve_as()?;
            Ok(UntypedIntegerFallback(value.parse_fallback()?))
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

    pub(crate) struct UntypedFloatFallback(pub FallbackFloat);

    impl ResolvableArgumentTarget for UntypedFloatFallback {
        type ValueType = UntypedFloatTypeData;
    }

    impl ResolvableArgumentOwned for UntypedFloatFallback {
        fn resolve_from_owned(input_value: ExpressionValue) -> ExecutionResult<Self> {
            let value: UntypedFloat = input_value.resolve_as()?;
            Ok(UntypedFloatFallback(value.parse_fallback()?))
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

    impl_delegated_resolvable_argument_for!(
        StringTypeData,
        (value: ExpressionString) -> String { value.value }
    );

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

    impl_delegated_resolvable_argument_for!(
        CharTypeData,
        (value: ExpressionChar) -> char { value.value }
    );

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
