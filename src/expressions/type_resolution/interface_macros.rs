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

#[cfg(test)]
macro_rules! handle_first_arg_type {
    ([NEXT] : $type:ty) => {
        $type
    };
    ([NEXT] : $type:ty, $($rest:tt)*) => {
        $type
    };
    ([NEXT] $consumed:tt $($rest:tt)*) => {
        handle_first_arg_type!([NEXT] $($rest)*)
    };
    // Start
    ($first:tt $($rest:tt)*) => {
        handle_first_arg_type!([NEXT] $($rest)*)
    };
}

macro_rules! generate_unary_interface {
    (REQUIRED [$ty1:ty,] OPTIONAL [] ARGS[$method:path]) => {
        UnaryOperationInterface {
            method: |context, a| apply_unary_fn($method, a, context),
            argument_ownership: <$ty1 as IsArgument>::OWNERSHIP,
        }
    };
}

macro_rules! generate_binary_interface {
    (REQUIRED [$ty1:ty, $ty2:ty,] OPTIONAL [] ARGS[$method:path]) => {
        BinaryOperationInterface {
            method: |context, a, b| apply_binary_fn($method, a, b, context),
            lhs_ownership: <$ty1 as IsArgument>::OWNERSHIP,
            rhs_ownership: <$ty2 as IsArgument>::OWNERSHIP,
        }
    };
}

