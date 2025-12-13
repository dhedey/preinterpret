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
        MethodInterface::Arity0 {
            method: |context| apply_fn0($method, context),
            argument_ownership: [],
        }
    };
    (REQUIRED [$ty1:ty,] OPTIONAL [] ARGS[$method:path]) => {
        MethodInterface::Arity1 {
            method: |context, a| apply_fn1($method, a, context),
            argument_ownership: [<$ty1 as IsArgument>::OWNERSHIP],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty,] OPTIONAL [] ARGS[$method:path]) => {
        MethodInterface::Arity2 {
            method: |context, a, b| apply_fn2($method, a, b, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty, $ty3:ty,] OPTIONAL [] ARGS[$method:path]) => {
        MethodInterface::Arity3 {
            method: |context, a, b, c| apply_fn3($method, a, b, c, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
                <$ty3 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty,] OPTIONAL [$ty2:ty,] ARGS[$method:path]) => {
        MethodInterface::Arity1PlusOptional1 {
            method: |context, a, b| apply_fn1_optional1($method, a, b, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty,] OPTIONAL [$ty3:ty,] ARGS[$method:path]) => {
        MethodInterface::Arity2PlusOptional1 {
            method: |context, a, b, c| apply_fn2_optional1($method, a, b, c, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
                <$ty3 as IsArgument>::OWNERSHIP,
            ],
        }
    };
    (REQUIRED [$ty1:ty, $ty2:ty, $ty3:ty,] OPTIONAL [$ty4:ty,] ARGS[$method:path]) => {
        MethodInterface::Arity3PlusOptional1 {
            method: |context, a, b, c, d| apply_fn3_optional1($method, a, b, c, d, context),
            argument_ownership: [
                <$ty1 as IsArgument>::OWNERSHIP,
                <$ty2 as IsArgument>::OWNERSHIP,
                <$ty3 as IsArgument>::OWNERSHIP,
                <$ty4 as IsArgument>::OWNERSHIP,
            ],
        }
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
    ([REQUIRED [$($req:tt)*] OPTIONAL $opt:tt => $callback:ident! $callback_args:tt] : $type:ty) => {
        parse_arg_types!([REQUIRED[$($req)* $type,] OPTIONAL $opt => $callback! $callback_args])
    };
    // Next tokens are `: X, ...` - we have a required argument (variant 2)
    ([REQUIRED [$($req:tt)*] OPTIONAL $opt:tt => $callback:ident! $callback_args:tt] : $type:ty, $($rest:tt)*) => {
        parse_arg_types!([REQUIRED[$($req)* $type,] OPTIONAL $opt => $callback! $callback_args] $($rest)*)
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
    f: fn(&mut MethodCallContext) -> R,
    context: &mut MethodCallContext,
) -> ExecutionResult<ReturnedValue>
where
    R: IsReturnable,
{
    let _output_span_range = context.output_span_range;
    f(context).to_returned_value()
}

pub(crate) fn apply_fn1<A, R>(
    f: fn(&mut MethodCallContext, A) -> R,
    a: Spanned<ArgumentValue>,
    context: &mut MethodCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    R: IsReturnable,
{
    f(context, A::from_argument(a)?).to_returned_value()
}

#[allow(unused)]
pub(crate) fn apply_fn1_optional1<A, B, C>(
    f: fn(&mut MethodCallContext, A, Option<B>) -> C,
    a: Spanned<ArgumentValue>,
    b: Option<Spanned<ArgumentValue>>,
    context: &mut MethodCallContext,
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
    f: fn(&mut MethodCallContext, A, B) -> C,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    context: &mut MethodCallContext,
) -> ExecutionResult<ReturnedValue>
where
    A: IsArgument,
    B: IsArgument,
    C: IsReturnable,
{
    f(context, A::from_argument(a)?, B::from_argument(b)?).to_returned_value()
}

pub(crate) fn apply_fn2_optional1<A, B, C, D>(
    f: fn(&mut MethodCallContext, A, B, Option<C>) -> D,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    c: Option<Spanned<ArgumentValue>>,
    context: &mut MethodCallContext,
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
    f: fn(&mut MethodCallContext, A, B, C) -> R,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    c: Spanned<ArgumentValue>,
    context: &mut MethodCallContext,
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
    f: fn(&mut MethodCallContext, A, B, C, Option<D>) -> R,
    a: Spanned<ArgumentValue>,
    b: Spanned<ArgumentValue>,
    c: Spanned<ArgumentValue>,
    d: Option<Spanned<ArgumentValue>>,
    context: &mut MethodCallContext,
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

pub(crate) struct MethodCallContext<'a> {
    pub interpreter: &'a mut Interpreter,
    pub output_span_range: SpanRange,
}

impl<'a> HasSpanRange for MethodCallContext<'a> {
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

macro_rules! define_interface {
    (
        struct $type_data:ident,
        parent: $parent_type_data:ident,
        $mod_vis:vis mod $mod_name:ident {
            $mod_methods_vis:vis mod methods {
                $(
                    $([$method_context:ident])? fn $method_name:ident($($method_args:tt)*) $(-> $method_output_ty:ty)? $([ignore_type_assertion $method_ignore_type_assertion:tt])? $method_body:block
                )*
            }
            $mod_unary_operations_vis:vis mod unary_operations {
                $(
                    $([$unary_context:ident])? fn $unary_name:ident($($unary_args:tt)*) $(-> $unary_output_ty:ty)? $([ignore_type_assertion $unary_ignore_type_assertion:tt])? $unary_body:block
                )*
            }
            $mod_binary_operations_vis:vis mod binary_operations {
                $(
                    $([$binary_context:ident])? fn $binary_name:ident($($binary_args:tt)*) $(-> $binary_output_ty:ty)? $([ignore_type_assertion $binary_ignore_type_assertion:tt])? $binary_body:block
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
                    if_exists! {
                        {$($method_ignore_type_assertion)?}
                        {}
                        {$($type_data::assert_output_type::<$method_output_ty>();)?}
                    }
                )*
                $(
                    $type_data::assert_first_argument::<handle_first_arg_type!($($unary_args)*,)>();
                    if_exists! {
                        {$($unary_ignore_type_assertion)?}
                        {}
                        {$($type_data::assert_output_type::<$unary_output_ty>();)?}
                    }
                )*
                $(
                    $type_data::assert_first_argument::<handle_first_arg_type!($($binary_args)*,)>();
                    if_exists! {
                        {$($binary_ignore_type_assertion)?}
                        {}
                        {$($type_data::assert_output_type::<$binary_output_ty>();)?}
                    }
                )*
            }

            $mod_methods_vis mod methods {
                #[allow(unused)]
                use super::*;
                $(
                    pub(crate) fn $method_name(if_empty!([$($method_context)?][_context]): &mut MethodCallContext, $($method_args)*) $(-> $method_output_ty)? {
                        $method_body
                    }
                )*
            }

            $mod_methods_vis mod method_definitions {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_methods_vis fn $method_name() -> MethodInterface {
                        parse_arg_types!([CALLBACK: generate_method_interface![methods::$method_name]] $($method_args)*)
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
                        parse_arg_types!([CALLBACK: generate_unary_interface![unary_operations::$unary_name]] $($unary_args)*)
                    }
                )*
            }

            $mod_binary_operations_vis mod binary_operations {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_binary_operations_vis fn $binary_name(if_empty!([$($binary_context)?][_context]): BinaryOperationCallContext, $($binary_args)*) $(-> $binary_output_ty)? {
                        $binary_body
                    }
                )*
            }

            $mod_binary_operations_vis mod binary_definitions {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_binary_operations_vis fn $binary_name() -> BinaryOperationInterface {
                        parse_arg_types!([CALLBACK: generate_binary_interface![binary_operations::$binary_name]] $($binary_args)*)
                    }
                )*
            }

            impl HierarchicalTypeData for $type_data {
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

                // Pass through resolve_own_unary_operation and resolve_own_binary_operation
                // until there's a better way to define them
                $($items)*
            }
        }
    }
}

pub(crate) use {
    define_interface, generate_binary_interface, generate_method_interface,
    generate_unary_interface, handle_first_arg_type, if_empty, ignore_all, parse_arg_types,
};