macro_rules! generate_method_interface {
    (REQUIRED [] OPTIONAL [] ARGS[$method:path]) => {
        FunctionInterface::Arity0 {
            method: |context| apply_fn0($method, context),
            argument_ownership: [],
        }
    };
    (REQUIRED [$ty1:ty,] OPTIONAL [] ARGS[$method:path]) => {
        FunctionInterface::Arity1 {
            method: |context, a| apply_fn1($method, a, context),
            argument_ownership: [<$ty1 as IsArgument>::OWNERSHIP],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty,] OPTIONAL [] ARGS[$method:path]) => {
        FunctionInterface::Arity2 {
            method: |context, a, b| apply_fn2($method, a, b, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty, $ty3:ty,] OPTIONAL [] ARGS[$method:path]) => {
        FunctionInterface::Arity3 {
            method: |context, a, b, c| apply_fn3($method, a, b, c, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
                <$ty3 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty,] OPTIONAL [$ty2:ty,] ARGS[$method:path]) => {
        FunctionInterface::Arity1PlusOptional1 {
            method: |context, a, b| apply_fn1_optional1($method, a, b, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty,] OPTIONAL [$ty3:ty,] ARGS[$method:path]) => {
        FunctionInterface::Arity2PlusOptional1 {
            method: |context, a, b, c| apply_fn2_optional1($method, a, b, c, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
                <$ty3 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty, $ty3:ty,] OPTIONAL [$ty4:ty,] ARGS[$method:path]) => {
        FunctionInterface::Arity3PlusOptional1 {
            method: |context, a, b, c, d| apply_fn3_optional1($method, a, b, c, d, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
                <$ty3 as IsArgument>::OWNERSHIP,
                <$ty4 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED $req:tt OPTIONAL $opt:tt ARGS[$($arg:tt)*]) => {
        compile_error!(stringify!("This method arity is currently unsupported - add support in `FunctionInterface` and `generate_method_interface`: ", $($arg)*));
    };
}

macro_rules! parse_arg_types {
    // Nothing left - activate callback
    ([REQUIRED $req:tt OPTIONAL $opt:tt => $callback:ident! $callback_args:tt]) => {
        // log_syntax!(REQUIRED $req OPTIONAL $opt ARGS $callback_args)
        $callback!(REQUIRED $req OPTIONAL $opt ARGS $callback_args)
    };
    // Next tokens are `: Option<X>` - we have an optional argument
    ([REQUIRED $req:tt OPTIONAL[$($opt:tt)*] => $callback:ident! $callback_args:tt] : Option<$type:ty> $($rest:tt)*) => {
        parse_arg_types!([REQUIRED $req OPTIONAL[$($opt)* $type,] => $callback! $callback_args] $($rest)*)
    };
    // Next tokens are `: X)` - we have a required argument (variant 1)
    ([REQUIRED [$($req:tt)*] OPTIONAL [] => $callback:ident! $callback_args:tt] : $type:ty) => {
        parse_arg_types!([REQUIRED[$($req)* $type,] OPTIONAL [] => $callback! $callback_args])
    };
    ([REQUIRED [$($req:tt)*] OPTIONAL $opt:tt => $callback:ident! $callback_args:tt] : $type:ty) => {
        compile_error!(stringify!("Required arguments must come before optional arguments:" $type));
    };
    // Next tokens are `: X, ...` - we have a required argument (variant 2)
    ([REQUIRED [$($req:tt)*] OPTIONAL [] => $callback:ident! $callback_args:tt] : $type:ty, $($rest:tt)*) => {
        parse_arg_types!([REQUIRED[$($req)* $type,] OPTIONAL [] => $callback! $callback_args] $($rest)*)
    };
    ([REQUIRED [$($req:tt)*] OPTIONAL $opt:tt => $callback:ident! $callback_args:tt] : $type:ty, $($rest:tt)*) => {
        compile_error!(stringify!("Required arguments must come before optional arguments:" $type));
    };
    // Next tokens are something else - ignore it and look at next
    ([REQUIRED $req:tt OPTIONAL $opt:tt => $callback:ident! $callback_args:tt] $consumed:tt $($rest:tt)*) => {
        parse_arg_types!([REQUIRED $req OPTIONAL $opt => $callback! $callback_args] $($rest)*)
    };
    // Start
    ([CALLBACK: $callback:ident! $callback_args:tt] $($rest:tt)*) => {
        parse_arg_types!([REQUIRED[] OPTIONAL[] => $callback! $callback_args] $($rest)*)
    };
}

// NOTE: We use function pointers here rather than generics to avoid monomorphization bloat.
// This means that we only need to compile the mapping glue combination once for each (A, B) -> C combination

#[allow(unused)]
pub(crate) fn apply_fn0<R>(
    f: fn(&mut FunctionCallContext) -> R,
    context: &mut FunctionCallContext,
) -> ExecutionResult<ReturnedValue>
where
    R: IsReturnable,
{
    let _output_span_range = context.output_span_range;
    f(context).to_returned_value()
}

pub(crate) fn apply_fn1<A, R>(
    f: fn(&mut FunctionCallContext, A) -> R,
    a: Spanned<ArgumentValue>,
    context: &mut FunctionCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    R: IsReturnable,
{
    f(context, A::from_argument(a)?).to_returned_value()
}

#[allow(unused)]
pub(crate) fn apply_fn1_optional1<A, B, C>(
    f: fn(&mut FunctionCallContext, A, Option<B>) -> C,
    a: Spanned<ArgumentValue>,
    b: Option<Spanned<ArgumentValue>>,
    context: &mut FunctionCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    B: IsArgument,
    C: IsReturnable,
{
    let output_span_range = context.output_span_range;
    f(
        context,
        A::from_argument(a)?,
        b.map(|b| B::from_argument(b)).transpose()?,
    )
    .to_returned_value()
}

pub(crate) fn apply_fn2<A, B, C>(
    f: fn(&mut FunctionCallContext, A, B) -> C,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    context: &mut FunctionCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    B: IsArgument,
    C: IsReturnable,
{
    f(context, A::from_argument(a)?, B::from_argument(b)?).to_returned_value()
}

pub(crate) fn apply_fn2_optional1<A, B, C, D>(
    f: fn(&mut FunctionCallContext, A, B, Option<C>) -> D,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    c: Option<Spanned<ArgumentValue>>,
    context: &mut FunctionCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    B: IsArgument,
    C: IsArgument,
    D: IsReturnable,
{
    f(
        context,
        A::from_argument(a)?,
        B::from_argument(b)?,
        c.map(|c| C::from_argument(c)).transpose()?,
    )
    .to_returned_value()
}

#[allow(unused)]
pub(crate) fn apply_fn3<A, B, C, R>(
    f: fn(&mut FunctionCallContext, A, B, C) -> R,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    c: Spanned<ArgumentValue>,
    context: &mut FunctionCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    B: IsArgument,
    C: IsArgument,
    R: IsReturnable,
{
    f(
        context,
        A::from_argument(a)?,
        B::from_argument(b)?,
        C::from_argument(c)?,
    )
    .to_returned_value()
}

pub(crate) fn apply_fn3_optional1<A, B, C, D, R>(
    f: fn(&mut FunctionCallContext, A, B, C, Option<D>) -> R,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    c: Spanned<ArgumentValue>,
    d: Option<Spanned<ArgumentValue>>,
    context: &mut FunctionCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    B: IsArgument,
    C: IsArgument,
    D: IsArgument,
    R: IsReturnable,
{
    f(
        context,
        A::from_argument(a)?,
        B::from_argument(b)?,
        C::from_argument(c)?,
        d.map(|d| D::from_argument(d)).transpose()?,
    )
    .to_returned_value()
}

pub(crate) fn apply_unary_fn<A, R>(
    f: fn(UnaryOperationCallContext, A) -> R,
    a: Spanned<ArgumentValue>,
    context: UnaryOperationCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    R: IsReturnable,
{
    f(context, A::from_argument(a)?).to_returned_value()
}

pub(crate) fn apply_binary_fn<A, B, R>(
    f: fn(BinaryOperationCallContext, A, B) -> R,
    lhs: Spanned<ArgumentValue>,
    rhs: Spanned<ArgumentValue>,
    context: BinaryOperationCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    B: IsArgument,
    R: IsReturnable,
{
    f(context, A::from_argument(lhs)?, B::from_argument(rhs)?).to_returned_value()
}

// ============================================================================
// Property Access Wrapper Functions
// ============================================================================

pub(crate) fn apply_property_shared<'a, S: ResolvableShared<AnyValue> + ?Sized + 'a>(
    f: for<'b> fn(PropertyAccessCallContext, &'b S) -> ExecutionResult<&'b AnyValue>,
    ctx: PropertyAccessCallContext,
    source: &'a AnyValue,
) -> ExecutionResult<&'a AnyValue> {
    let source = S::resolve_from_ref(
        source,
        ResolutionContext::new(&ctx.property.span_range(), "The property access source"),
    )?;
    f(ctx, source)
}

pub(crate) fn apply_property_mutable<'a, S: ResolvableMutable<AnyValue> + ?Sized + 'a>(
    f: for<'b> fn(PropertyAccessCallContext, &'b mut S, bool) -> ExecutionResult<&'b mut AnyValue>,
    ctx: PropertyAccessCallContext,
    source: &'a mut AnyValue,
    auto_create: bool,
) -> ExecutionResult<&'a mut AnyValue> {
    let source = S::resolve_from_mut(
        source,
        ResolutionContext::new(&ctx.property.span_range(), "The property access source"),
    )?;
    f(ctx, source, auto_create)
}

pub(crate) fn apply_property_owned<S: ResolvableOwned<AnyValue>>(
    f: fn(PropertyAccessCallContext, S) -> ExecutionResult<AnyValue>,
    ctx: PropertyAccessCallContext,
    source: AnyValue,
) -> ExecutionResult<AnyValue> {
    let source = S::resolve_from_value(
        source,
        ResolutionContext::new(&ctx.property.span_range(), "The property access source"),
    )?;
    f(ctx, source)
}

// ============================================================================
// Index Access Wrapper Functions
// ============================================================================

pub(crate) fn apply_index_shared<'a, S: ResolvableShared<AnyValue> + ?Sized + 'a>(
    f: for<'b> fn(
        IndexAccessCallContext,
        &'b S,
        Spanned<AnyValueRef>,
    ) -> ExecutionResult<&'b AnyValue>,
    ctx: IndexAccessCallContext,
    source: &'a AnyValue,
    index: Spanned<AnyValueRef>,
) -> ExecutionResult<&'a AnyValue> {
    let source = S::resolve_from_ref(
        source,
        ResolutionContext::new(&ctx.access.span_range(), "The index access source"),
    )?;
    f(ctx, source, index)
}

pub(crate) fn apply_index_mutable<'a, S: ResolvableMutable<AnyValue> + ?Sized + 'a>(
    f: for<'b> fn(
        IndexAccessCallContext,
        &'b mut S,
        Spanned<AnyValueRef>,
        bool,
    ) -> ExecutionResult<&'b mut AnyValue>,
    ctx: IndexAccessCallContext,
    source: &'a mut AnyValue,
    index: Spanned<AnyValueRef>,
    auto_create: bool,
) -> ExecutionResult<&'a mut AnyValue> {
    let source = S::resolve_from_mut(
        source,
        ResolutionContext::new(&ctx.access.span_range(), "The index access source"),
    )?;
    f(ctx, source, index, auto_create)
}

pub(crate) fn apply_index_owned<S: ResolvableOwned<AnyValue>>(
    f: fn(IndexAccessCallContext, S, Spanned<AnyValueRef>) -> ExecutionResult<AnyValue>,
    ctx: IndexAccessCallContext,
    source: AnyValue,
    index: Spanned<AnyValueRef>,
) -> ExecutionResult<AnyValue> {
    let source = S::resolve_from_value(
        source,
        ResolutionContext::new(&ctx.access.span_range(), "The index access source"),
    )?;
    f(ctx, source, index)
}

pub(crate) struct FunctionCallContext<'a> {
    pub interpreter: &'a mut Interpreter,
    pub output_span_range: SpanRange,
}

impl<'a> HasSpanRange for FunctionCallContext<'a> {
    fn span_range(&self) -> SpanRange {
        self.output_span_range
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UnaryOperationCallContext<'a> {
    pub operation: &'a UnaryOperation,
}

#[derive(Clone, Copy)]
pub(crate) struct BinaryOperationCallContext<'a> {
    pub operation: &'a BinaryOperation,
}

impl<'a> BinaryOperationCallContext<'a> {
    #[allow(unused)]
    pub(crate) fn err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        self.operation.value_err(message)
    }

    pub(crate) fn error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        self.operation.value_error(message)
    }
}

macro_rules! define_type_features {
    (
        impl $type_def:ident,
        $mod_vis:vis mod $mod_name:ident {
            $(functions {
                $(
                    $([$function_context:ident])? fn $function_name:ident($($function_args:tt)*) $(-> $function_output_ty:ty)? $([ignore_type_assertion $function_ignore_type_assertion:tt])? $function_body:block
                )*
            })?
            $(methods {
                $(
                    $([$method_context:ident])? fn $method_name:ident($($method_args:tt)*) $(-> $method_output_ty:ty)? $([ignore_type_assertion $method_ignore_type_assertion:tt])? $method_body:block
                )*
            })?
            $(unary_operations {
                $(
                    $([$unary_context:ident])? fn $unary_name:ident($($unary_args:tt)*) $(-> $unary_output_ty:ty)? $([ignore_type_assertion $unary_ignore_type_assertion:tt])? $unary_body:block
                )*
            })?
            $(binary_operations {
                $(
                    $([$binary_context:ident])? fn $binary_name:ident($($binary_args:tt)*) $(-> $binary_output_ty:ty)? $([ignore_type_assertion $binary_ignore_type_assertion:tt])? $binary_body:block
                )*
            })?
            $(property_access($property_source_ty:ty) {
                $([$property_shared_context:ident])? fn shared($($property_shared_args:tt)*) $property_shared_body:block
                $([$property_mutable_context:ident])? fn mutable($($property_mutable_args:tt)*) $property_mutable_body:block
                $([$property_owned_context:ident])? fn owned($($property_owned_args:tt)*) $property_owned_body:block
            })?
            $(index_access($index_source_ty:ty) {
                $([$index_shared_context:ident])? fn shared($($index_shared_args:tt)*) $index_shared_body:block
                $([$index_mutable_context:ident])? fn mutable($($index_mutable_args:tt)*) $index_mutable_body:block
                $([$index_owned_context:ident])? fn owned($($index_owned_args:tt)*) $index_owned_body:block
            })?
            $(interface_items {
                $($items:item)*
            })?
        }
    ) => {
        $mod_vis mod $mod_name {
            use super::*;

            #[allow(unused)]
            #[cfg(test)]
            fn asserts() {
                fn assert_first_argument<T: IsArgument<ValueType = $type_def>>() {}
                fn assert_output_type<T: IsReturnable>() {}
                $($(
                    assert_first_argument::<handle_first_arg_type!($($method_args)*,)>();
                    if_exists! {
                        {$($method_ignore_type_assertion)?}
                        {}
                        {$(assert_output_type::<$method_output_ty>();)?}
                    }
                )*)?
                $($(
                    assert_first_argument::<handle_first_arg_type!($($unary_args)*,)>();
                    if_exists! {
                        {$($unary_ignore_type_assertion)?}
                        {}
                        {$(assert_output_type::<$unary_output_ty>();)?}
                    }
                )*)?
                $($(
                    assert_first_argument::<handle_first_arg_type!($($binary_args)*,)>();
                    if_exists! {
                        {$($binary_ignore_type_assertion)?}
                        {}
                        {$(assert_output_type::<$binary_output_ty>();)?}
                    }
                )*)?
                // Note: property_access and index_access source types are verified
                // at compile time through the apply_* wrapper functions
            }

            $(
                pub(crate) mod functions {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $function_name(if_empty!([$($function_context)?][_context]): &mut FunctionCallContext, $($function_args)*) $(-> $function_output_ty)? {
                            $function_body
                        }
                    )*
                }

                pub(crate) mod function_definitions {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $function_name() -> FunctionInterface {
                            parse_arg_types!([CALLBACK: generate_method_interface![functions::$function_name]] $($function_args)*)
                        }
                    )*
                }
            )?

            $(
                pub(crate) mod methods {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $method_name(if_empty!([$($method_context)?][_context]): &mut FunctionCallContext, $($method_args)*) $(-> $method_output_ty)? {
                            $method_body
                        }
                    )*
                }
                
                pub(crate) mod method_definitions {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $method_name() -> FunctionInterface {
                            parse_arg_types!([CALLBACK: generate_method_interface![methods::$method_name]] $($method_args)*)
                        }
                    )*
                }
            )?

            $(
                pub(crate) mod unary_operations {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $unary_name(if_empty!([$($unary_context)?][_context]): UnaryOperationCallContext, $($unary_args)*) $(-> $unary_output_ty)? {
                            $unary_body
                        }
                    )*
                }

                pub(crate) mod unary_definitions {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $unary_name() -> UnaryOperationInterface {
                            parse_arg_types!([CALLBACK: generate_unary_interface![unary_operations::$unary_name]] $($unary_args)*)
                        }
                    )*
                }
            )?

            $(
                pub(crate) mod binary_operations {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $binary_name(if_empty!([$($binary_context)?][_context]): BinaryOperationCallContext, $($binary_args)*) $(-> $binary_output_ty)? {
                            $binary_body
                        }
                    )*
                }
                pub(crate) mod binary_definitions {
                    #[allow(unused)]
                    use super::*;
                    $(
                        pub(crate) fn $binary_name() -> BinaryOperationInterface {
                            parse_arg_types!([CALLBACK: generate_binary_interface![binary_operations::$binary_name]] $($binary_args)*)
                        }
                    )*
                }
            )?

            $(
                pub(crate) mod property_access {
                    #[allow(unused)]
                    use super::*;

                    pub(crate) fn shared<'a>(if_empty!([$($property_shared_context)?][_ctx]): PropertyAccessCallContext, $($property_shared_args)*) -> ExecutionResult<&'a AnyValue> $property_shared_body

                    pub(crate) fn mutable<'a>(if_empty!([$($property_mutable_context)?][_ctx]): PropertyAccessCallContext, $($property_mutable_args)*) -> ExecutionResult<&'a mut AnyValue> $property_mutable_body

                    pub(crate) fn owned(if_empty!([$($property_owned_context)?][_ctx]): PropertyAccessCallContext, $($property_owned_args)*) -> ExecutionResult<AnyValue> $property_owned_body
                }

                pub(crate) fn property_access_interface() -> PropertyAccessInterface {
                    PropertyAccessInterface {
                        shared_access: |ctx, source| apply_property_shared::<$property_source_ty>(property_access::shared, ctx, source),
                        mutable_access: |ctx, source, auto_create| apply_property_mutable::<$property_source_ty>(property_access::mutable, ctx, source, auto_create),
                        owned_access: |ctx, source| apply_property_owned::<$property_source_ty>(property_access::owned, ctx, source),
                    }
                }
            )?

            $(
                pub(crate) mod index_access {
                    #[allow(unused)]
                    use super::*;

                    pub(crate) fn shared<'a>(if_empty!([$($index_shared_context)?][_ctx]): IndexAccessCallContext, $($index_shared_args)*) -> ExecutionResult<&'a AnyValue> $index_shared_body

                    pub(crate) fn mutable<'a>(if_empty!([$($index_mutable_context)?][_ctx]): IndexAccessCallContext, $($index_mutable_args)*) -> ExecutionResult<&'a mut AnyValue> $index_mutable_body

                    pub(crate) fn owned(if_empty!([$($index_owned_context)?][_ctx]): IndexAccessCallContext, $($index_owned_args)*) -> ExecutionResult<AnyValue> $index_owned_body
                }

                pub(crate) fn index_access_interface() -> IndexAccessInterface {
                    IndexAccessInterface {
                        index_ownership: ArgumentOwnership::Shared,
                        shared_access: |ctx, source, index| apply_index_shared::<$index_source_ty>(index_access::shared, ctx, source, index),
                        mutable_access: |ctx, source, index, auto_create| apply_index_mutable::<$index_source_ty>(index_access::mutable, ctx, source, index, auto_create),
                        owned_access: |ctx, source, index| apply_index_owned::<$index_source_ty>(index_access::owned, ctx, source, index),
                    }
                }
            )?

            impl TypeData for $type_def {
                $(
                    #[allow(unreachable_code)]
                    fn resolve_own_method(method_name: &str) -> Option<FunctionInterface> {
                        Some(match method_name {
                            $(
                                stringify!($method_name) => method_definitions::$method_name(),
                            )*
                            _ => return None,
                        })
                    }
                )?

                $(
                    #[allow(unreachable_code)]
                    fn resolve_type_function(function_name: &str) -> Option<FunctionInterface> {
                        Some(match function_name {
                            $(
                                stringify!($function_name) => function_definitions::$function_name(),
                            )*
                            _ => return None,
                        })
                    }
                )?

                define_type_features!(@property_access_impl $($property_source_ty)?);
                define_type_features!(@index_access_impl $($index_source_ty)?);

                // Pass through resolve_own_unary_operation and resolve_own_binary_operation
                // until there's a better way to define them
                $($($items)*)?
            }
        }
    };

    // Helper rules for generating resolve_own_property_access when property_access is defined
    (@property_access_impl $source_ty:ty) => {
        fn resolve_own_property_access() -> Option<PropertyAccessInterface> {
            Some(property_access_interface())
        }
    };
    (@property_access_impl) => {};

    // Helper rules for generating resolve_own_index_access when index_access is defined
    (@index_access_impl $source_ty:ty) => {
        fn resolve_own_index_access() -> Option<IndexAccessInterface> {
            Some(index_access_interface())
        }
    };
    (@index_access_impl) => {};
}

#[cfg(test)]
pub(crate) use handle_first_arg_type;
pub(crate) use {
    define_type_features, generate_binary_interface, generate_method_interface,
    generate_unary_interface, if_empty, ignore_all, parse_arg_types,
};
